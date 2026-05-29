use std::collections::{HashMap, VecDeque};

use rac_ast::{
    DefinitionTable, Expr, Pattern, SID, Symbol, SymbolGenerator, SymbolicProgram, SymbolicType,
    VarKind::*, range,
    environ::Environment
};
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

pub fn typecheck(program: &SymbolicProgram, sg: &mut SymbolGenerator) -> Result<(), Report> {
    let mut constraints = VecDeque::new();

    // Collect constraints from function bodies
    for fdef in program.table.fun_defs.values() {
        let mut env = Environment::new();
        for (sym, typ) in fdef.args.iter() {
            env.define(sym.id, typ.clone());
        }
        println!("ha {:?}", env);
        let fun_constr = collect_constraints(&fdef.body, fdef.rt.clone(), &mut env, &program.table, sg)?;
        constraints.extend(fun_constr);
    }

    // Collect constraints from expressions
    for expr in program.exprs.iter() {
        let expr_constr = collect_constraints(
            expr,
            sg.fresh_type_var(),
            &mut Environment::new(),
            &program.table,
            sg,
        )?;
        constraints.extend(expr_constr);
    }

    // Solve the constraints
    solve_constraints(&mut constraints)
}

fn alpha_conversion(
    vars: &VecDeque<Symbol>,
    sg: &mut SymbolGenerator,
) -> (HashMap<SID, SymbolicType>, VecDeque<SymbolicType>) {
    let mut res = HashMap::new();
    let mut type_args = VecDeque::new();
    for var in vars.iter() {
        let tv = sg.fresh_type_var();
        type_args.push_back(tv.clone());
        res.insert(var.id, tv);
    }
    (res, type_args)
}

fn collect_constraints(
    expr: &Expr<Symbol, SymbolicType>,
    expected: SymbolicType,
    env: &mut Environment<SymbolicType>,
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
        Variable(name, s) => match env.lookup(name.id) {
            Some(typ) => Ok(single!(Constraint::new(expected, typ.clone(), *s))),
            None => {
                println!("{:?}", env);
                error!(*s, "Variable not present in the environment.")
            }
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
            (None, None) => error!(
                *s,
                format!(
                    "Could not find function or constructor named `{}`.",
                    sym.name
                )
            ),
            (Some(def), None) => {
                // Check arity
                if args.len() != def.args.len() {
                    return error!(
                        *s,
                        format!(
                            "Function `{}` takes {} arguments, but was given {}.",
                            sym.name,
                            def.args.len(),
                            args.len()
                        )
                    );
                }
                let mut res = VecDeque::new();
                // Perform alpha conversion on the function type parameters
                let (subst, _) = alpha_conversion(&def.type_vars, sg);
                res.push_back(Constraint::new(expected, type_subst(&def.rt, &subst), *s));
                for (arg, (_, typ)) in args.iter().zip(def.args.iter()) {
                    let mut arg_constr =
                        collect_constraints(arg, type_subst(typ, &subst), env, table, sg)?;
                    res.append(&mut arg_constr);
                }
                Ok(res)
            }
            (None, Some(def)) => {
                let tdef = &table.type_defs[&def.parent.id];
                let (subst, type_args) = alpha_conversion(&tdef.type_vars, sg);
                let mut res: VecDeque<Constraint> = VecDeque::new();
                res.push_front(Constraint::new(
                    expected,
                    ClassType(tdef.name.clone(), type_args),
                    *s,
                ));
                for (arg_expr, (_, arg_typ)) in args.iter().zip(def.args.iter()) {
                    let cur_constr = collect_constraints(
                        arg_expr,
                        type_subst(arg_typ, &subst),
                        env,
                        table,
                        sg,
                    )?;
                    res.extend(cur_constr);
                }
                Ok(res)
            }
            (Some(_), Some(_)) => error!(
                *s,
                format!(
                    "Ambiguous call: `{}` might refer to a function or a constructor.",
                    sym.name
                )
            ),
        },
        Sequence(lhs, rhs) => {
            let tv = sg.fresh_type_var();
            let mut res = collect_constraints(lhs, tv, env, table, sg)?;
            let more = collect_constraints(rhs, expected, env, table, sg)?;
            res.extend(more.into_iter());
            Ok(res)
        }
        Let(name, typ, val, body, _) => {
            let mut res = collect_constraints(val, typ.clone(), env, table, sg)?;
            env.push_scope();
            env.define(name.id, typ.clone());
            println!("ha {:?}", env);
            let body_constr = collect_constraints(body, expected, env, table, sg)?;
            env.pop_scope();
            res.extend(body_constr);
            Ok(res)
        }
        Ite(cond, thenb, elseb, _) => {
            let mut res = collect_constraints(cond, BoolType, env, table, sg)?;
            let mut then_constr =
                collect_constraints(thenb, expected.clone(), env, table, sg)?;
            let mut else_constr = collect_constraints(elseb, expected, env, table, sg)?;
            res.append(&mut then_constr);
            res.append(&mut else_constr);
            Ok(res)
        }
        Match(scrut, pats, _) => {
            let tv = sg.fresh_type_var();
            let mut res = collect_constraints(scrut, tv.clone(), env, table, sg)?;
            for (pat, branch) in pats.iter() {
                let (pat_constr, binds) = pattern_constraints(pat, tv.clone(), table, sg)?;
                res.extend(pat_constr);
                // let mut curenv = env;
                // cur
                env.push_scope();
                env.define_many(binds);
                let branch_constr =
                    collect_constraints(branch, expected.clone(), env, table, sg)?;
                env.pop_scope();
                res.extend(branch_constr);
            }
            Ok(res)
        }
        Error(msg, _) => collect_constraints(msg, StringType, env, table, sg),
    }
}

fn pattern_constraints(
    pat: &Pattern<Symbol>,
    expected: SymbolicType,
    table: &DefinitionTable,
    sg: &mut SymbolGenerator,
) -> Result<(VecDeque<Constraint>, HashMap<SID, SymbolicType>), Report> {
    use Pattern::*;
    macro_rules! literal {
        ($typ:ident, $range:expr) => {
            Ok((
                VecDeque::from([Constraint::new(expected, SymbolicType::$typ, $range)]),
                HashMap::new(),
            ))
        };
    }
    match pat {
        Wildcard(_) => Ok((VecDeque::new(), HashMap::new())),
        IdPattern(name, _) => {
            let mut mp = HashMap::new();
            mp.insert(name.id, expected.clone());
            Ok((VecDeque::new(), mp))
        }
        BoolPattern(_, s) => literal!(BoolType, *s),
        StringPattern(_, s) => literal!(StringType, *s),
        IntPattern(_, s) => literal!(IntType, *s),
        UnitPattern(s) => literal!(UnitType, *s),
        ClassPattern(name, argpats, s) => {
            let def = &table.class_defs[&name.id];
            let tdef = &table.type_defs[&def.parent.id];
            let (subst, type_args) = alpha_conversion(&tdef.type_vars, sg);
            let mut res = VecDeque::new();
            let mut mp = HashMap::new();
            res.push_back(Constraint::new(
                expected,
                SymbolicType::ClassType(tdef.name.clone(), type_args),
                *s,
            ));
            for (argpat, (_, arg_typ)) in argpats.iter().zip(def.args.iter()) {
                let (cur_constr, cur_mp) =
                    pattern_constraints(argpat, type_subst(arg_typ, &subst), table, sg)?;
                mp.extend(cur_mp);
                res.extend(cur_constr);
            }
            Ok((res, mp))
        }
    }
}

fn type_subst(typ: &SymbolicType, subst: &HashMap<SID, SymbolicType>) -> SymbolicType {
    use SymbolicType::*;
    match typ {
        IntType => IntType,
        BoolType => BoolType,
        StringType => StringType,
        UnitType => UnitType,
        ClassType(name, params) => {
            let mut newparams = VecDeque::new();
            for param in params.iter() {
                newparams.push_back(type_subst(param, subst));
            }
            ClassType(name.clone(), newparams)
        }
        Var(name, fluid) => match subst.get(&name.id) {
            None => Var(name.clone(), *fluid),
            Some(newtyp) => newtyp.clone(),
        },
    }
}

fn type_subst_mut(typ: &mut SymbolicType, from: SID, to: &SymbolicType) {
    use SymbolicType::*;
    match typ {
        IntType | BoolType | StringType | UnitType => {}
        ClassType(_, params) => {
            for param in params.into_iter() {
                type_subst_mut(param, from, to);
            }
        }
        Var(name, kind) if from == name.id => *typ = to.clone(),
        _ => {}
    }
}

fn constr_subst_mut(constraints: &mut VecDeque<Constraint>, from: SID, to: &SymbolicType) {
    for constr in constraints.iter_mut() {
        type_subst_mut(&mut constr.expected, from, to);
        type_subst_mut(&mut constr.found, from, to);
    }
}

fn solve_constraints(constraints: &mut VecDeque<Constraint>) -> Result<(), Report> {
    use SymbolicType::*;
    match constraints.pop_front() {
        None => Ok(()),
        Some(cur) => match (cur.expected, cur.found) {
            (Var(name, Fluid), other) => {
                constr_subst_mut(constraints, name.id, &other);
                solve_constraints(constraints)
            }
            (other, Var(name, Fluid)) => {
                constr_subst_mut(constraints, name.id, &other);
                solve_constraints(constraints)
            }
            (Var(name1, Rigid), Var(name2, Rigid)) if name1.id == name2.id => {
                solve_constraints(constraints)
            }
            (IntType, IntType) => solve_constraints(constraints),
            (StringType, StringType) => solve_constraints(constraints),
            (BoolType, BoolType) => solve_constraints(constraints),
            (UnitType, UnitType) => solve_constraints(constraints),
            (ClassType(name1, params1), ClassType(name2, params2)) => {
                if name1.id != name2.id {
                    return error!(
                        cur.range,
                        format!(
                            "Expected type `{}`, found `{}`.",
                            ClassType(name1, params1),
                            ClassType(name2, params2)
                        )
                    );
                }
                for (param1, param2) in params1.into_iter().zip(params2.into_iter()) {
                    constraints.push_front(Constraint::new(param1, param2, cur.range));
                }
                solve_constraints(constraints)
            }
            (expected, found) => {
                error!(
                    cur.range,
                    format!("Expected type `{}`, found `{}`.", expected, found)
                )
            }
        },
    }
}
