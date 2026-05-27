use std::collections::{HashMap, VecDeque};

use rac_ast::{DefinitionTable, Expr, Symbol, SymbolGenerator, SymbolicType, range};
use rac_diagnostics::{Report, Stage};

use crate::constraint::Constraint;

pub mod constraint;

macro_rules! single {
    ($elt:expr) => {{
        let mut res = VecDeque::new();
        res.push_front($elt);
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

fn collect_constraints(
    expr: &Expr<Symbol, SymbolicType>,
    expected: SymbolicType,
    env: &mut HashMap<Symbol, SymbolicType>,
    table: &DefinitionTable,
    sg: &mut SymbolGenerator,
) -> Result<VecDeque<Constraint>, Report> {
    use Expr::*;
    use SymbolicType::*;

    macro_rules! toplevel_constraint {
        ($found:expr, $range:expr) => {{
            let mut res = VecDeque::new();
            res.push_front(Constraint::new(expected, $found, $range));
            res
        }};
    }

    macro_rules! binop {
        ($lhs:expr, $rhs:expr, $rt:expr, $lhstyp:expr, $rhstyp:expr) => {{
            let mut res = collect_constraints($lhs, $lhstyp, env, table, sg)?;
            let mut rhs_constr = collect_constraints($rhs, $rhstyp, env, table, sg)?;
            res.append(&mut rhs_constr);
            res.push_front(Constraint::new(expected, $rt, range(expr)));
            Ok(res)
        }};
    }

    macro_rules! unary {
        ($arg:expr, $range:expr, $rt:expr, $argtyp:expr) => {{
            let mut res = collect_constraints($arg, $argtyp, env, table, sg)?;
            res.push_front(Constraint::new(expected, $rt, $range));
            Ok(res)
        }};
    }

    match expr {
        Variable(name, s) => match env.get(&name) {
            Some(typ) => Ok(single!(Constraint::new(expected, typ.clone(), *s))),
            None => error!(*s, "Variable not present in the environment."),
        },
        IntLiteral(_, s) => Ok(toplevel_constraint!(IntType, *s)),
        BoolLiteral(_, s) => Ok(toplevel_constraint!(BoolType, *s)),
        StringLiteral(_, s) => Ok(toplevel_constraint!(StringType, *s)),
        UnitLiteral(s) => Ok(toplevel_constraint!(UnitType, *s)),
        Plus(lhs, rhs) => binop!(lhs, rhs, IntType, IntType, IntType),
        Minus(lhs, rhs) => binop!(lhs, rhs, IntType, IntType, IntType),
        Times(lhs, rhs) => binop!(lhs, rhs, IntType, IntType, IntType),
        Div(lhs, rhs) => binop!(lhs, rhs, IntType, IntType, IntType),
        Mod(lhs, rhs) => binop!(lhs, rhs, IntType, IntType, IntType),
        LessThan(lhs, rhs) => binop!(lhs, rhs, BoolType, IntType, IntType),
        LessEquals(lhs, rhs) => binop!(lhs, rhs, BoolType, IntType, IntType),
        And(lhs, rhs) => binop!(lhs, rhs, BoolType, BoolType, BoolType),
        Or(lhs, rhs) => binop!(lhs, rhs, BoolType, BoolType, BoolType),
        Concat(lhs, rhs) => binop!(lhs, rhs, StringType, StringType, StringType),
        Equals(lhs, rhs) => {
            let tv = sg.fresh_type_var();
            binop!(lhs, rhs, BoolType, tv.clone(), tv)
        }
        Not(arg, s) => unary!(arg, *s, BoolType, BoolType),
        Neg(arg, s) => unary!(arg, *s, IntType, IntType),
        Call(sym, args, s) => match (table.fun_defs.get(&sym.id), table.class_defs.get(&sym.id)) {
            (None, None) => error!(*s, format!("Could not find function or constructor named `{}`.", sym.name)),
            (Some(def), _) => {
                if (args.len() != def.args.len()) {
                    return error!(*s, format!("Function `{}` takes `{}` arguments, but was given `{}`.", sym.name, def.args.len(), args.len()))
                }
                todo!()
            }
            _ => todo!()
        }
        Sequence(lhs, rhs) => {
            let tv = sg.fresh_type_var();
            let mut res = collect_constraints(lhs, tv, env, table, sg)?;
            let more = collect_constraints(rhs, expected, env, table, sg)?;
            res.extend(more.into_iter());
            Ok(res)
        },
        Let(_, _, _, _, _) => todo!(),
        Ite(cond, thenb, elseb, _s) => {
            let mut res = collect_constraints(cond, BoolType, env, table, sg)?;
            let mut then_constr = collect_constraints(thenb, expected.clone(), env, table, sg)?;
            let mut else_constr = collect_constraints(elseb, expected, env, table, sg)?;
            res.append(&mut then_constr);
            res.append(&mut else_constr);
            Ok(res)
        }
        Match(_, _, _) => todo!(),
        Error(msg, _) => collect_constraints(msg, StringType, env, table, sg),
    }
}
