use std::{
    cmp::Ordering,
    fmt::Display,
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value as V;
        match self {
            V::Bool(val) => write!(f, "{val}"),
            V::Int(val) => write!(f, "{val}"),
            V::String(val) => write!(f, "\"{val}\""),
            V::Unit => write!(f, "()"),
            V::CaseClassValue(name, args) => {
                let args_str = args
                    .iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{}({})", name, args_str)
            }
        }
    }
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
