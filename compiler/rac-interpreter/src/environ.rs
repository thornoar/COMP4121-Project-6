use std::collections::HashMap;
use rac_ast::Symbol;

use crate::value::Value;

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
