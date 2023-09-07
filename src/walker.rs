use std::collections::HashMap;

use crate::ast::*;
use crate::value::*;

#[derive(Debug, Clone)]
pub struct Walker {
    top_level: HashMap<String, Box<ASTNode>>,

    val_stack: Vec<Value>,

    local_variables: Vec<Vec<Value>>,
}

impl Walker {
    pub fn new(args: Vec<String>, top_level: ASTNode) -> Walker {
        let converted_args: Vec<Value>;
        converted_args = args.iter().map(|v| Value::String(v.clone())).collect();

        let mut symboltable: HashMap<String, Box<ASTNode>> = HashMap::new();

        if let ASTNode::Toplevel{ funcdefs } = top_level {
            for fdef in funcdefs {
                if let ASTNode::Funcdef {name, params, ret_type, body} = *fdef {
                    symboltable.insert(name, body);
                } else {
                    panic!("Funcdef node was in-fact not a funcdef node :(");
                }
            }
        } else {
            panic!("Top-level node was in-fact not a top level node :(");
        }

        Walker {
            top_level: symboltable,
            val_stack: vec![],
            local_variables: vec![vec![Value::Array(converted_args)]],
        }
    }

    // since std::process::exit expects an i32
    pub fn walk(&mut self) -> i32 {
        0
    }
}