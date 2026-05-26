use std::collections::{HashMap, VecDeque};

use rac_ast::{Expr, Symbol, Type};
use rac_diagnostics::{Report, Stage};

use crate::constraint::Constraint;

pub mod constraint;

macro_rules! single {
    ($elt:expr) => {{
        let mut res = VecDeque::new();
        res.push_back($elt);
        res
    }};
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        Err(Report {
            stage: Stage::Typechecking,
            range: $span,
            msg: String::from($msg),
        })
    };
}

fn collect_constraints(e: Expr<Symbol>, expected: Type<Symbol>, env: HashMap<Symbol, Type<Symbol>>) -> Result<VecDeque<Constraint>, Report> {
    use Expr::*;
    match e {
        Variable(name, s) => match env.get(&name) {
            Some(typ) => Ok(single!(Constraint::new(expected, typ.clone(), s))),
            None => error!(s, "Variable not present in the environment.")
        }
        // IntLiteral(val, s) => Ok(single!(Constraint::new(expected, Type::IntType(), s)))
        _ => todo!()
    }
}
