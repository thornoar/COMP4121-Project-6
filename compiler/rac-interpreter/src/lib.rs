use rac_ast::{Expr, Symbol, SymbolicProgram};
use crate::{environ::Environment, value::Value};

mod environ;
mod value;

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

        Expr::Variable(name, _) => match env.lookup(name) {
            Some(value) => value,
            None => panic!()
        },

        Expr::Plus(lhs, rhs) => interpret(lhs, env) + interpret(rhs, env),
        Expr::Minus(lhs, rhs) => interpret(lhs, env) - interpret(rhs, env),
        Expr::Times(lhs, rhs) => interpret(lhs, env) * interpret(rhs, env),
        Expr::Div(lhs, rhs) => interpret(lhs, env) / interpret(rhs, env),
        Expr::Mod(lhs, rhs) => interpret(lhs, env) % interpret(rhs, env),
        Expr::LessEquals(lhs, rhs) => Value::Bool(interpret(lhs, env) <= interpret(rhs, env)),
        Expr::LessThan(lhs, rhs) => Value::Bool(interpret(lhs, env) < interpret(rhs, env)),
        Expr::And(lhs, rhs) => {
            if let Value::Bool(false) = interpret(lhs, env) {
                Value::Bool(false)
            } else {
                interpret(rhs, env)
            }
        }
        Expr::Or(lhs, rhs) => {
            if let Value::Bool(true) = interpret(lhs, env) {
                Value::Bool(true)
            } else {
                interpret(rhs, env)
            }
        }
        Expr::Equals(lhs, rhs) => Value::Bool(interpret(lhs, env) == interpret(rhs, env)),
        Expr::Concat(lhs, rhs) => {
            let Value::String(s1) = interpret(lhs, env) else {
                panic!()
            };
            let Value::String(s2) = interpret(rhs, env) else {
                panic!()
            };

            Value::String(s1 + s2.as_str())
        }

        Expr::Not(e, _) => {
            let Value::Bool(b) = interpret(e, env) else {
                panic!()
            };

            Value::Bool(!b)
        }
        Expr::Neg(e, _) => {
            let Value::Int(b) = interpret(e, env) else {
                panic!()
            };

            Value::Int(-b)
        }

        Expr::Call(name, args, _) => todo!(),

        Expr::Sequence(discard, ret) => {
            interpret(discard, env);
            interpret(ret, env)
        }
        Expr::Let(name, _, value, body, _) => {
            let value = interpret(value, env);
            env.define(name.clone(), value);

            interpret(body, env)
        },
        Expr::Ite(cond, then, elze, _) => {
            let Value::Bool(condval) = interpret(cond, env) else {
                panic!()
            };

            if condval {
                interpret(then, env)
            } else {
                interpret(elze, env)
            }
        }

        Expr::Match(scrut, cases, _) => todo!(),

        Expr::Error(msg, _) => {
            let Value::String(str) = interpret(msg, env) else {
                panic!()
            };

            panic!("Error: {str}")
        },

        _ => todo!()
    }
}
