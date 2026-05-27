use std::fmt::Display;

use rac_ast::SymbolicType;
use rac_diagnostics::Span;


pub struct Constraint {
    pub lhs: SymbolicType,
    pub rhs: SymbolicType,
    pub range: Span
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.rhs, self.rhs)
    }
}

impl Constraint {
    pub fn new(lhs: SymbolicType, rhs: SymbolicType, range: Span) -> Self {
        Self { lhs, rhs, range }
    }
}
