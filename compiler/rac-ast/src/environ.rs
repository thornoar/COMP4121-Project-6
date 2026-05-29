use std::collections::HashMap;

use crate::SID;

#[derive(Debug, Clone)]
pub struct Environment<T> {
    scopes: Vec<HashMap<SID, T>>,
}

impl<T: Clone> Environment<T> {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn define(&mut self, id: SID, value: T) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(id, value);
        }
    }

    pub fn define_many(&mut self, iter: impl IntoIterator<Item = (SID, T)>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.extend(iter);
        }
    }

    pub fn lookup(&self, id: SID) -> Option<T> {
        self.scopes
            .iter()
            .rfold(None, |acc, map| acc.or_else(|| map.get(&id).cloned()))
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
