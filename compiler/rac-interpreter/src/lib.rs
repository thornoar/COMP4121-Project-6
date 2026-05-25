mod environ;

use rac_ast::{Expr, Symbol, SymbolicProgram};
use crate::environ::{Environment, Value};

pub fn interpret_program(program: SymbolicProgram) {
    let env = Environment::new();
    program.exprs.iter().for_each(|expr| { interpret(expr, &mut env.clone()); })
}

pub fn interpret(expr: &Expr<Symbol>, env: &mut Environment) -> Value {
    match expr {
        Expr::BoolLiteral(b, _) => Value::Bool(*b),
        Expr::IntLiteral(i, _) => Value::Int(*i),
        Expr::StringLiteral(s, _) => Value::String(s.clone()),
        Expr::UnitLiteral(_) => Value::Unit,

        Expr::Variable(name, _) if let Some(value) = env.lookup(name) => value,

        Expr::Plus(lhs, rhs) => interpret(lhs, env) + interpret(rhs, env),
        Expr::Minus(lhs, rhs) => interpret(lhs, env) - interpret(rhs, env),
        Expr::Times(lhs, rhs) => interpret(lhs, env) * interpret(rhs, env),
        Expr::Div(lhs, rhs) => interpret(lhs, env) / interpret(rhs, env),
        Expr::Mod(lhs, rhs) => interpret(lhs, env) % interpret(rhs, env),

        _ => todo!()
    }
}
