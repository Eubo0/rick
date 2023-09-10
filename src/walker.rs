use std::collections::HashMap;

use crate::ast::*;
use crate::value::*;
use crate::token::Token;

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
                // For "de-allocating" if there are any vardefs in statements
                let idx: usize = self.local_variables.len();
                let initial_size: usize = self.local_variables[idx - 1].len();

                for statement in statements {
                    if self.visit_node(statement) {
                        self.local_variables[idx - 1].truncate(initial_size);
                        return true;
                    }
                }

                self.local_variables[idx - 1].truncate(initial_size);
            },
            ASTNode::VarDef { tipe: _, names } => {
                let len: usize = self.local_variables.len();
                for _ in names {
                    self.local_variables[len - 1].push(Value::None);
                }
            },
            ASTNode::Call { name, args } => {
                let mut new_scope: Vec<Value> = vec![];

                for arg in args {
                    self.visit_node(arg);
                    
                    new_scope.push(self.val_stack.pop().unwrap());
                }
                self.local_variables.push(new_scope);

                let (_, body) = self.top_level.get(&name).unwrap();
                self.visit_node(body.clone());

                self.local_variables.pop();
            },
            ASTNode::Let { offset, index, is_array, rhs } => {
                self.visit_node(rhs);

                if is_array {
                    let cap: i32 = self.val_stack.pop().unwrap().force_int();
                    let mut arr: Vec<Value> = Vec::with_capacity(cap as usize);
                    for i in 0..(cap as u32) {
                        arr.push(Value::None);
                    }
                    
                    let idx: usize = self.local_variables.len();
                    self.local_variables[idx - 1][offset as usize] = Value::Array(arr);
                } else {
                    if index.is_some() {
                        self.visit_node(index.unwrap());
                        let idx: i32 = self.val_stack.pop().unwrap().force_int();
                        let j: usize = self.local_variables.len();
                        let arr: Value = self.local_variables[j - 1][offset as usize].clone();
                        if let Value::Array(mut inner) = arr {
                            inner[idx as usize] = self.val_stack.pop().unwrap();
                            self.local_variables[j - 1][offset as usize] = Value::Array(inner);
                        } else {
                            panic!("Typechecking fail");
                        }
                    } else {
                        let idx = self.local_variables.len();
                        let val = self.val_stack.pop().unwrap();
                        self.local_variables[idx - 1][offset as usize] = val;
                    }
                }
            },
            ASTNode::If { branches, else_case } => {
                for (cond, body) in branches {
                    self.visit_node(cond);

                    if self.val_stack.pop().unwrap().is_truthy() {
                        if self.visit_node(body) {
                            return true;
                        } else {
                            return false;
                        }
                    }
                }
                if else_case.is_some() {
                    return self.visit_node(else_case.unwrap());
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
            ASTNode::GetIndex { offset, idx } => {
                let len = self.local_variables.len();
                let arr = self.local_variables[len - 1][offset as usize].clone();

                self.visit_node(idx);

                let true_idx: usize = self.val_stack.pop().unwrap().force_int() as usize;
                
                if let Value::Array(inner) = arr {
                    self.val_stack.push(inner[true_idx].clone());
                } else {
                    panic!("Somehow indexed a non-array");
                }
            },
            // TODO: BinaryOp looks messy
            ASTNode::BinaryOp { lhs, op, rhs } => {
                self.visit_node(lhs);
                self.visit_node(rhs);
                
                let rval = self.val_stack.pop().unwrap();
                let lval = self.val_stack.pop().unwrap();

                match op {
                    Token::Add => {
                        self.val_stack.push(lval + rval);
                    },
                    Token::Sub => {
                        self.val_stack.push(lval - rval);
                    },
                    Token::Mul => {
                        self.val_stack.push(lval * rval);
                    },
                    Token::Div => {
                        self.val_stack.push(lval / rval);
                    },
                    Token::Mod => {
                        self.val_stack.push(lval % rval);
                    },
                    Token::Gt => {
                        self.val_stack.push(Value::Boolean(lval.is_gt(&rval)));
                    },
                    Token::Gte => {
                        self.val_stack.push(Value::Boolean(lval.is_gte(&rval)));
                    },
                    Token::Lt => {
                        self.val_stack.push(Value::Boolean(lval.is_lt(&rval)));
                    },
                    Token::Lte => {
                        self.val_stack.push(Value::Boolean(lval.is_lte(&rval)));
                    },
                    Token::Eq => {
                        self.val_stack.push(Value::Boolean(lval.is_eq(&rval)));
                    },
                    Token::Neq => {
                        self.val_stack.push(Value::Boolean(lval.is_neq(&rval)));
                    },
                    Token::Pow => {
                        self.val_stack.push(lval.pow_value(&rval));
                    },
                    _ => {
                        unimplemented!("Binary operator '{}'", op);
                    }
                }
            },
            ASTNode::UnaryOp { op, value } => {
                self.visit_node(value);
                let val = self.val_stack.pop().unwrap();

                match &op {
                    Token::Sub => {
                        self.val_stack.push(-val);
                    },
                    _ => panic!("Not a unary op '{}'", op),
                }
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