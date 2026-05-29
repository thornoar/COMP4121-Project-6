use rac_ast::{DefinitionTable, Expr, Pattern, Symbol, SymbolicProgram, SymbolicType, range};
use rac_diagnostics::{Report, Stage, join};

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

        Expr::Variable(name, span) => env.lookup(name).ok_or(report!(
            *span,
            format!("Variable `{}` not found in scope.", name.name)
        )),

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
            let lhs_res = interpret(lhs, env, table)?;
            let Value::String(s1) = lhs_res else {
                return Err(report!(
                    range(lhs),
                    format!("expected string, found `{}`", lhs_res)
                ));
            };
            let rhs_res = interpret(rhs, env, table)?;
            let Value::String(s2) = rhs_res else {
                return Err(report!(
                    range(lhs),
                    format!("expected string, found `{}`", rhs_res)
                ));
            };
            Ok(Value::String(s1 + s2.as_str()))
        }

        Expr::Not(e, _) => {
            let res = interpret(e, env, table)?;
            let Value::Bool(b) = res else {
                return Err(report!(
                    range(e),
                    format!("Expected a boolean, found `{}`.", res)
                ));
            };

            Ok(Value::Bool(!b))
        }
        Expr::Neg(e, _) => {
            let res = interpret(e, env, table)?;
            let Value::Int(i) = res else {
                return Err(report!(
                    range(e),
                    format!("Expected an integer, found `{}`.", res)
                ));
            };

            Ok(Value::Int(-i))
        }

        Expr::Call(name, args, span) => {
            if let Some(def) = table.fun_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret(e, env, table))
                    .collect::<Result<Vec<_>, _>>()?;

                // Arity checks are already done by the typechecker

                // if values.len() != def.args.len() {
                //     return Err(report!(
                //         *span,
                //         format!(
                //             "expected {} arguments, found {}",
                //             def.args.len(),
                //             values.len()
                //         )
                //     ));
                // }
                

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

                // Arity checks are done by the typechecker

                // if values.len() != def.args.len() {
                //     return Err(report!(
                //         *span,
                //         format!(
                //             "expected {} arguments, found {}",
                //             def.args.len(),
                //             values.len()
                //         )
                //     ));
                // }

                Ok(Value::CaseClassValue(
                    def.name.clone(),
                    values.into_iter().map(Box::new).collect(),
                ))
            } else {
                Err(report!(*span, String::from("Unresolved call.")))
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
            let condres = interpret(cond, env, table)?;
            let Value::Bool(condval) = condres else {
                return Err(report!(
                    range(cond),
                    format!("Expected a boolean, found `{}`.", condres)
                ));
            };

            if condval {
                interpret(then, env, table)
            } else {
                interpret(elze, env, table)
            }
        }

        Expr::Match(e, cases, span) => {
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

            Err(report!(
                join(range(e), *span),
                format!(
                    "Match error: no case pattern matches value `{}`.",
                    scrutinee
                )
            ))
        }

        Expr::Error(msg, span) => {
            let Value::String(str) = interpret(msg, env, table)? else {
                return Err(report!(
                    range(msg),
                    format!("expected boolean, found `{}`", msg.show(0))
                ));
            };

            Err(report!(*span, format!("error: {str}")))
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
