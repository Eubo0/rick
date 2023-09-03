use std::fmt;

pub const NONE: usize = 0;
pub const BOOLEAN: usize = 1;
pub const INTEGER: usize = 2;
pub const FLOAT: usize = 3;
pub const STRING: usize = 4;
pub const ARRAY: usize = 5;
pub const FUNC: usize = 6;

// XXX: this looks useful
// type tipe = [bool; 7];
#[derive(Debug, Clone)]
pub struct Properties {
    pub tipe: [bool; 7],

    pub params: Vec<(String, [bool; 7])>,
}

impl fmt::Display for Properties {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {

        if self.tipe[FUNC] {
            // callable
            if self.tipe[NONE] {
                write!(f, "procedure")
            } else {
                if self.tipe[BOOLEAN] {
                    write!(f, "boolean ")?;
                } else if self.tipe[INTEGER] {
                    write!(f, "integer ")?;
                } else if self.tipe[FLOAT] {
                    write!(f, "float ")?;
                } else if self.tipe[STRING] {
                    write!(f, "string ")?;
                }

                if self.tipe[ARRAY] {
                    write!(f, "array ")?;
                }

                write!(f, "function")
            }

        } else {
            if self.tipe[BOOLEAN] {
                write!(f, "boolean")?;
            } else if self.tipe[INTEGER] {
                write!(f, "integer")?;
            } else if self.tipe[FLOAT] {
                write!(f, "float")?;
            } else if self.tipe[STRING] {
                write!(f, "string")?;
            }

            if self.tipe[ARRAY] {
                write!(f, " array")?;
            }

            Ok(())
        }
    }
}