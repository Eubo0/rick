use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Array(Vec<Value>),
    None,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => {
                *b
            },
            Value::Integer(i) => {
                *i != 0
            },
            Value::Float(f) => {
                *f != 0.0
            },
            _ => {
                panic!("'{:#?}' should not be checked for whether it is truthy or falsey.", self);
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => {
                write!(f, "{}", s)
            },
            Value::Boolean(b) => {
                write!(f, "{}", b)
            },
            Value::Integer(i) => {
                write!(f, "{}", i)
            },
            Value::Array(arr) => {
                write!(f, "[")?;
                if arr.len() > 0 {
                    write!(f, "{}", arr[0])?;

                    for i in 1..=arr.len() - 1 {
                        write!(f, ", {}", arr[i])?;
                    }
                }
                write!(f, "]")
            },
            _ => {
                println!("This is a fallback & shouldn't happen, consider it an error.");
                write!(f, "{:#?}", self)
            }
        }
    }
}