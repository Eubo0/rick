use std::collections::HashMap;

use crate::{error, error::*};
use crate::token::*;
use crate::properties::*;

pub struct Parser {
    tokens: Vec<(Token, (u32, u32))>,

    symboltable: HashMap<String, Properties>,

    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, (u32, u32))>) -> Parser {
        Parser {
            tokens,
            symboltable: HashMap::new(),
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

        while self.current().0 != Token::Rpar {
            self.next_token();
        }
        self.next_token();

        if self.current().0.is_type_start() {
            self.parse_type();
        }
        self.parse_statement();
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

        while self.current().0 != Token::Rbrace {
            self.parse_statement();
        }

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
        unimplemented!()
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
        unimplemented!()
    }

    fn parse_expr(&mut self) {
        unimplemented!()   
    }

    fn parse_simple(&mut self) {

    }

    fn parse_term(&mut self) {

    }

    fn parse_factor(&mut self) {
        // base { '**' base}
    }

    fn parse_base(&mut self) {
        match self.current().0 {
            Token::Identifier(_) => {

            },
            Token::FloatLiteral(_) => {

            },
            Token::IntegerLiteral(_) => {

            },
            Token::StringLiteral(_) => {

            },
            Token::Lpar => {

            },
            Token::Negate => {

            },
            Token::True => {

            },
            Token::False => {

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
        let mut args: Vec<(String, [bool; 7])> = vec![];
        let mut ret_type: [bool; 7] = [true, false, false, false, false, false, true];

        self.next_token();

        self.expect_identifier(&mut name);

        self.expect(Token::Lpar);

        if self.current().0.is_type_start() {
            let mut t: [bool; 7] = self.parse_type();
            
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
            ret_type[FUNC] = true;
        }

        let props: Properties = Properties {
            tipe: ret_type,
            params: args,
        };

        self.symboltable.insert(name, props);
    }

    fn parse_type(&mut self) -> [bool; 7] {
        let mut output: [bool; 7] = [false; 7];
        
        if !self.current().0.is_type_start() {
            let (line, col) = self.current().1;
            error::set_loc(line, col);
            report_err(RickError::MissingTypeSpecifier);
        }

        match self.current().0 {
            Token::Integer => {
                output[INTEGER] = true;
            },
            Token::Float => {
                output[FLOAT] = true;
            },
            Token::Boolean => {
                output[BOOLEAN] = true;
            },
            Token::String => {
                output[STRING] = true;
            }
            _ => {
                panic!("Unreachable!");
            },
        }

        self.next_token();

        if self.current().0 == Token::Array {
            output[ARRAY] = true;
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