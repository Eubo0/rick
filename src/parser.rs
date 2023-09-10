use std::collections::HashMap;

use crate::{error, error::*};
use crate::token::*;
use crate::{properties::*};
use crate::ast::*;
use crate::value::Value;

pub struct Parser {
    tokens: Vec<(Token, (u32, u32))>,

    symboltable: HashMap<String, Properties>,

    local_table: HashMap<String, Properties>,

    current_ret_type: u8,

    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, (u32, u32))>) -> Parser {
        Parser {
            tokens,
            symboltable: HashMap::new(),
            local_table: HashMap::new(),
            current_ret_type: NONE,
            idx: 0,
        }
    }

    // XXX: This is a 2-pass "compiler"
    // I don't want to forward declare things.
    // I'm not as good at language design as His Worshipfulness.
    pub fn parse_tok_stream(&mut self) -> ASTNode {
        self.parse_func_signatures();   // Pass 1
        self.idx = 0;
        let root: ASTNode = self.parse_program();   // Pass 2

        root
    }

    fn parse_func_signatures(&mut self) {
        while self.current().0 != Token::Eof {
            if self.current().0 == Token::Func {
                self.parse_func_type_info();
            } else {
                self.next_token();
            }
        }
    }

    fn parse_program(&mut self) -> ASTNode {
        let mut top_level: Vec<Box<ASTNode>> = vec![];

        while self.current().0 != Token::Eof {
            match self.current().0 {
                Token::Func => {
                    top_level.push(Box::new(self.parse_subdef()));
                },
                _ => {
                    panic!("ERROR: unimplemented top level statement '{}'", self.current().0);
                }
            }
        }

        let t: ASTNode = ASTNode::Toplevel{ funcdefs: top_level };

        t
    }

    fn parse_subdef(&mut self) -> ASTNode {
        // TODO: I'm exploiting the fact that the first pass has already collected the 
        //      function signature. I should probably do something better than skipping 
        //      to the closing parentheses.
        
        self.next_token();

        let mut name: String = String::new();
        self.expect_identifier(&mut name);
        
        while self.current().0 != Token::Rpar {
            self.next_token();
        }
        self.next_token();

        if self.current().0.is_type_start() {
            self.current_ret_type = self.parse_type();
        } else {
            self.current_ret_type = NONE;
        }

        // TODO: proper error reporting
        let props = self.symboltable.get(&name).expect("Function not found!").clone();
        
        self.local_table.drain();
        for (name, tipe) in &props.params {
            self.local_table.insert(name.clone(), Properties{ tipe: *tipe, offset: Some(self.local_table.len() as u32), params: vec![] });    
        }

        let body: ASTNode = self.parse_statement();

        self.current_ret_type = NONE;

        ASTNode::Funcdef {
            name,
            params: props.params,
            ret_type: props.tipe,
            body: Box::new(body),
        }
    }

    fn parse_statement(&mut self) -> ASTNode {
        match self.current().0 {
            Token::Lbrace => {
                let block: ASTNode = self.parse_block();
                return block;
            },
            Token::If => {
                let if_statement: ASTNode = self.parse_if();
                return if_statement;
            },
            Token::While => {
                let while_statement: ASTNode = self.parse_while();
                return while_statement;
            },
            Token::Let => {
                let assign_statement: ASTNode = self.parse_assign();
                self.expect(Token::Semicolon);

                return assign_statement;
            },
            Token::Var => {
                let vardef_statement = self.parse_vardef();
                self.expect(Token::Semicolon);

                return vardef_statement;
            },
            Token::Identifier(_) => {
                let call_statement: ASTNode = self.parse_call();
                self.expect(Token::Semicolon);

                return call_statement;
            },
            Token::Read => {
                let read_statement: ASTNode = self.parse_read();
                self.expect(Token::Semicolon);

                return read_statement;
            },
            Token::Print => {
                let print_statement: ASTNode = self.parse_print();
                self.expect(Token::Semicolon);

                return print_statement;
            },
            Token::Return => {
                let return_statement: ASTNode = self.parse_return();
                self.expect(Token::Semicolon);

                return return_statement;
            },
            _ => {
                // TODO: implement proper error message
                panic!("Expected statement");
            },
        }

        // ASTNode::Value{val: Value::Integer(69)}
    }

    fn parse_block(&mut self) -> ASTNode {
        let mut stats: Vec<Box<ASTNode>> = vec![];

        self.expect(Token::Lbrace);
        let local_size: u32 = self.local_table.len() as u32;

        while self.current().0 != Token::Rbrace {
            stats.push( Box::new(self.parse_statement()) );
        }

        // XXX: this is important for scoping of variables
        self.local_table.retain(|_, p| p.offset.unwrap() < local_size);

        self.expect(Token::Rbrace);

        ASTNode::Block {
            statements: stats,
        }
    }

    fn parse_if(&mut self) -> ASTNode {
        let cond: Box<ASTNode>;
        let stat: Box<ASTNode>;

        let mut branches: Vec<(Box<ASTNode>, Box<ASTNode>)> = vec![];
        let mut else_case: Option<Box<ASTNode>> = None;

        let mut cond_type: u8 = NONE;

        self.expect(Token::If);

        cond = Box::new(self.parse_expr(&mut cond_type));

        if cond_type != BOOLEAN {
            panic!("Expected boolean, found {} for if condition", type_string(cond_type));
        }

        stat = Box::new(self.parse_statement());
        branches.push((cond, stat));

        while self.current().0 == Token::Elif {
            let else_cond: Box<ASTNode>;
            let else_stat: Box<ASTNode>;

            self.next_token();
            else_cond = Box::new(self.parse_expr(&mut cond_type));

            if cond_type != BOOLEAN {
                panic!("Expected boolean, found {} for elif condition", type_string(cond_type));
            }
            else_stat = Box::new(self.parse_statement());

            branches.push((else_cond, else_stat));
        }

        if self.current().0 == Token::Else {
            self.next_token();
            else_case = Some(Box::new(self.parse_statement()));
        }

        ASTNode::If {
            branches,
            else_case,
        }
    }

    fn parse_while(&mut self) -> ASTNode {
        let expr: Box<ASTNode>;
        let stat: Box<ASTNode>;
        let mut cond_type: u8 = NONE;
        
        self.expect(Token::While);

        expr = Box::new(self.parse_expr(&mut cond_type));
        if cond_type != BOOLEAN {
            panic!("Expected boolean, found {} for while condition", type_string(cond_type));
        }

        stat = Box::new(self.parse_statement());

        ASTNode::While {
            condition: expr,
            statement: stat,
        }
    }

    fn parse_assign(&mut self) -> ASTNode {
        let mut name: String = String::new();
        let props: Properties;
        let mut tipe: u8;
        let mut index: Option<Box<ASTNode>> = None;
        let mut is_array: bool = false;
        let right_expr: Box<ASTNode>;

        self.expect(Token::Let);

        self.expect_identifier(&mut name);

        // TODO: better error reporting
        props = self.local_table.get(&name).expect("Undefined variable").clone();
        tipe = props.tipe;

        if self.current().0 == Token::Lbrack {
            if tipe | ARRAY == 0 {
                // TODO: better error reporting
                panic!("Cannot index non-array type");
            }

            index = Some(Box::new(self.parse_index()));

            tipe ^= ARRAY;
        }

        self.expect(Token::Assign);

        let mut rhs: u8 = NONE;

        if self.current().0 == Token::Array {
            if tipe | ARRAY == 0 {
                // TODO: better error reporting
                panic!("Can't assign array to non-array variable");
            }
            is_array = true;
            self.next_token();

            let mut simp_type: u8 = NONE;
            right_expr = Box::new(self.parse_simple(&mut simp_type));

            rhs = simp_type | ARRAY;
        } else {
            right_expr = Box::new(self.parse_expr(&mut rhs));
        }

        if rhs != tipe {
            // TODO: better error reporting
            panic!("Type mismatch in assignment.");
        }

        ASTNode::Let {
            offset: props.offset.unwrap(),
            index,
            is_array,
            rhs: right_expr,
        }
    }

    fn parse_vardef(&mut self) -> ASTNode {
        let mut id: String = String::new(); 
        let tipe: u8;
        let mut names: Vec<String> = vec![];

        self.expect(Token::Var);

        tipe = self.parse_type();

        self.expect_identifier(&mut id);
        names.push(id.clone());

        if self.local_table.insert(id.clone(), Properties {tipe, offset: Some(self.local_table.len() as u32), params: vec![]}).is_some() {
            // TODO: better error reporting
            panic!("Error: Multiple definition of local variable.");
        }

        while self.current().0 == Token::Comma {
            self.next_token();
            self.expect_identifier(&mut id);
            names.push(id.clone());

            if self.local_table.insert(id.clone(), Properties {tipe, offset: Some(self.local_table.len() as u32), params: vec![]}).is_some() {
                // TODO: better error reporting
                panic!("Error: Multiple definition of local variable.");
            }
        }

        ASTNode::VarDef {
            tipe,
            names
        }
    }

    fn parse_index(&mut self) -> ASTNode {
        let index: ASTNode;
        let mut index_type: u8 = NONE;
        self.expect(Token::Lbrack);

        index = self.parse_simple(&mut index_type);

        if index_type != INTEGER {
            // TODO: better error reporting
            panic!("Invalid index value");
        }

        self.expect(Token::Rbrack);
    
        index
    }

    fn parse_call(&mut self) -> ASTNode {
        let mut id: String = String::new();
        self.expect_identifier(&mut id);

        if !self.symboltable.contains_key(&id) {
            // TODO: better error reporting
            panic!("Identifier '{}' does not exist as a function", id);
        }

        let props = self.symboltable.get(&id).unwrap().clone();
        if props.tipe & !FUNC != 0 {
            // TODO: better error reporting
            panic!("'{}' is not a procedure", id);
        }

        let args: Vec<Box<ASTNode>> = self.parse_arglist(props, id.clone());

        ASTNode::Call {
            name: id,
            args: args,
        }
    }

    fn parse_read(&mut self) -> ASTNode {
        let mut id: String = String::new();
        let output: ASTNode;

        self.expect(Token::Read);
        self.expect(Token::Lpar);

        self.expect_identifier(&mut id);

        // TODO: better error reporting
        let props = self.local_table.get(&id).expect("Variable doesnt exist!");

        if self.current().0 == Token::Lbrack {
            if props.tipe & ARRAY == 0 {
                // TODO: proper error reporting
                panic!("Not an array!");
            }
            let idx: ASTNode = self.parse_index();
            output = ASTNode::Read { name: id, maybe_index: Some(Box::new(idx)) };
        } else {
            if props.tipe & ARRAY != 0 {
                // TODO: better error reporting
                panic!("Index array before reading input");
            }
            // TODO: 'get' variable with id 
            output = ASTNode::Read { name: id, maybe_index: None };
        }

        self.expect(Token::Rpar);

        output
    }

    fn parse_print(&mut self) -> ASTNode {
        let mut expr_type: u8 = NONE;
        let mut items: Vec<Box<ASTNode>> = vec![];

        self.expect(Token::Print);
        self.expect(Token::Lpar);

        loop {
            let expr: ASTNode = self.parse_expr(&mut expr_type);
            
            items.push(Box::new(expr));

            if self.current().0 != Token::Concat {
                break;
            }
            self.next_token();
        }

        self.expect(Token::Rpar);

        ASTNode::Print {
            items,
        }
    }

    fn parse_return(&mut self) -> ASTNode {
        let mut expr_type: u8 = NONE;
        let mut ret_expr: Option<Box<ASTNode>> = None;

        self.expect(Token::Return);

        if self.current().0.start_expression() {
            ret_expr = Some(Box::new(self.parse_expr(&mut expr_type)));
            if expr_type != self.current_ret_type {
                // TODO: Proper error reporting
                panic!("ERROR: incorrect type for return expression.");
            }

        } else if self.current_ret_type != NONE {
            // TODO: Proper error reporting
            panic!("ERROR: return statement missing an expression.");
        }

        ASTNode::Return{ expr: ret_expr }
    }

    fn parse_expr(&mut self, parent_type: &mut u8) -> ASTNode {
        let mut output: ASTNode;
        let mut rhs: u8 = NONE;

        output = self.parse_simple(parent_type);

        if self.current().0.is_ordering_op() {
            if !is_numeric_type(*parent_type) {
                panic!("Expected numeric type, found {}", type_string(*parent_type));
            }
        }

        if self.current().0.is_relational_op() {
            let op = self.current().0;
            self.next_token();

            output = ASTNode::BinaryOp { 
                        lhs: Box::new(output), 
                        op,
                        rhs: Box::new(self.parse_expr(&mut rhs))
                    } ;

            if is_numeric_type(*parent_type) && !is_numeric_type(rhs) {
                panic!("Expected numeric type, found {} type", type_string(rhs));
            } 
            if !is_numeric_type(*parent_type) && *parent_type != rhs {
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }

            *parent_type = BOOLEAN;
        }

        output
    }

    fn parse_simple(&mut self, parent_type: &mut u8) -> ASTNode {
        let mut output: ASTNode;
        let mut rhs: u8 = NONE;

        if self.current().0 == Token::Sub {
            self.next_token();
            output = ASTNode::UnaryOp { op: Token::Sub, value: Box::new(self.parse_term(parent_type)) };
        } else {
            output = self.parse_term(parent_type);
        }

        if self.current().0 == Token::Or {
            if *parent_type != BOOLEAN {
                panic!("Expected boolean type, found {}", type_string(*parent_type));
            }
        }

        while self.current().0.is_additive_op() {
            let op = self.current().0;
            self.next_token();

            output = ASTNode::BinaryOp {
                        lhs: Box::new(output),
                        op,
                        rhs: Box::new(self.parse_term(&mut rhs))
                    };

            if *parent_type != rhs {
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }
        }

        output
    }

    fn parse_term(&mut self, parent_type: &mut u8) -> ASTNode {
        let mut output: ASTNode;
        let mut rhs: u8 = NONE;

        output = self.parse_factor(parent_type);

        if self.current().0 == Token::And {
            if *parent_type != BOOLEAN {
                // TODO: better error reporting
                panic!("Expected boolean type, found {} type", type_string(*parent_type));
            }
        }

        while self.current().0.is_multiplicative_op() {
            let op = self.current().0;
            self.next_token();

            output = ASTNode::BinaryOp {
                        lhs: Box::new(output),
                        op,
                        rhs: Box::new(self.parse_factor(&mut rhs))
                    };

            if rhs != *parent_type {
                // TODO: better error reporting
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }
        }

        output
    }

    fn parse_factor(&mut self, parent_type: &mut u8) -> ASTNode {
        let mut output: ASTNode;
        let mut rhs: u8 = NONE;

        output = self.parse_base(parent_type);

        if self.current().0.is_exponent_op() {
            if parent_type != &INTEGER {
                panic!("Expected {} type, found {} type", type_string(INTEGER), type_string(*parent_type));
            }
        }

        while self.current().0.is_exponent_op() {
            let op = self.current().0;
            self.next_token();

            output = ASTNode::BinaryOp {
                        lhs: Box::new(output),
                        op,
                        rhs: Box::new(self.parse_base(&mut rhs))
                    };

            if rhs != INTEGER {
                panic!("Expected {} type, found {} type", type_string(INTEGER), type_string(rhs));
            }
        }

        output
    }

    fn parse_base(&mut self, parent_type: &mut u8) -> ASTNode {
        match self.current().0 {
            Token::Identifier(_) => {
                let mut id: String = String::new();
                self.expect_identifier(&mut id);

                if self.current().0 == Token::Lpar {
                    // TODO: proper error reportings
                    let props = self.symboltable.get(&id).expect("Variable does not exist!").clone();
                    let tipe = props.tipe;
                    if props.tipe & FUNC == 0 {
                        // TODO: better error reporting
                        panic!("{} is not a callable", id);
                    }
                    let args: Vec<Box<ASTNode>> = self.parse_arglist(props, id.clone());
                    *parent_type = tipe & !FUNC;

                    return ASTNode::Call {
                        name: id,
                        args,
                    };

                } else if self.current().0 == Token::Lbrack {
                    let props = self.local_table.get(&id).expect("Variable does not exist!").clone();
                    if props.tipe & ARRAY == 0 {
                        // TODO: proper error reporting
                        panic!("Trying to index non-array '{}'", id);
                    }
                    
                    *parent_type = props.tipe & !ARRAY;

                    return ASTNode::GetIndex {
                        offset: props.offset.unwrap(),
                        idx: Box::new(self.parse_index())
                    }

                } else {
                    let props = self.local_table.get(&id).expect("Variable does not exist!").clone();
                    // TODO: better error reporting
                    *parent_type = props.tipe;

                    return ASTNode::GetVar {
                        name: id,
                        offset: props.offset.unwrap(),
                    };
                }
            },
            Token::FloatLiteral(f) => {
                self.next_token();
                *parent_type = FLOAT;

                return ASTNode::Value {
                    val: Value::Float(f)
                };
            },
            Token::IntegerLiteral(i) => {
                self.next_token();
                *parent_type = INTEGER;

                return ASTNode::Value {
                    val: Value::Integer(i)
                };
            },
            Token::StringLiteral(s) => {
                self.next_token();
                *parent_type = STRING;

                return ASTNode::Value {
                    val: Value::String(s)
                };
            },
            Token::Lpar => {
                let output: ASTNode;

                self.next_token();
                output = self.parse_expr(parent_type);
                self.expect(Token::Rpar);

                return output;
            },
            Token::Negate => {
                self.next_token();

                return ASTNode::UnaryOp {
                    op: Token::Negate,
                    value: Box::new(self.parse_base(parent_type))
                };
            },
            Token::True => {
                self.next_token();
                *parent_type = BOOLEAN;

                return ASTNode::Value {
                    val: Value::Boolean(true),
                };
            },
            Token::False => {
                self.next_token();
                *parent_type = BOOLEAN;

                return ASTNode::Value {
                    val: Value::Boolean(false),
                };
            },

            _ => {
                // TODO: proper error reporting
                panic!("ERROR: expected valid expression base: '{:#?}', {:#?}", self.current().0, self.current().1);
            }
        }
    }


    fn parse_arglist(&mut self, props: Properties, id: String) -> Vec<Box<ASTNode>> {
        let mut output: Vec<Box<ASTNode>> = vec![];
        let mut i: usize = 0;
        self.expect(Token::Lpar);

        if self.current().0.start_expression() {
            let mut expr_type: u8 = NONE;
            
            output.push(Box::new(self.parse_expr(&mut expr_type)));

            if i + 1 > props.params.len() {
                // TODO: proper error reporting
                panic!("Too many arguments for {}", id);
            }
            if expr_type != props.params[i].1 {
                // TODO: proper error reporting
                panic!("Type mismatch for argument {} of {}", i, id);
            }

            i += 1;
            
            while self.current().0 == Token::Comma {
                let mut expr_type: u8 = NONE;
                self.next_token();
                output.push(Box::new(self.parse_expr(&mut expr_type)));
                if i + 1 > props.params.len() {
                    // TODO: proper error reporting
                    panic!("Too many arguments for {}", id);
                }
                if expr_type != props.params[i].1 {
                    // TODO: proper error reporting
                    panic!("Type mismatch for argument {} of {}", i, id);
                }

                i += 1;
            }
        }

        if i != props.params.len() {
            panic!("Too few arguments to {}", id);
        }

        self.expect(Token::Rpar);

        output
    }


// #######################################################################
// ####################### UTILITY FUNCTIONS #############################
// #######################################################################
    fn parse_func_type_info(&mut self) {  
        let mut name: String = String::new();
        let mut args: Vec<(String, u8)> = vec![];
        let mut ret_type: u8 = NONE;

        self.next_token();

        self.expect_identifier(&mut name);

        self.expect(Token::Lpar);

        if self.current().0.is_type_start() {
            let mut t: u8 = self.parse_type();
            
            let mut id: String = String::new();
            self.expect_identifier(&mut id);

            args.push((id, t));

            while self.current().0 == Token::Comma {
                let mut id: String = String::new();
                self.next_token();

                t = self.parse_type();
                self.expect_identifier(&mut id);

                args.push((id, t));
            }
        }
        
        self.expect(Token::Rpar);

        if self.current().0.is_type_start() {
            ret_type = self.parse_type();
            ret_type |= FUNC;
        }

        let props: Properties = Properties {
            tipe: ret_type,
            offset: None,
            params: args,
        };

        self.symboltable.insert(name, props);
    }

    fn parse_type(&mut self) -> u8 {
        let mut output: u8 = NONE;
        
        if !self.current().0.is_type_start() {
            let (line, col) = self.current().1;
            error::set_loc(line, col);
            report_err(RickError::MissingTypeSpecifier);
        }

        match self.current().0 {
            Token::Integer => {
                output |= INTEGER;
            },
            Token::Float => {
                output |= FLOAT;
            },
            Token::Boolean => {
                output |= BOOLEAN;
            },
            Token::String => {
                output |= STRING;
            }
            _ => {
                panic!("Unreachable!");
            },
        }

        self.next_token();

        if self.current().0 == Token::Array {
            output |= ARRAY;
            self.next_token();
        }

        output
    }

    fn expect(&mut self, expected: Token) {
        if std::mem::discriminant(&expected) == std::mem::discriminant(&self.current().0) {
            self.next_token();
        } else {
            let (line, col) = self.current().1;
            error::set_loc(line, col);
            report_err(RickError::Expected(self.current().0, expected));
        }
    }

    fn expect_identifier(&mut self, target: &mut String) {
        if let (Token::Identifier(id), (_, _)) = self.current() {
            *target = id;
            self.next_token();
        } else {
            let (line, col) = self.current().1;
            error::set_loc(line, col);
            report_err(RickError::Expected(self.current().0, Token::Identifier("identifier".into())));
        }
    }

    fn next_token(&mut self) {
        if self.idx < self.tokens.len() - 1 {
            self.idx += 1;
        }
    }

    #[inline(always)]
    fn current(&self) -> (Token, (u32, u32)) {
        self.tokens[self.idx].clone()
    }
}