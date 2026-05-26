use rac_ast::{Expr, Symbol, SymbolicProgram};

use crate::{environ::Environment, value::Value};

mod environ;
mod value;

pub fn interpret_program(program: SymbolicProgram) {
    let env = Environment::new();
    program.exprs.iter().for_each(|expr| {
        interpret(expr, &mut env.clone(), &program);
    })
}

pub fn interpret(expr: &Expr<Symbol>, env: &mut Environment, prog: &SymbolicProgram) -> Value {
    match expr {
        Expr::BoolLiteral(b, _) => Value::Bool(*b),
        Expr::IntLiteral(i, _) => Value::Int(*i),
        Expr::StringLiteral(s, _) => Value::String(s.clone()),
        Expr::UnitLiteral(_) => Value::Unit,

        Expr::Variable(name, _) => match env.lookup(name) {
            Some(value) => value,
            None => panic!(),
        },

        Expr::Plus(lhs, rhs) => interpret(lhs, env, prog) + interpret(rhs, env, prog),
        Expr::Minus(lhs, rhs) => interpret(lhs, env, prog) - interpret(rhs, env, prog),
        Expr::Times(lhs, rhs) => interpret(lhs, env, prog) * interpret(rhs, env, prog),
        Expr::Div(lhs, rhs) => interpret(lhs, env, prog) / interpret(rhs, env, prog),
        Expr::Mod(lhs, rhs) => interpret(lhs, env, prog) % interpret(rhs, env, prog),
        Expr::LessEquals(lhs, rhs) => {
            Value::Bool(interpret(lhs, env, prog) <= interpret(rhs, env, prog))
        }
        Expr::LessThan(lhs, rhs) => {
            Value::Bool(interpret(lhs, env, prog) < interpret(rhs, env, prog))
        }
        Expr::And(lhs, rhs) => {
            if let Value::Bool(false) = interpret(lhs, env, prog) {
                Value::Bool(false)
            } else {
                interpret(rhs, env, prog)
            }
        }
        Expr::Or(lhs, rhs) => {
            if let Value::Bool(true) = interpret(lhs, env, prog) {
                Value::Bool(true)
            } else {
                interpret(rhs, env, prog)
            }
        }
        Expr::Equals(lhs, rhs) => {
            Value::Bool(interpret(lhs, env, prog) == interpret(rhs, env, prog))
        }
        Expr::Concat(lhs, rhs) => {
            let Value::String(s1) = interpret(lhs, env, prog) else {
                panic!()
            };
            let Value::String(s2) = interpret(rhs, env, prog) else {
                panic!()
            };

            Value::String(s1 + s2.as_str())
        }

        Expr::Not(e, _) => {
            let Value::Bool(b) = interpret(e, env, prog) else {
                panic!()
            };

            Value::Bool(!b)
        }
        Expr::Neg(e, _) => {
            let Value::Int(b) = interpret(e, env, prog) else {
                panic!()
            };

            Value::Int(-b)
        }

        Expr::Call(name, args, _) => {
            if let Some((arglist, _, body)) = prog.fun_defs.get(&name.id) {
                let values = args.iter().map(|e| interpret(e, env, prog)).collect::<Vec<_>>();
                if values.len() != arglist.len() {
                    panic!("mismatched arity of function call");
                }

                let map = arglist.iter().zip(values).map(|((sym, _), val)| (sym.clone(), val));
                env.push_scope();
                env.define_many(map);
                let ret = interpret(body, env, prog);
                env.pop_scope();

                ret
            } else if let Some((arglist, id)) = prog.class_defs.get(&name.id) {
                let values = args.iter().map(|e| Box::new(interpret(e, env, prog))).collect::<Vec<_>>();
                if values.len() != arglist.len() {
                    panic!("mismatched arity of constructor call");
                }

                Value::CaseClassValue(id.clone(), values)
            } else {
                panic!("unresolved call")
            }
        }

        Expr::Sequence(discard, ret) => {
            interpret(discard, env, prog);
            interpret(ret, env, prog)
        }
        Expr::Let(name, _, value, body, _) => {
            let value = interpret(value, env, prog);
            env.define(name.clone(), value);

            interpret(body, env, prog)
        }
        Expr::Ite(cond, then, elze, _) => {
            let Value::Bool(condval) = interpret(cond, env, prog) else {
                panic!()
            };

            if condval {
                interpret(then, env, prog)
            } else {
                interpret(elze, env, prog)
            }
        }

        Expr::Match(scrut, cases, _) => todo!(),

        Expr::Error(msg, _) => {
            let Value::String(str) = interpret(msg, env, prog) else {
                panic!()
            };

            panic!("Error: {str}")
        }

        _ => todo!(),
    }
}
