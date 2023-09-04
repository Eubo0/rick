use std::collections::HashMap;

use crate::{error, error::*};
use crate::token::*;
use crate::properties::*;

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
        while self.current().0 != Token::Eof {
            match self.current().0 {
                Token::Func => {
                    self.parse_subdef();
                },
                _ => {
                    panic!("ERROR: unimplemented top level statement.");
                }
            }
        }
    }

    fn parse_subdef(&mut self) {
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
        let props = self.symboltable.get(&name).expect("Function not found!");
        
        self.local_table.drain();
        for (name, tipe) in &props.params {
            self.local_table.insert(name.clone(), Properties{ tipe: *tipe, offset: self.local_table.len() as i32, params: vec![] });    
        }

        self.parse_statement();

        self.current_ret_type = NONE;
    }

    fn parse_statement(&mut self) {
        match self.current().0 {
            Token::Lbrace => {
                self.parse_block();
            },
            Token::If => {
                self.parse_if();
            },
            Token::While => {
                self.parse_while();
            },
            Token::Let => {
                self.parse_assign();
                self.expect(Token::Semicolon);
            },
            Token::Var => {
                self.parse_vardef();
                self.expect(Token::Semicolon);
            },
            Token::Identifier(_) => {
                self.parse_call();
                self.expect(Token::Semicolon);
            },
            Token::Read => {
                self.parse_read();
                self.expect(Token::Semicolon);
            },
            Token::Print => {
                self.parse_print();
                self.expect(Token::Semicolon);
            },
            Token::Return => {
                self.parse_return();
                self.expect(Token::Semicolon);
            },
            _ => {
                // TODO: implement proper error message
                panic!("Expected statement");
            },
        }
    }

    fn parse_block(&mut self) {
        self.expect(Token::Lbrace);
        let local_size: i32 = self.local_table.len() as i32;

        while self.current().0 != Token::Rbrace {
            self.parse_statement();
        }

        // XXX: this is important for scoping of variables
        self.local_table.retain(|_, p| p.offset < local_size);

        self.expect(Token::Rbrace);
    }

    fn parse_if(&mut self) {
        unimplemented!()
    }

    fn parse_while(&mut self) {
        unimplemented!()
    }

    fn parse_assign(&mut self) {
        unimplemented!()
    }

    fn parse_vardef(&mut self) {
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

    }

    fn parse_call(&mut self) {
        unimplemented!()
    }

    fn parse_read(&mut self) {
        unimplemented!()
    }

    fn parse_print(&mut self) {
        unimplemented!()
    }

    fn parse_return(&mut self) {
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
            panic!("ERROR: return statement missing an expression.")
        }
    }

    fn parse_expr(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_simple(parent_type);

        if self.current().0.is_relational_op() {
            self.next_token();

            self.parse_expr(&mut rhs);
        }
    }

    fn parse_simple(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_term(parent_type);

        while self.current().0.is_additive_op() {
            self.next_token();

            self.parse_term(&mut rhs);
        }
    }

    fn parse_term(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_factor(parent_type);

        while self.current().0.is_multiplicative_op() {
            self.next_token();

            self.parse_factor(&mut rhs);
        }
    }

    fn parse_factor(&mut self, parent_type: &mut u8) {
        let mut rhs: u8 = NONE;

        self.parse_base(parent_type);

        while self.current().0.is_exponent_op() {
            self.next_token();

            self.parse_base(&mut rhs);
        }
    }

    fn parse_base(&mut self, parent_type: &mut u8) {
        match self.current().0 {
            Token::Identifier(_) => {
                let mut id: String = String::new();
                self.expect_identifier(&mut id);

                if self.current().0 == Token::Lpar {

                } else if self.current().0 == Token::Lbrack {

                } else {
                    // TODO: better error reporting
                    let props = self.local_table.get(&id).expect("Variable does not exist!");
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