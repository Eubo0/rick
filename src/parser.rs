use std::collections::HashMap;

use crate::{error, error::*};
use crate::token::*;
use crate::{properties, properties::*};
use crate::ast::*;

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
    pub fn parse_tok_stream(&mut self) {
        self.parse_func_signatures();   // Pass 1
        self.idx = 0;
        self.parse_program();           // Pass 2
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

    fn parse_program(&mut self) {
        let mut top_level: Vec<Box<ASTNode>> = vec![];

        while self.current().0 != Token::Eof {
            match self.current().0 {
                Token::Func => {
                    top_level.push(Box::new(self.parse_subdef()));
                },
                _ => {
                    panic!("ERROR: unimplemented top level statement.");
                }
            }
        }

        let t: ASTNode = ASTNode::Toplevel{ funcdefs: top_level };

        println!("{:#?}", t);
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
            self.local_table.insert(name.clone(), Properties{ tipe: *tipe, offset: self.local_table.len() as i32, params: vec![] });    
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
        let local_size: i32 = self.local_table.len() as i32;

        while self.current().0 != Token::Rbrace {
            stats.push( Box::new(self.parse_statement()) );
        }

        // XXX: this is important for scoping of variables
        self.local_table.retain(|_, p| p.offset < local_size);

        self.expect(Token::Rbrace);

        ASTNode::Block {
            statements: stats,
        }
    }

    fn parse_if(&mut self) -> ASTNode {
        let cond: Box<ASTNode>;
        let first_stat: Box<ASTNode>;

        let mut elif_branches: Vec<(Box<ASTNode>, Box<ASTNode>)> = vec![];
        let mut else_case: Option<Box<ASTNode>> = None;

        let mut cond_type: u8 = NONE;

        self.expect(Token::If);

        self.parse_expr(&mut cond_type);
        cond = Box::new(ASTNode::Value{ val: Value::String("condition".into())} );

        if cond_type != BOOLEAN {
            panic!("Expected boolean, found {} for if condition", type_string(cond_type));
        }

        first_stat = Box::new(self.parse_statement());

        while self.current().0 == Token::Elif {
            let else_cond: Box<ASTNode>;
            let else_stat: Box<ASTNode>;

            self.next_token();
/* else_cond = */ self.parse_expr(&mut cond_type);
            else_cond = Box::new(ASTNode::Value{ val: Value::String("condition".into())} );

            if cond_type != BOOLEAN {
                panic!("Expected boolean, found {} for elif condition", type_string(cond_type));
            }
            else_stat = Box::new(self.parse_statement());

            elif_branches.push((else_cond, else_stat));
        }

        if self.current().0 == Token::Else {
            self.next_token();
            else_case = Some(Box::new(self.parse_statement()));
        }

        ASTNode::If {
            first_condition: cond,
            first_statement: first_stat,
            elif_branches,
            else_case,
        }
    }

    fn parse_while(&mut self) -> ASTNode {
        let expr: Box<ASTNode>;
        let stat: Box<ASTNode>;
        let mut cond_type: u8 = NONE;
        
        self.expect(Token::While);

        self.parse_expr(&mut cond_type);
        expr = Box::new(ASTNode::Value{ val: Value::String("expression".into()) });
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
        let mut props: Properties;
        let mut tipe: u8;

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

            self.parse_index();

            tipe ^= ARRAY;
        }

        self.expect(Token::Assign);

        let mut rhs: u8 = NONE;

        if self.current().0 == Token::Array {
            if tipe | ARRAY == 0 {
                // TODO: better error reporting
                panic!("Can't assign array to non-array variable");
            }
            self.next_token();

            let mut simp_type: u8 = NONE;
            self.parse_simple(&mut simp_type);

            rhs = simp_type | ARRAY;
        } else {
            self.parse_expr(&mut rhs);
        }

        if rhs != tipe {
            // TODO: better error reporting
            panic!("Type mismatch in assignment.");
        }

        todo!()
    }

    fn parse_vardef(&mut self) -> ASTNode {
        let mut id: String = String::new(); 
        let mut tipe: u8 = NONE;

        self.expect(Token::Var);

        tipe = self.parse_type();

        self.expect_identifier(&mut id);

        if self.local_table.insert(id.clone(), Properties {tipe, offset: self.local_table.len() as i32, params: vec![]}).is_some() {
            // TODO: better error reporting
            panic!("Error: Multiple definition of local variable.");
        }

        while self.current().0 == Token::Comma {
            self.next_token();
            self.expect_identifier(&mut id);

            if self.local_table.insert(id.clone(), Properties {tipe, offset: self.local_table.len() as i32, params: vec![]}).is_some() {
                // TODO: better error reporting
                panic!("Error: Multiple definition of local variable.");
            }
        }

        todo!()
    }

    fn parse_index(&mut self) -> ASTNode {
        let mut index_type: u8 = NONE;
        self.expect(Token::Lbrack);

        self.parse_simple(&mut index_type);

        if index_type != INTEGER {
            // TODO: better error reporting
            panic!("Invalid index value");
        }

        self.expect(Token::Rbrack);
    
        ASTNode::Value{val: Value::String("index".into())}
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
            self.parse_expr(&mut expr_type);
            items.push(Box::new(ASTNode::Value{ val: Value::Integer(69) }));

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

        self.expect(Token::Return);

        if self.current().0.start_expression() {
            self.parse_expr(&mut expr_type);
            if expr_type != self.current_ret_type {
                // TODO: Proper error reporting
                panic!("ERROR: incorrect type for return expression.");
            }

        } else if self.current_ret_type != NONE {
            // TODO: Proper error reporting
            panic!("ERROR: return statement missing an expression.");
        }

        ASTNode::Return { expr: Some( Box::new(ASTNode::Value{ val: Value::Integer(69) })) }
    }

    fn parse_expr(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_simple(parent_type);

        if self.current().0.is_ordering_op() {
            if !is_numeric_type(*parent_type) {
                panic!("Expected numeric type, found {}", type_string(*parent_type));
            }
        }

        if self.current().0.is_relational_op() {
            self.next_token();

            self.parse_expr(&mut rhs);

            if is_numeric_type(*parent_type) && !is_numeric_type(rhs) {
                panic!("Expected numeric type, found {} type", type_string(rhs));
            } 
            if !is_numeric_type(*parent_type) && *parent_type != rhs {
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }

            *parent_type = BOOLEAN;
        }
    }

    fn parse_simple(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_term(parent_type);

        if self.current().0 == Token::Or {
            if *parent_type != BOOLEAN {
                panic!("Expected boolean type, found {}", type_string(*parent_type));
            }
        }

        while self.current().0.is_additive_op() {
            self.next_token();

            self.parse_term(&mut rhs);

            if *parent_type != rhs {
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }
        }
    }

    fn parse_term(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_factor(parent_type);

        if self.current().0 == Token::And {
            if *parent_type != BOOLEAN {
                // TODO: better error reporting
                panic!("Expected boolean type, found {} type", type_string(*parent_type));
            }
        }

        while self.current().0.is_multiplicative_op() {
            self.next_token();

            self.parse_factor(&mut rhs);

            if rhs != *parent_type {
                // TODO: better error reporting
                panic!("Expected {} type, found {} type", type_string(*parent_type), type_string(rhs));
            }
        }
    }

    fn parse_factor(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_base(parent_type);

        if self.current().0.is_exponent_op() {
            if parent_type != &INTEGER {
                panic!("Expected {} type, found {} type", type_string(INTEGER), type_string(*parent_type));
            }
        }

        while self.current().0.is_exponent_op() {
            self.next_token();

            self.parse_base(&mut rhs);
            if rhs != INTEGER {
                panic!("Expected {} type, found {} type", type_string(INTEGER), type_string(rhs));
            }
        }
    }

    fn parse_base(&mut self, parent_type: &mut u8) {
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
                    self.parse_arglist(props, id);
                    *parent_type = tipe & !FUNC;

                } else if self.current().0 == Token::Lbrack {
                    let props = self.local_table.get(&id).expect("Variable does not exist!").clone();
                    if props.tipe & ARRAY == 0 {
                        // TODO: proper error reporting
                        panic!("Trying to index non-array '{}'", id);
                    }
                    self.parse_index();
                    *parent_type = props.tipe & !ARRAY;

                } else {
                    let props = self.local_table.get(&id).expect("Variable does not exist!").clone();
                    // TODO: better error reporting
                    *parent_type = props.tipe;
                }
            },
            Token::FloatLiteral(_) => {
                self.next_token();
                *parent_type = FLOAT;
            },
            Token::IntegerLiteral(_) => {
                self.next_token();
                *parent_type = INTEGER;
            },
            Token::StringLiteral(_) => {
                self.next_token();
                *parent_type = STRING;
            },
            Token::Lpar => {
                self.next_token();
                self.parse_expr(parent_type);
                self.expect(Token::Rpar);
            },
            Token::Negate => {
                self.next_token();
                self.parse_base(parent_type);
            },
            Token::True => {
                self.next_token();
                *parent_type = BOOLEAN;
            },
            Token::False => {
                self.next_token();
                *parent_type = BOOLEAN;
            },

            _ => {
                // TODO: proper error reporting
                panic!("ERROR: expected valid expression base!");
            }
        }
    }


    fn parse_arglist(&mut self, props: Properties, id: String) -> Vec<Box<ASTNode>> {
        let mut output: Vec<Box<ASTNode>> = vec![];
        let mut i: usize = 0;
        self.expect(Token::Lpar);

        if self.current().0.start_expression() {
            let mut expr_type: u8 = NONE;
            self.parse_expr(&mut expr_type);
            output.push(Box::new(ASTNode::Value{val: Value::String("argument".into())}));

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
                self.parse_expr(&mut expr_type);
                output.push(Box::new(ASTNode::Value{val: Value::String("argument".into())}));
                if i + 1 > props.params.len() {
                    // TODO: proper error reporting
                    panic!("Too many arguments for {}", id);
                }
                if expr_type != props.params[i].1 {
                    // TODO: proper error reporting
                    panic!("Type mismatch for argument {} of {}", i, id);
                }
            }
        }

        if i < props.params.len() {
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
            offset: -1,
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