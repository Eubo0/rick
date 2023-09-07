

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Array(Vec<Value>),
}