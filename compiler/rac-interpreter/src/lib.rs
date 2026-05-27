use rac_ast::{range, Expr, Pattern, Symbol, SymbolicProgram, SymbolicType};
use rac_diagnostics::{Report, Stage};

use crate::{environ::Environment, value::Value};

mod environ;
mod value;

pub fn interpret_program(program: SymbolicProgram) -> Result<(), Report> {
    let env = Environment::new();
    for expr in &program.exprs {
        interpret(expr, &mut env.clone(), &program)?;
    }

    Ok(())
}

pub fn interpret(expr: &Expr<Symbol, SymbolicType>, env: &mut Environment, prog: &SymbolicProgram) -> Result<Value, Report> {
    match expr {
        Expr::BoolLiteral(b, _) => Ok(Value::Bool(*b)),
        Expr::IntLiteral(i, _) => Ok(Value::Int(*i)),
        Expr::StringLiteral(s, _) => Ok(Value::String(s.clone())),
        Expr::UnitLiteral(_) => Ok(Value::Unit),

        Expr::Variable(name, span) => env.lookup(name).ok_or(Report {
            stage: Stage::Interpreting,
            range: *span,
            msg: format!("undefined variable {name}"),
        }),

        Expr::Plus(lhs, rhs) => Ok(interpret(lhs, env, prog)? + interpret(rhs, env, prog)?),
        Expr::Minus(lhs, rhs) => Ok(interpret(lhs, env, prog)? - interpret(rhs, env, prog)?),
        Expr::Times(lhs, rhs) => Ok(interpret(lhs, env, prog)? * interpret(rhs, env, prog)?),
        Expr::Div(lhs, rhs) => Ok(interpret(lhs, env, prog)? / interpret(rhs, env, prog)?),
        Expr::Mod(lhs, rhs) => Ok(interpret(lhs, env, prog)? % interpret(rhs, env, prog)?),
        Expr::LessEquals(lhs, rhs) => {
            Ok(Value::Bool(interpret(lhs, env, prog)? <= interpret(rhs, env, prog)?))
        }
        Expr::LessThan(lhs, rhs) => {
            Ok(Value::Bool(interpret(lhs, env, prog)? < interpret(rhs, env, prog)?))
        }
        Expr::And(lhs, rhs) => {
            if let Value::Bool(false) = interpret(lhs, env, prog)? {
                Ok(Value::Bool(false))
            } else {
                interpret(rhs, env, prog)
            }
        }
        Expr::Or(lhs, rhs) => {
            if let Value::Bool(true) = interpret(lhs, env, prog)? {
                Ok(Value::Bool(true))
            } else {
                interpret(rhs, env, prog)
            }
        }
        Expr::Equals(lhs, rhs) => {
            Ok(Value::Bool(interpret(lhs, env, prog)? == interpret(rhs, env, prog)?))
        }
        Expr::Concat(lhs, rhs) => {
            let Value::String(s1) = interpret(lhs, env, prog)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(lhs),
                    msg: format!("expected string, found `{}`", lhs.show(0))
                });
            };
            let Value::String(s2) = interpret(rhs, env, prog)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(rhs),
                    msg: format!("expected string, found `{}`", lhs.show(0))
                });
            };

            Ok(Value::String(s1 + s2.as_str()))
        }

        Expr::Not(e, _) => {
            let Value::Bool(b) = interpret(e, env, prog)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(e),
                    msg: format!("expected boolean, found `{}`", e.show(0))
                });
            };

            Ok(Value::Bool(!b))
        }
        Expr::Neg(e, _) => {
            let Value::Int(i) = interpret(e, env, prog)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(e),
                    msg: format!("expected integer, found `{}`", e.show(0))
                });
            };

            Ok(Value::Int(-i))
        }

        Expr::Call(name, args, span) => {
            if let Some(def) = prog.fun_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret(e, env, prog))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != def.args.len() {
                    return Err(
                        Report {
                            stage: Stage::Interpreting,
                            range: *span,
                            msg: format!("expected {} arguments, found {}", def.args.len(), values.len())
                        }
                    );
                }

                let map = def
                    .args
                    .iter()
                    .zip(values)
                    .map(|((sym, _, _), val)| (sym.clone(), val));
                env.push_scope();
                env.define_many(map);
                let ret = interpret(&def.body, env, prog);
                env.pop_scope();

                ret
            } else if let Some(def) = prog.class_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret(e, env, prog))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != def.args.len() {
                    return Err(
                        Report {
                            stage: Stage::Interpreting,
                            range: *span,
                            msg: format!("expected {} arguments, found {}", def.args.len(), values.len())
                        }
                    );
                }

                Ok(Value::CaseClassValue(def.name.clone(), values.into_iter().map(Box::new).collect()))
            } else {
                todo!("unresolved call")
            }
        }

        Expr::Sequence(discard, ret) => {
            interpret(discard, env, prog)?;
            interpret(ret, env, prog)
        }
        Expr::Let(name, _, value, body, _) => {
            let value = interpret(value, env, prog)?;
            env.define(name.clone(), value);

            interpret(body, env, prog)
        }
        Expr::Ite(cond, then, elze, _) => {
            let Value::Bool(condval) = interpret(cond, env, prog)? else {
                return Err(
                    Report {
                        stage: Stage::Interpreting,
                        range: range(cond),
                        msg: format!("expected boolean, found `{}`", cond.show(0))
                    }
                );
            };

            if condval {
                interpret(then, env, prog)
            } else {
                interpret(elze, env, prog)
            }
        }

        Expr::Match(e, cases, _) => {
            let scrutinee = interpret(e, env, prog)?;

            for (pattern, body) in cases {
                if let Some(bindings) = match_and_bind(&scrutinee, pattern) {
                    env.push_scope();
                    env.define_many(bindings);
                    let ret = interpret(body, env, prog);
                    env.pop_scope();

                    return ret;
                }
            }

            Err(Report {
                stage: Stage::Interpreting,
                range: range(e),
                msg: format!("match error: no case pattern matches expression {}", e.show(0))
            })
        }

        Expr::Error(msg, span) => {
            let Value::String(str) = interpret(msg, env, prog)? else {
                return Err(
                    Report {
                        stage: Stage::Interpreting,
                        range: range(msg),
                        msg: format!("expected boolean, found `{}`", msg.show(0))
                    }
                );
            };

            Err(
                Report {
                    stage: Stage::Interpreting,
                    range: *span,
                    msg: format!("error: {str}")
                }
            )
        }
    }
}

fn match_and_bind(scrutinee: &Value, pattern: &Pattern<Symbol>) -> Option<Vec<(Symbol, Value)>> {
    match (scrutinee, pattern) {
        (_, Pattern::Wildcard(_)) => Some(vec![]),
        (value, Pattern::IdPattern(sym, _)) => Some(vec![(sym.clone(), value.clone())]),
        (Value::Bool(b), Pattern::BoolPattern(b2, _)) if b == b2 => Some(vec![]),
        (Value::Int(i), Pattern::IntPattern(i2, _)) if i == i2 => Some(vec![]),
        (Value::String(s), Pattern::StringPattern(s2, _)) if s == s2 => Some(vec![]),
        (Value::Unit, Pattern::UnitPattern(_)) => Some(vec![]),
        (Value::CaseClassValue(n, args), Pattern::ClassPattern(id, arg_patterns, _)) if n == id => {
            args.iter()
                .zip(arg_patterns)
                .map(|(scrut, pat)| match_and_bind(scrut, pat))
                .fold(Some(vec![]), |acc, opt| {
                    if let (Some(v1), Some(v2)) = (acc, opt) {
                        Some([v1.as_slice(), v2.as_slice()].concat())
                    } else {
                        None
                    }
                })
        }
        _ => None,
    }
}
