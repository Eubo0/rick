use crate::token::*;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Toplevel {
        funcdefs: Vec<Box<ASTNode>>
    },
    Funcdef {
        name: String,
        params: Vec<(String, u8)>,
        ret_type: u8,

        body: Box<ASTNode>
    },
    Block {
        statements: Vec<Box<ASTNode>>,
    },
    If {
        first_condition: Box<ASTNode>,
        first_statement: Box<ASTNode>,

        // branches.0 = conditions, branches.1 = statement
        elif_branches: Vec<(Box<ASTNode>, Box<ASTNode>)>,

        else_case: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,

        statement: Box<ASTNode>,
    },
    VarDef {
        tipe: u8,
        names: Vec<String>,
    },
    Call {
        name: String,
        args: Vec<Box<ASTNode>>,
    },
    Let {
        name: String,
        index: Option<Box<ASTNode>>,

        // does 'array' preceed the assignment expression
        is_array: bool,
        rhs: Box<ASTNode>,
    },
    Read {
        name: String,
        maybe_index: Option<Box<ASTNode>>
    },
    Print {
        items: Vec<Box<ASTNode>>,
    },
    Return {
        expr: Option<Box<ASTNode>>,
    },
    UnaryOp {
        op: Token,
        value: Box<ASTNode>,
    },
    BinaryOP {
        lhs: Box<ASTNode>,
        op: Token,
        rhs: Box<ASTNode>,
    },
    GetVar {
        name: String,
        offset: u32,
    },
    IndexArray {
        name: String,
        offset: u32,
        idx: Box<ASTNode>,
    },
    Value {
        val: Value,
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i32),
    Float(f32),
}