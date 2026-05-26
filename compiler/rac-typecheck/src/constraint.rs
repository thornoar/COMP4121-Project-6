use std::fmt::Display;

use rac_ast::{Symbol, Type};
use rac_diagnostics::Span;


pub struct Constraint {
    pub lhs: Type<Symbol>,
    pub rhs: Type<Symbol>,
    pub range: Span
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.rhs, self.rhs)
    }
}

impl Constraint {
    pub fn new(lhs: Type<Symbol>, rhs: Type<Symbol>, range: Span) -> Self {
        Self { lhs, rhs, range }
    }
}
