

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),

    // Literals start

                            // Boolean literals handled by reserved words
    StringLiteral(String),  // literal string e.g. "hello world"
    IntegerLiteral(i32),    // literal int e.g. 42                            
    FloatLiteral(f32),      // literal float e.g. 3.1415
    
    // Literals end

    // Reserved words start

    And,            // "and"
    Or,             // "or"
    Func,           // "func"
    True,           // "true"
    False,          // "false"
    Let,            // "let"
    If,             // "if"
    Elif,           // "elif"
    Else,           // "else"
    While,          // "while"
    Return,         // "return"
    Read,           // "read"
    Print,          // "print"
    Array,          // "array"
    Integer,        // "int"
    Float,          // "float"
    Boolean,        // "bool"
    
    // Reserved words end

    Lpar,           // ' ( '
    Rpar,           // ' ) '
    Lbrace,         // ' { '
    Rbrace,         // ' } '
    Lbrack,         // ' [ '
    Rbrack,         // ' ] '
    
    Comma,          // ' , '
    Semicolon,      // ' ; '
    Colon,          // ' : '
    Quote,          // ' " '
    Negate,         // ' ! ' 
    
    ArrowLeft,      // ' <- '
    ArrowRight,     // ' -> '
    Concat,         // ' <> ' e.g. print("The answer is " <> answer <> "\n")

    Add,            // ' + '
    Sub,            // ' - '
    Mul,            // ' * '
    Div,            // ' / '
    Mod,            // ' % '
    Pow,            // ' ** '

    Assign,         // ' = '

    Eq,             // ' == '
    Neq,            // ' != '
    Gt,             // ' > '
    Gte,            // ' >= '
    Lt,             // ' < '
    Lte,            // ' <= '
}

impl Token {
    pub fn is_relational_op(&self) -> bool {
        matches!(self, Token::Eq | Token::Neq | Token::Gt 
            | Token::Gte | Token::Lt | Token::Lte)
    }

    pub fn is_ordering_op(&self) -> bool {
        matches!(self, Token::Gt | Token::Gte | Token::Lt | Token::Lte)
    }

    pub fn is_additive_op(&self) -> bool {
        matches!(self, Token::Add | Token::Sub | Token::Or)
    }

    pub fn is_multiplicative_op(&self) -> bool {
        matches!(self, Token::Mul | Token::Div | Token::And)
    }

    pub fn is_exponent_op(&self) -> bool {
        matches!(self, Token::Pow)
    }

    pub fn is_type_start(&self) -> bool {
        matches!(self, Token::Integer | Token::Float | Token::Boolean)
    }

    pub fn starts_factor(&self) -> bool {
        matches!(self, Token::Identifier(_) | Token::IntegerLiteral(_) 
                | Token::FloatLiteral(_) | Token::True | Token::False
                | Token::Lpar | Token::Negate)
    }

    pub fn start_expression(&self) -> bool {
        if self.starts_factor() || matches!(self, Token::Sub) {
            true
        } else {
            false
        }
    }
}