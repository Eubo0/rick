use crate::{error, *};

use crate::token::*;

#[derive(Debug)]
pub struct Scanner {
    source: Vec<char>,
    idx: usize,

    last_read: char,

    pub tok_stream: Vec<Token>,
}

impl Scanner {
    pub fn new(filename: String) -> Scanner {
        let source: Vec<char> = std::fs::read_to_string(filename).unwrap().chars().collect();

        Scanner {
            source,
            idx: 0,
            tok_stream: vec![],
            last_read: '\0',
        }
    }

    pub fn scan_source(&mut self) {
        let mut tok: Token = self.get_token();

        while tok != Token::Eof {
            self.tok_stream.push(tok);
            tok = self.get_token();
        }
    }

    fn get_token(&mut self) -> Token {
        let mut output: Token = Token::Eof;

        self.skip_whitespace();
        
        if self.is_eof() {
            return output;
        }

        if matches!(self.ch(), 'A'..='Z' | 'a'..='z' | '_') {
            return self.scan_word();

        } else if matches!(self.ch(), '0'..='9') {
            return self.scan_number();

        } else if self.ch() == '"' {
            return self.scan_string();

        } else {
            match self.ch() {
                '(' => {
                    output = Token::Lpar;
                },

                ')' => {
                    output = Token::Rpar;                    
                },

                '{' => {
                    output = Token::Lbrace;
                },

                '}' => {
                    output = Token::Rbrace;
                },

                '[' => {
                    output = Token::Lbrack;  
                },

                ']' => {
                    output = Token::Rbrack;
                },

                ',' => {
                    output = Token::Comma;
                },

                ';' => {
                    output = Token::Semicolon;
                },

                ':' => {
                    output = Token::Colon;
                },

                '!' => {
                    self.next_char();
                    if self.ch() == '=' {
                        output = Token::Neq;
                    } else {
                        return Token::Negate;
                    }
                },

                '<' => {
                    self.next_char();

                    if self.ch() == '=' {
                        output = Token::Lte;
                    } else if self.ch() == '-' {
                        output = Token::ArrowLeft;
                    } else if self.ch() == '>' {
                        output = Token::Concat;
                    } else {
                        return Token::Lt;
                    }
                },

                '+' => {
                    output = Token::Add;
                },

                '-' => {
                    self.next_char();

                    if self.ch() == '>' {
                        output = Token::ArrowRight;
                    } else {
                        return Token::Sub;
                    }
                },

                '*' => {
                    self.next_char();

                    if self.ch() == '*' {
                        output = Token::Pow;
                    } else {
                        return Token::Mul;
                    }
                },

                '/' => {
                    output = Token::Div;
                },

                '%' => {
                    output = Token::Mod;
                },

                '=' => {
                    self.next_char();

                    if self.ch() == '=' {
                        output = Token::Eq;
                    } else {
                        return Token::Assign;
                    }
                },

                '>' => {
                    self.next_char();

                    if self.ch() == '=' {
                        output = Token::Gte;
                    } else {
                        return Token::Gt;
                    }
                },

                _ => {
                    report_err(RickError::IllegalCharacter(self.ch() as u8));
                },
            }

            self.next_char();
        }


        output
    }

    fn scan_string(&mut self) -> Token {
        unimplemented!()
    }

    fn scan_number(&mut self) -> Token {
        unimplemented!()
    }

    fn scan_word(&mut self) -> Token {
        unimplemented!()
    }

    fn next_char(&mut self) {
        if !self.is_eof() {
            self.last_read = self.ch();
            self.idx += 1;

            if self.last_read == '\n' {
                error::inc_line();
            } else {
                error::inc_col();
            }
        }
    }

    #[inline(always)]
    fn ch(&self) -> char {
        self.source[self.idx]
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.ch().is_ascii_whitespace() {
            self.next_char();
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.source.len()
    }
}