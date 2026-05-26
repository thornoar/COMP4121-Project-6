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

    pub fn define(&mut self, name: Symbol, value: Value) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }

    pub fn define_many(&mut self, iter: impl IntoIterator<Item = (Symbol, Value)>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.extend(iter);
        }
    }

    pub fn lookup(&self, name: &Symbol) -> Option<Value> {
        self.scopes
            .iter()
            .rfold(None, |acc, map| acc.or(map.get(name).cloned()))
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}
