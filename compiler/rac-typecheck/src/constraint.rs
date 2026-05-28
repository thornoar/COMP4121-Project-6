use std::fmt::Display;

use rac_ast::SymbolicType;
use rac_diagnostics::Span;

pub struct Constraint {
    pub expected: SymbolicType,
    pub found: SymbolicType,
    pub range: Span,
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.expected, self.found)
    }
}

impl Constraint {
    pub fn new(expected: SymbolicType, found: SymbolicType, range: Span) -> Self {
        Self {
            expected,
            found,
            range,
        }
    }
}
