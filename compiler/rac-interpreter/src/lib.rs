use rac_ast::{
    DefinitionTable, Expr, Pattern, SID, Symbol, SymbolicProgram, SymbolicType,
    environ::Environment, range,
};
use rac_diagnostics::{Report, Stage, join};

use crate::{builtin::*, value::Value};

mod value;
mod builtin;

pub fn interpret(program: SymbolicProgram) -> Result<(), Report> {
    let mut env = Environment::new();
    for expr in &program.exprs {
        interpret_expr(expr, &mut env, &program.table)?;
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

pub fn interpret_expr(
    expr: &Expr<Symbol, SymbolicType>,
    env: &mut Environment<Value>,
    table: &DefinitionTable,
) -> Result<Value, Report> {
    match expr {
        Expr::BoolLiteral(b, _) => Ok(Value::Bool(*b)),
        Expr::IntLiteral(i, _) => Ok(Value::Int(*i)),
        Expr::StringLiteral(s, _) => Ok(Value::String(s.clone())),
        Expr::UnitLiteral(_) => Ok(Value::Unit),

        Expr::Variable(name, span) => env.lookup(name.id).ok_or(report!(
            *span,
            format!("Variable `{}` not found in scope.", name.name)
        )),

        Expr::Plus(lhs, rhs) => Ok(interpret_expr(lhs, env, table)? + interpret_expr(rhs, env, table)?),
        Expr::Minus(lhs, rhs) => Ok(interpret_expr(lhs, env, table)? - interpret_expr(rhs, env, table)?),
        Expr::Times(lhs, rhs) => Ok(interpret_expr(lhs, env, table)? * interpret_expr(rhs, env, table)?),
        Expr::Div(lhs, rhs) => Ok(interpret_expr(lhs, env, table)? / interpret_expr(rhs, env, table)?),
        Expr::Mod(lhs, rhs) => Ok(interpret_expr(lhs, env, table)? % interpret_expr(rhs, env, table)?),
        Expr::LessEquals(lhs, rhs) => Ok(Value::Bool(
            interpret_expr(lhs, env, table)? <= interpret_expr(rhs, env, table)?,
        )),
        Expr::LessThan(lhs, rhs) => Ok(Value::Bool(
            interpret_expr(lhs, env, table)? < interpret_expr(rhs, env, table)?,
        )),
        Expr::And(lhs, rhs) => {
            if let Value::Bool(false) = interpret_expr(lhs, env, table)? {
                Ok(Value::Bool(false))
            } else {
                interpret_expr(rhs, env, table)
            }
        }
        Expr::Or(lhs, rhs) => {
            if let Value::Bool(true) = interpret_expr(lhs, env, table)? {
                Ok(Value::Bool(true))
            } else {
                interpret_expr(rhs, env, table)
            }
        }
        Expr::Equals(lhs, rhs) => Ok(Value::Bool(
            interpret_expr(lhs, env, table)? == interpret_expr(rhs, env, table)?,
        )),
        Expr::Concat(lhs, rhs) => {
            let lhs_res = interpret_expr(lhs, env, table)?;
            let Value::String(s1) = lhs_res else {
                return Err(report!(
                    range(lhs),
                    format!("Expected a string, found `{}`.", lhs_res)
                ));
            };
            let rhs_res = interpret_expr(rhs, env, table)?;
            let Value::String(s2) = rhs_res else {
                return Err(report!(
                    range(lhs),
                    format!("Expected a string, found `{}`.", rhs_res)
                ));
            };
            Ok(Value::String(s1 + s2.as_str()))
        }

        Expr::Not(e, _) => {
            let res = interpret_expr(e, env, table)?;
            let Value::Bool(b) = res else {
                return Err(report!(
                    range(e),
                    format!("Expected a boolean, found `{}`.", res)
                ));
            };

            Ok(Value::Bool(!b))
        }
        Expr::Neg(e, _) => {
            let res = interpret_expr(e, env, table)?;
            let Value::Int(i) = res else {
                return Err(report!(
                    range(e),
                    format!("Expected an integer, found `{}`.", res)
                ));
            };

            Ok(Value::Int(-i))
        }

        Expr::Call(name, args, span) => {
            macro_rules! call_builtin {
                ($fun:ident) => {{
                    let values = args
                        .iter()
                        .map(|e| interpret_expr(e, env, table))
                        .collect::<Result<Vec<_>, _>>()?;
                    return $fun(values, *span);
                }};
            }
            match name.name.as_str() {
                "Std.printInt" => call_builtin!(print_int),
                "Std.printString" => call_builtin!(print_string),
                "Std.readString" => call_builtin!(read_string),
                "Std.readInt" => call_builtin!(read_int),
                _ => {}
            }
            if let Some(def) = table.fun_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret_expr(e, env, table))
                    .collect::<Result<Vec<_>, _>>()?;

                let map = def
                    .args
                    .iter()
                    .zip(values)
                    .map(|((sym, _), val)| (sym.id, val));
                env.push_scope();
                env.define_many(map);
                let ret = interpret_expr(&def.body, env, table);
                env.pop_scope();

                ret
            } else if let Some(def) = table.class_defs.get(&name.id) {
                let values = args
                    .iter()
                    .map(|e| interpret_expr(e, env, table))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Value::CaseClassValue(
                    def.name.clone(),
                    values.into_iter().map(Box::new).collect(),
                ))
            } else {
                Err(report!(*span, String::from("Unresolved call.")))
            }
        }

        Expr::Sequence(discard, ret) => {
            interpret_expr(discard, env, table)?;
            interpret_expr(ret, env, table)
        }
        Expr::Let(name, _, value, body, _) => {
            let value = interpret_expr(value, env, table)?;
            env.push_scope();
            env.define(name.id, value);
            let res = interpret_expr(body, env, table);
            env.pop_scope();
            res
        }
        Expr::Ite(cond, then, elze, _) => {
            let condres = interpret_expr(cond, env, table)?;
            let Value::Bool(condval) = condres else {
                return Err(report!(
                    range(cond),
                    format!("Expected a boolean, found `{}`.", condres)
                ));
            };

            if condval {
                interpret_expr(then, env, table)
            } else {
                interpret_expr(elze, env, table)
            }
        }

        Expr::Match(e, cases, span) => {
            let scrutinee = interpret_expr(e, env, table)?;

            for (pattern, body) in cases {
                if let Some(bindings) = match_and_bind(&scrutinee, pattern) {
                    env.push_scope();
                    env.define_many(bindings);
                    let ret = interpret_expr(body, env, table);
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
            let Value::String(str) = interpret_expr(msg, env, table)? else {
                return Err(report!(
                    range(msg),
                    format!("Expected a boolean, found `{}`.", msg.show(0))
                ));
            };

            Err(report!(*span, format!("Error: {str}")))
        }
    }
}

fn match_and_bind(scrutinee: &Value, pattern: &Pattern<Symbol>) -> Option<Vec<(SID, Value)>> {
    match (scrutinee, pattern) {
        (_, Pattern::Wildcard(_)) => Some(vec![]),
        (value, Pattern::IdPattern(sym, _)) => Some(vec![(sym.id, value.clone())]),
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
