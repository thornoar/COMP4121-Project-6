use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Rem, Sub},
};

use rac_ast::Symbol;

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Bool(bool),
    CaseClassValue(Symbol, Vec<Box<Value>>),
    Int(i32),
    String(String),
    Unit,
}

impl Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            _ => panic!("unsupported operands of add"),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            _ => panic!("unsupported operands of minus"),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            _ => panic!("unsupported operands of minus"),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
            _ => panic!("unsupported operands of minus"),
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            _ => panic!("unsupported operands of minus"),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
