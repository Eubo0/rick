use std::fmt;
use std::cmp::Ordering;

use crate::properties::*;

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

    pub fn is_gt(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 > i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 > f2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn is_gte(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 >= i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 >= f2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn is_lt(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 < i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 < f2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn is_lte(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 <= i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 <= f2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn is_eq(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 == i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 == f2
            },
            (Value::Boolean(b1), Value::Boolean(b2)) => {
                b1 == b2
            },
            (Value::String(s1), Value::String(s2)) => {
                s1 == s2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn is_neq(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                i1 != i2
            },
            (Value::Float(f1), Value::Float(f2)) => {
                f1 != f2
            },
            (Value::Boolean(b1), Value::Boolean(b2)) => {
                b1 != b2
            },
            (Value::String(s1), Value::String(s2)) => {
                s1 != s2
            },
            _ => {
                panic!("Non comparable types '{}' & '{}'", self, rhs);
            }
        }
    }

    pub fn pow_value(&self, exponent: &Value) -> Value {
        match (self, exponent) {
            (Value::Integer(base), Value::Integer(exp)) => {
                Value::Integer(i32::pow(*base, *exp as u32))
            },
            _ => {
                panic!("Power unimplemented!")
            }
        }
    }

    pub fn force_int(&self) -> i32 {
        match self {
            Value::Integer(i) => *i,
            _ => panic!("Invalid index :("),
        }
    }
}

pub fn string_to_val(tipe: u8, string: String) -> Value {
    if tipe & STRING != 0 {
        Value::String(string)
    
    } else if tipe & BOOLEAN != 0 {
        let b: bool = string.parse().unwrap();
        Value::Boolean(b)

    } else if tipe & INTEGER != 0 {
        let i: i32 = string.parse().unwrap();
        Value::Integer(i)

    } else if tipe & FLOAT != 0 {
        let f: f32 = string.parse().unwrap();
        Value::Float(f)

    } else {
        panic!("Not able to coerce input to a specific value");
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
                write!(f, "{:#?}", self)
            }
        }
    }
}

use std::ops::{Add, Sub, Mul, Div, Rem, Not, Neg};

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        match (&self, &rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 + i2)
            },
            (Value::Float(f1), Value::Float(f2)) => {
                Value::Float(f1 + f2)
            },
            (Value::String(s1), Value::String(s2)) => {
                Value::String(format!("{}{}", s1, s2))
            },
            _ => {
                panic!("Addition not allowed for '{:#?}' and {:#?}. Typechecking failed you :(", self, rhs)
            },
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Value {
        match (&self, &rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 - i2)
            },
            (Value::Float(f1), Value::Float(f2)) => {
                Value::Float(f1 - f2)
            },
            _ => {
                panic!("Subtraction not allowed for '{:#?}' and {:#?}. Typechecking failed you :(", self, rhs)
            },
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Value {
        match (&self, &rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 * i2)
            },
            (Value::Float(f1), Value::Float(f2)) => {
                Value::Float(f1 * f2)
            },
            _ => {
                panic!("Multiplication not allowed for '{:#?}' and {:#?}. Typechecking failed you :(", self, rhs)
            },
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Value {
        match (&self, &rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 / i2)
            },
            (Value::Float(f1), Value::Float(f2)) => {
                Value::Float(f1 / f2)
            },
            _ => {
                panic!("Division not allowed for '{:#?}' and {:#?}. Typechecking failed you :(", self, rhs)
            },
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Value) -> Value {
        match (&self, &rhs) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 % i2)
            },
            (Value::Float(f1), Value::Float(f2)) => {
                Value::Float(f1 % f2)
            },
            _ => {
                panic!("Modulo not allowed for '{:#?}' and {:#?}. Typechecking failed you :(", self, rhs)
            },
        }
    }
}

impl Neg for Value{
    type Output = Value;

    fn neg(self) -> Value {
        match &self {
            Value::Integer(i) => Value::Integer(-i),
            Value::Float(f) => Value::Float(-f),
            _ => panic!("Can't negate non-numeric type '{:#?}'", self),
        }
    }
}