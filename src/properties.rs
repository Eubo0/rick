use std::fmt;

pub const NONE: u8 = 0;
pub const BOOLEAN: u8 = 1;
pub const INTEGER: u8 = 2;
pub const FLOAT: u8 = 4;
pub const STRING: u8 = 8;
pub const ARRAY: u8 = 16;
pub const FUNC: u8 = 32;

/*
    0 = NONE,
    1 = BOOLEAN,
    2 = INTEGER,
    4 = FLOAT,
    8 = STRING,
    16 = ARRAY,
    32 = FUNC, 
*/

// XXX: this looks useful
// type tipe = [bool; 7];
#[derive(Debug, Clone)]
pub struct Properties {
    pub tipe: u8,
    pub offset: Option<u32>,

    pub params: Vec<(String, u8)>,
}

pub fn type_string(t: u8) -> String {
    let mut out: String = String::new();

    if t & FUNC != 0 {
        // callable
        if t & NONE != 0 {
            out.push_str("procedure");
        } else {
            if t & BOOLEAN != 0 {
                out.push_str("boolean ");
            } else if t & INTEGER != 0 {
                out.push_str("integer ");
            } else if t & FLOAT != 0 {
                out.push_str("float ");
            } else if t & STRING != 0 {
                out.push_str("string ");
            }

            if t & ARRAY != 0 {
                out.push_str("array ");
            }

            out.push_str("function");
        }

    } else {
        if t & BOOLEAN != 0 {
            out.push_str("boolean");
        } else if t & INTEGER != 0 {
            out.push_str("integer");
        } else if t & FLOAT != 0 {
            out.push_str("float");
        } else if t & STRING != 0 {
            out.push_str("string");
        }

        if t & ARRAY != 0 {
            out.push_str("array");
        }
    }

    out
}

pub fn is_numeric_type(t: u8) -> bool {
    if t == INTEGER || t == FLOAT {
        true
    } else {
        false
    }
}

impl fmt::Display for Properties {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {

        if self.offset.is_some() {
            write!(f, "@{} ", self.offset.unwrap())?;
        } else {
            write!(f, "@_ ")?;
        }

        if self.tipe & FUNC != 0 {
            // callable
            if self.tipe & NONE != 0 {
                write!(f, "procedure")
            } else {
                if self.tipe & BOOLEAN != 0 {
                    write!(f, "boolean ")?;
                } else if self.tipe & INTEGER != 0 {
                    write!(f, "integer ")?;
                } else if self.tipe & FLOAT != 0 {
                    write!(f, "float ")?;
                } else if self.tipe & STRING != 0 {
                    write!(f, "string ")?;
                }

                if self.tipe & ARRAY != 0 {
                    write!(f, "array ")?;
                }

                write!(f, "function")
            }

        } else {
            if self.tipe & BOOLEAN != 0 {
                write!(f, "boolean")?;
            } else if self.tipe & INTEGER != 0 {
                write!(f, "integer")?;
            } else if self.tipe & FLOAT != 0 {
                write!(f, "float")?;
            } else if self.tipe & STRING != 0 {
                write!(f, "string")?;
            }

            if self.tipe & ARRAY != 0 {
                write!(f, " array")?;
            }

            Ok(())
        }
    }
}