use rac_ast::{
    Expr, Name, NominalModule, NominalType, Pattern, SID, Symbol, SymbolGenerator,
    SymbolicClassDef, SymbolicFunDef, SymbolicProgram, SymbolicType, SymbolicTypeDef,
};
use rac_diagnostics::{Report, Span, Stage};
use std::collections::{HashMap, VecDeque};

use crate::table::{CallTable, SymbolTable, TypeTable};

pub mod table;

macro_rules! error {
    ($range:expr, $msg:expr) => {
        Err(Report {
            stage: Stage::Resolving,
            range: $range,
            msg: String::from($msg),
        })
    };
}

macro_rules! check_unique {
    ($name:expr, $range:expr, $mp:expr, $msg:expr) => {
        if $mp.contains_key($name) {
            return error!($range, $msg);
        }
    };
}

macro_rules! find_type_id {
    ($name:expr, $range:expr, $type_syms:expr) => {
        match $type_syms.get($name) {
            Some(id) => Ok(*id),
            None => error!(
                $range,
                format!("Could not find a type or type variable named `{}`.", $name)
            ),
        }
    };
}

macro_rules! find_class_id {
    ($name:expr, $range:expr, $class_syms:expr) => {
        match $class_syms.get($name) {
            Some(id) => Ok(*id),
            None => error!(
                $range,
                format!("Could not find a constructor named `{}`.", $name)
            ),
        }
    };
}

macro_rules! find_call_id {
    ($name:expr, $range:expr, $cls_syms:expr, $fun_syms:expr) => {
        match $fun_syms.get($name) {
            Some(id) => Ok(*id),
            None => match $cls_syms.get($name) {
                Some(id) => Ok(*id),
                None => error!(
                    $range,
                    format!("No function or constructor named `{}`.", $name)
                ),
            },
        }
    };
}

pub fn resolve(
    modules: VecDeque<NominalModule>,
    sg: &mut SymbolGenerator,
) -> Result<SymbolicProgram, Report> {
    // Definition maps
    let mut type_defs: HashMap<SID, SymbolicTypeDef> = HashMap::new();
    let mut class_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
    let mut fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();

    // Symbol tables
    let mut type_syms: HashMap<String, HashMap<String, SID>> = HashMap::new();
    let mut class_syms: HashMap<String, HashMap<String, SID>> = HashMap::new();
    let mut fun_syms: HashMap<String, HashMap<String, SID>> = HashMap::new();

    // Discover type symbols and definitions
    for md in modules.iter() {
        if type_syms.contains_key(&md.name) {
            return error!(md.range, format!("Module `{}` already defined.", md.name));
        }

        let mut cur_type_defs: HashMap<SID, SymbolicTypeDef> = HashMap::new();
        let mut cur_type_syms: HashMap<String, SID> = HashMap::new();

        for def in md.type_defs.iter() {
            // Check if type is already defined
            check_unique!(
                &def.name,
                def.range,
                cur_type_syms,
                format!(
                    "A type named `{}` is already defined in the module `{}`",
                    def.name, md.name
                )
            );

            // Generate the type SID
            let sid = sg.fresh_id();
            cur_type_syms.insert(def.name.clone(), sid);

            // Generate the type name
            let sym_name = Symbol::new(&def.name, sid);

            // Generate type variable names
            let mut sym_type_vars: VecDeque<Symbol> = VecDeque::new();
            for type_var in def.type_vars.iter() {
                // Check if this type variable is already used
                for sym_var in sym_type_vars.iter() {
                    if sym_var.name == *type_var {
                        return error!(
                            def.range,
                            format!(
                                "A type variable named {} is already used in this abstract class.",
                                def.name
                            )
                        );
                    }
                }
                sym_type_vars.push_back(sg.fresh(&type_var));
            }

            // Insert the type definition
            cur_type_defs.insert(
                sid,
                SymbolicTypeDef {
                    name: sym_name,
                    type_vars: sym_type_vars,
                    range: def.range,
                },
            );
        }

        type_syms.insert(md.name.clone(), cur_type_syms);
        type_defs.extend(cur_type_defs);
    }

    // Discovering class definitions
    for md in modules.iter() {
        let mut cur_cls_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
        let mut cur_cls_syms: HashMap<String, SID> = HashMap::new();

        for def in md.class_defs.iter() {
            // Check if the class is already defined
            check_unique!(
                &def.name,
                def.range,
                cur_cls_syms,
                format!(
                    "A case class named `{}` is already defined in the `{}` module",
                    def.name, md.name
                )
            );

            // Generate the SID and add it to the table.
            let sid = sg.fresh_id();
            cur_cls_syms.insert(def.name.clone(), sid);

            // Generate the symbol
            let sym_name = Symbol::new(&def.name, sid);

            // Generate the parent SID and symbol
            let parent_id = find_type_id!(&def.parent, def.range, type_syms[&md.name])?;
            let sym_parent = Symbol::new(&def.parent, parent_id);

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ) in def.args.iter() {
                let sym_typ = resolve_type(
                    typ,
                    &TypeTable::new(&md.name, &type_defs[&parent_id].type_vars, &type_syms),
                )?;
                let sym_name = sg.fresh(&name);
                sym_args.push_back((sym_name, sym_typ));
            }

            cur_cls_defs.insert(
                sid,
                SymbolicClassDef {
                    name: sym_name,
                    args: sym_args,
                    parent: sym_parent,
                    range: def.range,
                },
            );
        }

        class_defs.extend(cur_cls_defs);
        class_syms.insert(md.name.clone(), cur_cls_syms);
    }

    let mut exprs: VecDeque<(String, Expr<Name, NominalType>)> = VecDeque::new();

    // Discover function symbols
    for md in modules.iter() {
        let mut cur_fun_syms: HashMap<String, SID> = HashMap::new();
        for def in md.fun_defs.iter() {
            check_unique!(
                &def.name,
                def.range,
                cur_fun_syms,
                format!(
                    "A function `{}` is already defined in the `{}` module.",
                    def.name, md.name
                )
            );

            cur_fun_syms.insert(def.name.clone(), sg.fresh_id());
        }

        fun_syms.insert(md.name.clone(), cur_fun_syms);
    }

    // Discover function definitions
    for md in modules.into_iter() {
        let mut cur_fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();
        for def in md.fun_defs.into_iter() {
            // Retrieve the SID
            let sid = fun_syms[&md.name][&def.name];

            // Generate the name
            let sym_name = Symbol::new(&def.name, sid);

            // Generate the type variables
            let mut sym_type_vars: VecDeque<Symbol> = VecDeque::new();
            for type_var in def.type_vars.iter() {
                // Checking if this type variable is already used
                for sym_var in sym_type_vars.iter() {
                    if sym_var.name == *type_var {
                        return error!(
                            def.range,
                            format!(
                                "A type variable named {} is already used in this abstract class.",
                                def.name
                            )
                        );
                    }
                }
                sym_type_vars.push_back(sg.fresh(&type_var));
            }

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ) in def.args.iter() {
                let sym_typ =
                    resolve_type(typ, &TypeTable::new(&md.name, &sym_type_vars, &type_syms))?;
                let sym_name = sg.fresh(&name);
                sym_args.push_back((sym_name, sym_typ));
            }

            // Resolve the return type
            let sym_rt = resolve_type(
                &def.rt,
                &TypeTable::new(&md.name, &sym_type_vars, &type_syms),
            )?;

            // Resolve the body
            let mut binds = HashMap::new();
            for (sym, _) in sym_args.iter() {
                binds.insert(sym.name.clone(), sym.id);
            }
            let sym_body = resolve_expr(
                def.body,
                &SymbolTable::new(
                    &md.name,
                    &sym_type_vars,
                    &type_syms,
                    &class_syms,
                    &fun_syms,
                    Some((def.name, sym_name.id)),
                ),
                binds,
                sg,
            )?;

            cur_fun_defs.insert(
                sym_name.id,
                SymbolicFunDef {
                    name: sym_name,
                    type_vars: sym_type_vars,
                    args: sym_args,
                    rt: sym_rt,
                    body: sym_body,
                    range: def.range,
                },
            );
        }

        fun_defs.extend(cur_fun_defs);

        if let Some(e) = md.expr {
            exprs.push_back((md.name, e));
        }
    }

    // Transform expressions
    let mut sym_exprs = VecDeque::new();
    for (modname, expr) in exprs.into_iter() {
        let sym_expr = resolve_expr(
            expr,
            &SymbolTable::new(
                &modname,
                &VecDeque::new(),
                &type_syms,
                &class_syms,
                &fun_syms,
                None,
            ),
            HashMap::new(),
            sg,
        )?;
        sym_exprs.push_back(sym_expr);
    }

    Ok(SymbolicProgram {
        type_defs: type_defs,
        class_defs: class_defs,
        fun_defs: fun_defs,
        exprs: sym_exprs,
    })
}

fn resolve_type(arg: &NominalType, env: &TypeTable) -> Result<SymbolicType, Report> {
    use SymbolicType as ST;
    use rac_ast::NominalType::*;
    match arg {
        IntType(_) => Ok(ST::IntType),
        BoolType(_) => Ok(ST::BoolType),
        StringType(_) => Ok(ST::StringType),
        UnitType(_) => Ok(ST::UnitType),
        IdType(qn, params, s) => match qn.owner.clone() {
            None => {
                for var in env.type_vars.iter() {
                    if var.name == qn.name {
                        if params.len() > 0 {
                            return error!(
                                *s,
                                format!(
                                    "The type variable `{}` cannot take any type parameters.",
                                    qn.name
                                )
                            );
                        }
                        return Ok(ST::Var(Symbol::new(&qn.name, var.id)));
                    }
                }
                let sid = find_type_id!(&qn.name, *s, &env.type_syms[env.cur_mod])?;
                let mut sym_params = VecDeque::new();
                for param in params.iter() {
                    let sym_param = resolve_type(param, env)?;
                    sym_params.push_back(sym_param);
                }
                Ok(ST::ClassType(Symbol::new(&qn.name, sid), sym_params))
            }
            Some(owner) => match env.type_syms.get(&owner) {
                Some(mp) => {
                    let sid = find_type_id!(&qn.name, *s, mp)?;
                    let mut sym_params = VecDeque::new();
                    for param in params.iter() {
                        let sym_param = resolve_type(param, env)?;
                        sym_params.push_back(sym_param);
                    }
                    Ok(ST::ClassType(Symbol::new(&qn.name, sid), sym_params))
                }
                None => error!(*s, format!("No module named `{}`.", owner)),
            },
        },
    }
}

fn resolve_call(arg: &Name, range: Span, env: &CallTable) -> Result<Symbol, Report> {
    match &arg.owner {
        None => {
            let sid = find_call_id!(
                &arg.name,
                range,
                &env.class_syms[env.cur_mod],
                &env.fun_syms[env.cur_mod]
            )?;
            Ok(Symbol::new(&arg.name, sid))
        }
        Some(owner) => match (env.fun_syms.get(owner), env.class_syms.get(owner)) {
            (Some(mp1), Some(mp2)) => {
                let sid = find_call_id!(&arg.name, range, mp2, mp1)?;
                Ok(Symbol::new(&arg.name, sid))
            }
            _ => error!(range, format!("No module named `{}`.", owner)),
        },
    }
}

fn resolve_class(arg: &Name, range: Span, env: &SymbolTable) -> Result<Symbol, Report> {
    match &arg.owner {
        None => {
            let sid = find_class_id!(&arg.name, range, &env.class_syms[env.cur_mod])?;
            Ok(Symbol::new(&arg.name, sid))
        }
        Some(owner) => match env.class_syms.get(owner) {
            Some(mp2) => {
                let sid = find_class_id!(&arg.name, range, mp2)?;
                Ok(Symbol::new(&arg.name, sid))
            }
            None => error!(range, format!("No module named `{}`.", owner)),
        },
    }
}

fn resolve_pattern(
    pat: Pattern<Name>,
    env: &SymbolTable,
    binds: &mut HashMap<String, SID>,
    sg: &mut SymbolGenerator,
) -> Result<Pattern<Symbol>, Report> {
    use Pattern::*;
    match pat {
        Wildcard(s) => Ok(Wildcard(s)),
        IdPattern(qn, s) => {
            let sym_name = sg.fresh(&qn.name);
            binds.insert(qn.name, sym_name.id);
            Ok(IdPattern(sym_name, s))
        }
        BoolPattern(val, s) => Ok(BoolPattern(val, s)),
        StringPattern(val, s) => Ok(StringPattern(val, s)),
        IntPattern(val, s) => Ok(IntPattern(val, s)),
        UnitPattern(s) => Ok(UnitPattern(s)),
        ClassPattern(qn, subpats, s) => {
            let sym_name = resolve_class(&qn, s, env)?;
            let mut sym_subpats = VecDeque::new();
            for sp in subpats.into_iter() {
                let sym_sp = resolve_pattern(sp, env, binds, sg)?;
                sym_subpats.push_back(sym_sp);
            }
            Ok(ClassPattern(sym_name, sym_subpats, s))
        }
    }
}

fn resolve_expr(
    e: Expr<Name, NominalType>,
    env: &SymbolTable,
    mut binds: HashMap<String, SID>,
    sg: &mut SymbolGenerator,
) -> Result<Expr<Symbol, SymbolicType>, Report> {
    use Expr::*;

    macro_rules! binop {
        ($lhs:expr, $rhs:expr, $constr:ident) => {{
            let sym_lhs = resolve_expr($lhs, env, binds.clone(), sg)?;
            let sym_rhs = resolve_expr($rhs, env, binds, sg)?;
            Ok($constr(Box::new(sym_lhs), Box::new(sym_rhs)))
        }};
    }

    macro_rules! unop {
        ($arg:expr, $range:expr, $constr:ident) => {{
            let sym_arg = resolve_expr($arg, env, binds, sg)?;
            Ok($constr(Box::new(sym_arg), $range))
        }};
    }

    match e {
        Variable(qn, s) => match binds.get(&qn.name) {
            Some(sid) => Ok(Variable(Symbol::new(&qn.name, *sid), s)),
            None => error!(
                s,
                format!("Variable `{}` not found in current scope.", qn.name)
            ),
        },
        IntLiteral(val, s) => Ok(IntLiteral(val, s)),
        BoolLiteral(val, s) => Ok(BoolLiteral(val, s)),
        StringLiteral(val, s) => Ok(StringLiteral(val, s)),
        UnitLiteral(s) => Ok(UnitLiteral(s)),
        Plus(lhs, rhs) => binop!(*lhs, *rhs, Plus),
        Minus(lhs, rhs) => binop!(*lhs, *rhs, Minus),
        Times(lhs, rhs) => binop!(*lhs, *rhs, Times),
        Div(lhs, rhs) => binop!(*lhs, *rhs, Div),
        Mod(lhs, rhs) => binop!(*lhs, *rhs, Mod),
        LessThan(lhs, rhs) => binop!(*lhs, *rhs, LessThan),
        LessEquals(lhs, rhs) => binop!(*lhs, *rhs, LessEquals),
        And(lhs, rhs) => binop!(*lhs, *rhs, And),
        Or(lhs, rhs) => binop!(*lhs, *rhs, Or),
        Equals(lhs, rhs) => binop!(*lhs, *rhs, Equals),
        Concat(lhs, rhs) => binop!(*lhs, *rhs, Concat),
        Sequence(lhs, rhs) => binop!(*lhs, *rhs, Sequence),
        Not(arg, s) => unop!(*arg, s, Not),
        Neg(arg, s) => unop!(*arg, s, Neg),
        Call(qn, args, s) => {
            let sym_name = match (&qn.owner, &env.fname) {
                (None, Some((name, sid))) if qn.name == *name => Ok(Symbol::new(&name, *sid)),
                _ => resolve_call(&qn, s, &CallTable::from(&*env)),
            }?;
            let mut sym_args = VecDeque::new();
            // let iter = args.iter().map(|arg| resolve_expr(arg, env, sg))
            for arg in args.into_iter() {
                let sym_arg = resolve_expr(arg, env, binds.clone(), sg)?;
                sym_args.push_back(sym_arg);
            }
            Ok(Call(sym_name, sym_args, s))
        }
        Let(qn, typ, val, body, s) => {
            let sym_name = sg.fresh(&qn.name);
            let sym_typ = resolve_type(&typ, &TypeTable::from(&*env))?;
            let sym_val = resolve_expr(*val, env, binds.clone(), sg)?;
            binds.insert(qn.name, sym_name.id);
            let sym_body = resolve_expr(*body, env, binds, sg)?;
            Ok(Let(
                sym_name,
                sym_typ,
                Box::new(sym_val),
                Box::new(sym_body),
                s,
            ))
        }
        Ite(cond, thenb, elseb, s) => {
            let sym_cond = resolve_expr(*cond, env, binds.clone(), sg)?;
            let sym_then = resolve_expr(*thenb, env, binds.clone(), sg)?;
            let sym_else = resolve_expr(*elseb, env, binds, sg)?;
            Ok(Ite(
                Box::new(sym_cond),
                Box::new(sym_then),
                Box::new(sym_else),
                s,
            ))
        }
        Match(scrut, cases, s) => {
            let sym_scrut = resolve_expr(*scrut, env, binds.clone(), sg)?;
            let mut sym_cases = VecDeque::new();
            for (pat, expr) in cases.into_iter() {
                let mut newbinds = binds.clone();
                let sym_pat = resolve_pattern(pat, env, &mut newbinds, sg)?;
                let sym_expr = resolve_expr(expr, env, newbinds, sg)?;
                sym_cases.push_back((sym_pat, sym_expr));
            }
            Ok(Match(Box::new(sym_scrut), sym_cases, s))
        }
        Error(msg, s) => {
            let sym_msg = resolve_expr(*msg, env, binds, sg)?;
            Ok(Error(Box::new(sym_msg), s))
        }
    }
}
