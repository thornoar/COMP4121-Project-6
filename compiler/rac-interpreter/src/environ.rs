use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use rac_ast::Symbol;

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    CaseClassValue(Vec<Box<Value>>),
    Int(i32),
    String(String),
    Unit,
}

impl Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            _ => panic!("unsupported operands of add")
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            _ => panic!("unsupported operands of minus")
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            _ => panic!("unsupported operands of minus")
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
            _ => panic!("unsupported operands of minus")
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            _ => panic!("unsupported operands of minus")
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    scopes: Vec<HashMap<Symbol, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn lookup(&self, name: &Symbol) -> Option<Value> {
        self.scopes.iter().rfold(None, |acc, map| {
            acc.or(map.get(name).cloned())
        })
    }
}
