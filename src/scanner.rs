use std::collections::HashMap;

use crate::{error, *};

use crate::token::*;

#[derive(Debug)]
pub struct Scanner {
    source: Vec<char>,
    idx: usize,

    last_read: char,

    reserved_words: HashMap<String, Token>,
}

impl Scanner {
    pub fn new(filename: String) -> Scanner {
        let source: Vec<char> = std::fs::read_to_string(filename).unwrap()
                                                                 .chars()
                                                                 .collect();

        // XXX: There are probably better ways to do this :|
        let reserved_words: HashMap<String, Token> = HashMap::from([
            ("array".into(), Token::Array),
            ("and".into(), Token::And),
            ("boolean".into(), Token::Boolean),
            ("elif".into(), Token::Elif),
            ("else".into(), Token::Else),
            ("false".into(), Token::False),
            ("or".into(), Token::Or),
            ("func".into(), Token::Func),
            ("true".into(), Token::True),
            ("let".into(), Token::Let),
            ("if".into(), Token::If),
            ("while".into(), Token::While),
            ("return".into(), Token::Return),
            ("read".into(), Token::Read),
            ("print".into(), Token::Print),
            ("integer".into(), Token::Integer),
            ("float".into(), Token::Float),
            ("string".into(), Token::String),
            ("var".into(), Token::Var),
        ]);

        Scanner {
            source,
            idx: 0,
            reserved_words,
            last_read: '\0',
        }
    }

    pub fn scan_source(&mut self) -> Vec<(Token, (u32, u32))> {
        let mut tok_stream: Vec<(Token, (u32, u32))> = vec![];
        let mut tok: (Token, (u32, u32)) = self.get_token();

        while tok.0 != Token::Eof {
            tok_stream.push(tok);
            tok = self.get_token();
        }
        tok_stream.push(tok);
        
        tok_stream
    }

    fn get_token(&mut self) -> (Token, (u32, u32)) {
        let mut output: Token = Token::Eof;

        self.skip_whitespace();

        error::save_loc();
        
        if self.is_eof() {
            return (output, get_loc());
        }

        if matches!(self.ch(), 'A'..='Z' | 'a'..='z' | '_') {
            return (self.scan_word(),  get_loc());

        } else if matches!(self.ch(), '0'..='9') {
            return (self.scan_number(), get_loc());

        } else if self.ch() == '"' {
            return (self.scan_string(), get_loc());

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
                        return (Token::Negate, get_loc());
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
                        return (Token::Lt, get_loc());
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
                        return (Token::Sub, get_loc());
                    }
                },

                '*' => {
                    self.next_char();

                    if self.ch() == '*' {
                        output = Token::Pow;
                    } else {
                        return (Token::Mul, get_loc());
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
                        return (Token::Assign, get_loc());
                    }
                },

                '>' => {
                    self.next_char();

                    if self.ch() == '=' {
                        output = Token::Gte;
                    } else {
                        return (Token::Gt, get_loc());
                    }
                },

                _ => {
                    report_err(RickError::IllegalCharacter(self.ch() as u8));
                },
            }

            self.next_char();
        }


        (output, get_loc())
    }

    fn scan_string(&mut self) -> Token {
        let mut string: String = String::new();

        self.next_char();

        while !self.is_eof() && self.ch() != '"' {
            if self.ch() == '\\' {
                self.next_char();

                if !matches!(self.ch(), '\\' | 't' | 'n' | '"') {
                    set_col(*COLUMN_NUM.lock().unwrap());
                    report_err(RickError::IllegalEscapeCode(self.ch()));
                } else {
                    match self.ch() {
                        'n' => string.push('\n'),
                        '\\' => string.push('\\'),
                        't' => string.push('\t'),
                        '"' => string.push('"'),
                        _ => panic!("Unreachable"),
                    }
                }
            } else {
                string.push(self.ch());
            }

            self.next_char();
        }

        if self.is_eof() {
            report_err(RickError::UnclosedString);
        } else {
            self.next_char();
        }

        Token::StringLiteral(string)
    }

    fn scan_number(&mut self) -> Token {
        let mut digits: String = String::new();
        let mut is_float: bool = false;

        while !self.is_eof() && self.ch().is_ascii_digit() {
            digits.push(self.ch());
            self.next_char();
        }

        if !self.is_eof() && self.ch() == '.' {
            digits.push('.');
            self.next_char();
            is_float = true;

            while !self.is_eof() && self.ch().is_ascii_digit() {
                digits.push(self.ch());
                self.next_char();
            }
        }

        if is_float {
            let res = digits.parse::<f32>();
            if res.is_ok() {
                return Token::FloatLiteral(res.unwrap());
            } else {
                report_err(RickError::NumberParseFailure);
            }
        } else {
            let res = digits.parse::<i32>();
            if res.is_ok() {
                return Token::IntegerLiteral(res.unwrap());
            } else {
                report_err(RickError::NumberParseFailure);
            }
        }
    }

    fn scan_word(&mut self) -> Token {
        let mut word: String = String::new();

        while !self.is_eof() && (self.ch().is_ascii_alphanumeric() || self.ch() == '_') {
            word.push(self.ch());
            self.next_char();            
        }

        let res = self.reserved_words.get(&word);

        if res.is_some() {
            res.unwrap().clone()
        } else {
            Token::Identifier(word)
        }
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

    fn ch(&self) -> char {
        if !self.is_eof() {
            self.source[self.idx]
        } else {
            self.last_read
        } 
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.ch().is_ascii_whitespace() {
            self.next_char();
        }
    }

    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.idx >= self.source.len()
    }
}

fn is_printable(ch: char) -> bool {
    matches!(ch as u8, 0x20..=0x7e)
}