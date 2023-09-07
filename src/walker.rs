use std::collections::HashMap;

use crate::ast::*;
use crate::value::*;

#[derive(Debug, Clone)]
pub struct Walker {
    // TODO: figure out if this is a sustainable way of storing top level nodes.
    // The idea is that each top-level node takes n arguments off the stack.
    top_level: HashMap<String, (u32, Box<ASTNode>)>,

    val_stack: Vec<Value>,

    local_variables: Vec<Vec<Value>>,
}

// XXX:
// I barely escaped the mess of the parser, precious AST in hands,
// just to create another mess. At least it's my mess. That I can deal with.
// If you contribute, you daren't cause more of a mess.
impl Walker {
    pub fn new(args: Vec<String>, top_level: ASTNode) -> Walker {
        let converted_args: Vec<Value>;
        converted_args = args.iter().map(|v| Value::String(v.clone())).collect();

        let mut symboltable: HashMap<String, (u32, Box<ASTNode>)> = HashMap::new();

        if let ASTNode::Toplevel{ funcdefs } = top_level {
            for fdef in funcdefs {
                if let ASTNode::Funcdef {name, params, ret_type: _, body} = *fdef {
                    symboltable.insert(name, (params.len() as u32, body));
                } else {
                    panic!("Funcdef node was in-fact not a funcdef node :(");
                }
            }
        } else {
            panic!("Top-level node was in-fact not a top level node :(");
        }

        Walker {
            top_level: symboltable,
            val_stack: vec![Value::Array(converted_args)],
            local_variables: vec![vec![]],
        }
    }

    // since std::process::exit expects an i32
    pub fn walk(&mut self) -> i32 {
        let start: &(u32, Box<ASTNode>) = self.top_level.get("main").expect("Programs are required to have a main function.");

        let mut arg_slice = self.val_stack.split_off(self.val_stack.len() - (start.0 as usize) );
        let size: usize = self.local_variables.len();
        self.local_variables[size - 1].append(&mut arg_slice);

        self.visit_node(start.1.clone());

        let result = self.val_stack.pop().expect("Empty stack upon exit :(");

        if let Value::Integer(i) = result {
            return i;
        } else {
            panic!("Non-integer type returned upon exit :(");
        }
    }

    fn visit_node(&mut self, node: Box<ASTNode>) -> bool {
        match *node {
            ASTNode::Block { statements } => {
                for statement in statements {
                    if self.visit_node(statement) {
                        return true;
                    }
                }
            },
            ASTNode::VarDef { tipe: _, names } => {
                let len: usize = self.local_variables.len();
                for _ in names {
                    self.local_variables[len - 1].push(Value::None);
                }
            },
            ASTNode::While { condition, statement } => {
                let mut res: bool;

                self.visit_node(condition.clone());
                res = self.val_stack.pop().unwrap().is_truthy();

                while res {
                    if self.visit_node(statement.clone()) {
                        return true;
                    }
                    self.visit_node(condition.clone());
                    res = self.val_stack.pop().unwrap().is_truthy();
                } 
            },
            ASTNode::Return { expr } => {
                if expr.is_some() {
                    self.visit_node(expr.unwrap());
                }
                return true;
            },
            ASTNode::Print { items } => {
                for item in items {
                    self.visit_node(item);
                    print!("{}", self.val_stack.pop().unwrap());
                }
            },
            ASTNode::GetVar { name: _, offset } => {
                let len: usize = self.local_variables.len();
                let val = self.local_variables[len - 1][offset as usize].clone();
                self.val_stack.push(val);
            },
            ASTNode::Value { val } => {
                self.val_stack.push(val);
            },
            _ => {
                unimplemented!("{:#?}", node);
            },
        }

        false
    }
}