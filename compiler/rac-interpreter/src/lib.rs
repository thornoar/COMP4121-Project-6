use rac_ast::{DefinitionTable, Expr, Pattern, Symbol, SymbolicProgram, SymbolicType, range};
use rac_diagnostics::{Report, Stage};

use crate::{environ::Environment, value::Value};

mod environ;
mod value;

pub fn interpret_program(program: SymbolicProgram) -> Result<(), Report> {
    let env = Environment::new();
    for expr in &program.exprs {
        interpret(expr, &mut env.clone(), &program.table)?;
    }

    Ok(())
}

macro_rules! report {
    ($span:expr, $msg:expr) => {
        Report {
            stage: Stage::Interpreting,
            range: $span,
            msg: $msg,
        }
    };
}

pub fn interpret(
    expr: &Expr<Symbol, SymbolicType>,
    env: &mut Environment,
    table: &DefinitionTable,
) -> Result<Value, Report> {
    match expr {
        Expr::BoolLiteral(b, _) => Ok(Value::Bool(*b)),
        Expr::IntLiteral(i, _) => Ok(Value::Int(*i)),
        Expr::StringLiteral(s, _) => Ok(Value::String(s.clone())),
        Expr::UnitLiteral(_) => Ok(Value::Unit),

        Expr::Variable(name, span) => env
            .lookup(name)
            .ok_or(report!(*span, format!("undefined variable {name}"))),

        Expr::Plus(lhs, rhs) => Ok(interpret(lhs, env, table)? + interpret(rhs, env, table)?),
        Expr::Minus(lhs, rhs) => Ok(interpret(lhs, env, table)? - interpret(rhs, env, table)?),
        Expr::Times(lhs, rhs) => Ok(interpret(lhs, env, table)? * interpret(rhs, env, table)?),
        Expr::Div(lhs, rhs) => Ok(interpret(lhs, env, table)? / interpret(rhs, env, table)?),
        Expr::Mod(lhs, rhs) => Ok(interpret(lhs, env, table)? % interpret(rhs, env, table)?),
        Expr::LessEquals(lhs, rhs) => Ok(Value::Bool(
            interpret(lhs, env, table)? <= interpret(rhs, env, table)?,
        )),
        Expr::LessThan(lhs, rhs) => Ok(Value::Bool(
            interpret(lhs, env, table)? < interpret(rhs, env, table)?,
        )),
        Expr::And(lhs, rhs) => {
            if let Value::Bool(false) = interpret(lhs, env, table)? {
                Ok(Value::Bool(false))
            } else {
                interpret(rhs, env, table)
            }
        }
        Expr::Or(lhs, rhs) => {
            if let Value::Bool(true) = interpret(lhs, env, table)? {
                Ok(Value::Bool(true))
            } else {
                interpret(rhs, env, table)
            }
        }
        Expr::Equals(lhs, rhs) => Ok(Value::Bool(
            interpret(lhs, env, table)? == interpret(rhs, env, table)?,
        )),
        Expr::Concat(lhs, rhs) => {
            let Value::String(s1) = interpret(lhs, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(lhs),
                    msg: format!("expected string, found `{}`", lhs.show(0)),
                });
            };
            let Value::String(s2) = interpret(rhs, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(rhs),
                    msg: format!("expected string, found `{}`", lhs.show(0)),
                });
            };

            Ok(Value::String(s1 + s2.as_str()))
        }

        Expr::Not(e, _) => {
            let Value::Bool(b) = interpret(e, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(e),
                    msg: format!("expected boolean, found `{}`", e.show(0)),
                });
            };

            Ok(Value::Bool(!b))
        }
        Expr::Neg(e, _) => {
            let Value::Int(i) = interpret(e, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(e),
                    msg: format!("expected integer, found `{}`", e.show(0)),
                });
            };

            Ok(Value::Int(-i))
        }

        Expr::Call(name, args, span) => {
            if let Some(def) = table.fun_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret(e, env, table))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != def.args.len() {
                    return Err(Report {
                        stage: Stage::Interpreting,
                        range: *span,
                        msg: format!(
                            "expected {} arguments, found {}",
                            def.args.len(),
                            values.len()
                        ),
                    });
                }

                let map = def
                    .args
                    .iter()
                    .zip(values)
                    .map(|((sym, _), val)| (sym.clone(), val));
                env.push_scope();
                env.define_many(map);
                let ret = interpret(&def.body, env, table);
                env.pop_scope();

                ret
            } else if let Some(def) = table.class_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret(e, env, table))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != def.args.len() {
                    return Err(Report {
                        stage: Stage::Interpreting,
                        range: *span,
                        msg: format!(
                            "expected {} arguments, found {}",
                            def.args.len(),
                            values.len()
                        ),
                    });
                }

                Ok(Value::CaseClassValue(
                    def.name.clone(),
                    values.into_iter().map(Box::new).collect(),
                ))
            } else {
                todo!("unresolved call")
            }
        }

        Expr::Sequence(discard, ret) => {
            interpret(discard, env, table)?;
            interpret(ret, env, table)
        }
        Expr::Let(name, _, value, body, _) => {
            let value = interpret(value, env, table)?;
            env.define(name.clone(), value);

            interpret(body, env, table)
        }
        Expr::Ite(cond, then, elze, _) => {
            let Value::Bool(condval) = interpret(cond, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(cond),
                    msg: format!("expected boolean, found `{}`", cond.show(0)),
                });
            };

            if condval {
                interpret(then, env, table)
            } else {
                interpret(elze, env, table)
            }
        }

        Expr::Match(e, cases, _) => {
            let scrutinee = interpret(e, env, table)?;

            for (pattern, body) in cases {
                if let Some(bindings) = match_and_bind(&scrutinee, pattern) {
                    env.push_scope();
                    env.define_many(bindings);
                    let ret = interpret(body, env, table);
                    env.pop_scope();

                    return ret;
                }
            }

            Err(Report {
                stage: Stage::Interpreting,
                range: range(e),
                msg: format!(
                    "match error: no case pattern matches expression {}",
                    e.show(0)
                ),
            })
        }

        Expr::Error(msg, span) => {
            let Value::String(str) = interpret(msg, env, table)? else {
                return Err(Report {
                    stage: Stage::Interpreting,
                    range: range(msg),
                    msg: format!("expected boolean, found `{}`", msg.show(0)),
                });
            };

            Err(Report {
                stage: Stage::Interpreting,
                range: *span,
                msg: format!("error: {str}"),
            })
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
