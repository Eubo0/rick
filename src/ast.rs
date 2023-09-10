use crate::token::*;
use crate::value::Value;

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
        // branches.0 = conditions, branches.1 = statement
        branches: Vec<(Box<ASTNode>, Box<ASTNode>)>,

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
        offset: u32,
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
    BinaryOp {
        lhs: Box<ASTNode>,
        op: Token,
        rhs: Box<ASTNode>,
    },
    GetVar {
        name: String,
        offset: u32,
    },
    GetIndex {
        offset: u32,
        idx: Box<ASTNode>, // expr node
    },
    SetIndex {
        offset: u32,
        idx: Box<ASTNode>,
    },
    Value {
        val: Value,
    }
}