use std::collections::HashMap;

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    CaseClassValue(Vec<Box<Value>>),
    Int(i32),
    String(String),
    Unit,
}

#[derive(Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }
}
