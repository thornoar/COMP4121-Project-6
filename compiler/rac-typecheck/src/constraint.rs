use std::fmt::Display;

use rac_ast::{Symbol, Type};


pub struct Constraint {
    pub lhs: Type<Symbol>,
    pub rhs: Type<Symbol>
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.rhs, self.rhs)
    }
}
