use rac_ast::{
    Expr, Name, NominalModule, NominalType, Pattern, SID, Symbol, SymbolGenerator, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef, SymbolicProgram, SymbolicType
};
use rac_diagnostics::{Report, Span, Stage};
use std::{collections::{HashMap, VecDeque}};

use crate::table::{SymbolTable, CallTable, TypeTable};

pub mod table;

macro_rules! error {
    ($span:expr, $msg:expr) => {
        Err(Report {
            stage: Stage::Resolving,
            range: $span,
            msg: String::from($msg),
        })
    };
}

macro_rules! check_unique {
    ($name:expr, $mp:expr, $msg:expr) => {
        for def in $mp.values() {
            if def.name.name == $name {
                return error!(def.range, $msg);
            }
        }
    };
}

fn collect<V>(mp: HashMap<String, HashMap<SID, V>>) -> HashMap<SID, V> {
    let mut res = HashMap::new();
    for sm in mp.into_values() {
        res.extend(sm);
    }
    res
}

pub fn resolve(
    modules: &VecDeque<NominalModule>,
    sg: &mut SymbolGenerator,
) -> Result<SymbolicProgram, Report> {
    // let mut types_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();
    // let mut classes_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();
    // let mut fun_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();

    let mut type_defs: HashMap<String, HashMap<SID, SymbolicAbstDef>> = HashMap::new();
    let mut class_defs: HashMap<String, HashMap<SID, SymbolicClassDef>> = HashMap::new();
    let mut fun_defs: HashMap<String, HashMap<SID, SymbolicFunDef>> = HashMap::new();

    // Discovering user types
    for md in modules.iter() {
        // mod_ids.insert(&md.name, md.id);
        let mut cur_types: HashMap<SID, SymbolicAbstDef> = HashMap::new();
        // let mut cur_types_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.abstract_defs.iter() {
            // Check if type is already defined
            check_unique!(
                def.name,
                cur_types,
                format!("An abstract class named `{}` is already defined.", def.name)
            );

            // Generate the name
            let sym_name = sg.fresh(&def.name);

            // Generate type variable names
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

            // cur_types_by_mod.push_back(sym_name.id);

            cur_types.insert(
                sym_name.id,
                SymbolicAbstDef {
                    name: sym_name,
                    type_vars: sym_type_vars,
                    range: def.range,
                },
            );
        }
        // types_by_mod.insert(&md.name, cur_types_by_mod);
        type_defs.insert(md.name.clone(), cur_types);
    }

    // Discovering class definitions
    for md in modules.iter() {
        let mut cur_cls_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
        // let mut cur_cls_defs_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.class_defs.iter() {
            // Check if the class is already defined
            check_unique!(
                def.name,
                cur_cls_defs,
                format!("A case class named `{}` is already defined.", def.name)
            );

            // Generate the name
            let sym_name = sg.fresh(&def.name);

            // Resolve the parent
            let parent_id = find_type_symbol(&def.parent, def.range, &type_defs[&md.name])?;
            let sym_parent = Symbol::new(&def.parent, parent_id);

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ) in def.args.iter() {
                let sym_typ = resolve_type(
                    typ,
                    &TypeTable::new(
                        &md.name,
                        &type_defs[&md.name][&parent_id].type_vars,
                        &type_defs,
                    ),
                )?;
                let sym_name = sg.fresh(&name);
                sym_args.push_back((sym_name, sym_typ));
            }

            // cur_cls_defs_by_mod.push_back(sym_name.id);

            let sym_def = SymbolicClassDef {
                name: sym_name,
                args: sym_args,
                parent: sym_parent,
                range: def.range,
            };

            cur_cls_defs.insert(sym_def.name.id, sym_def);
        }
        // classes_by_mod.insert(md.id, cur_cls_defs);
        // classes_by_mod.insert(&md.name, cur_cls_defs_by_mod);
        // class_defs.extend(cur_cls_defs);
        class_defs.insert(md.name.clone(), cur_cls_defs);
    }

    // Discovering function definitions
    for md in modules.iter() {
        let mut cur_fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();
        let mut cur_fun_defs_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.fun_defs.iter() {
            // Check if the function is already defined
            check_unique!(
                def.name,
                cur_fun_defs,
                format!("A function `{}` is already defined.", def.name)
            );

            // Generate the name
            let sym_name = sg.fresh(&def.name);

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
                    resolve_type(typ, &TypeTable::new(&md.name, &sym_type_vars, &type_defs))?;
                let sym_name = sg.fresh(&name);
                sym_args.push_back((sym_name, sym_typ));
            }

            // Resolve the return type
            let sym_rt =
                resolve_type(&def.rt, &TypeTable::new(&md.name, &sym_type_vars, &type_defs))?;

            // Resolve the body
            let sym_body = todo!();

            cur_fun_defs_by_mod.push_back(sym_name.id);

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
        // fun_by_mod.insert(&md.name, cur_fun_defs_by_mod);
        // fun_defs.extend(cur_fun_defs);
        fun_defs.insert(md.name.clone(), cur_fun_defs);
    }

    let mut exprs: VecDeque<Expr<Symbol, SymbolicType>> = VecDeque::new();
    for md in modules.iter() {
        match &md.expr {
            None => {}
            Some(expr) => {}
        }
    }

    Ok(SymbolicProgram {
        type_defs: collect(type_defs),
        class_defs: collect(class_defs),
        fun_defs: collect(fun_defs),
        exprs,
    })
    // todo!()
}

fn find_type_symbol(
    name: &String,
    range: Span,
    defs: &HashMap<SID, SymbolicAbstDef>,
) -> Result<SID, Report> {
    for def in defs.values() {
        if def.name.name == *name {
            return Ok(def.name.id);
        }
    }
    return error!(
        range,
        format!("Could not find an abstract class named `{}`.", name)
    );
}

fn find_class_symbol(
    name: &String,
    range: Span,
    defs: &HashMap<SID, SymbolicClassDef>,
) -> Result<SID, Report> {
    for def in defs.values() {
        if def.name.name == *name {
            return Ok(def.name.id);
        }
    }
    return error!(
        range,
        format!("Could not find a case class named `{}`.", name)
    );
}

fn find_fun_symbol(
    name: &String,
    range: Span,
    defs: &HashMap<SID, SymbolicFunDef>,
) -> Result<SID, Report> {
    for def in defs.values() {
        if def.name.name == *name {
            return Ok(def.name.id);
        }
    }
    return error!(
        range,
        format!("Could not find a function named `{}`.", name)
    );
}

fn find_call_symbol(
    name: &String,
    range: Span,
    class_defs: &HashMap<SID, SymbolicClassDef>,
    fun_defs: &HashMap<SID, SymbolicFunDef>,
) -> Result<SID, Report> {
    for def in fun_defs.values() {
        if def.name.name == *name {
            return Ok(def.name.id);
        }
    }
    for def in class_defs.values() {
        if def.name.name == *name {
            return Ok(def.name.id);
        }
    }
    return error!(
        range,
        format!("Could not find a function named `{}`.", name)
    );
}

fn resolve_type(arg: &NominalType, env: &TypeTable) -> Result<SymbolicType, Report> {
    use SymbolicType as ST;
    use rac_ast::NominalType::*;
    match arg {
        IntType(_) => Ok(ST::IntType),
        BoolType(_) => Ok(ST::BoolType),
        StringType(_) => Ok(ST::StringType),
        UnitType(_) => Ok(ST::UnitType),
        IdType(qn, s) => match qn.owner.clone() {
            None => {
                for var in env.type_vars.iter() {
                    if var.name == qn.name {
                        return Ok(ST::Var(Symbol::new(&qn.name, var.id)));
                    }
                }
                let sid = find_type_symbol(&qn.name, *s, &env.type_defs[env.cur_mod])?;
                Ok(ST::ClassType(Symbol::new(&qn.name, sid)))
            }
            Some(owner) => match env.type_defs.get(&owner) {
                Some(mp) => {
                    let sid = find_type_symbol(&qn.name, *s, mp)?;
                    Ok(ST::ClassType(Symbol::new(&qn.name, sid)))
                }
                None => error!(*s, format!("No module named `{}`.", owner)),
            },
        },
    }
}

fn resolve_call(arg: &Name, range: Span, env: &CallTable) -> Result<Symbol, Report> {
    match &arg.owner {
        None => {
            let sid = find_call_symbol(&arg.name, range, &env.class_defs[env.cur_mod], &env.fun_defs[env.cur_mod])?;
            Ok(Symbol::new(&arg.name, sid))
        }
        Some(owner) => match (env.fun_defs.get(owner), env.class_defs.get(owner)) {
            (Some(mp1), Some(mp2)) => {
                let sid = find_call_symbol(&arg.name, range, mp2, mp1)?;
                Ok(Symbol::new(&arg.name, sid))
            }
            _ => error!(range, format!("No module named `{}`.", owner)),
        },
    }
}

fn resolve_pattern(
    pat: &Pattern<Name>,
    env: &SymbolTable,
    binds: &mut HashMap<String, SID>
) -> Result<Pattern<Symbol>, Report> {
    todo!()
}

fn resolve_expr(
    e: Expr<Name, NominalType>,
    env: &mut SymbolTable,
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
            let sym_name = resolve_call(&qn, s, &CallTable::from(&*env))?;
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
        Match(scrut, pats, s) => {
            let sym_scrut = resolve_expr(*scrut, env, binds.clone(), sg)?;
            todo!()
        }
        _ => todo!(),
    }
}

// fn contains_with_name(lst: &VecDeque<Symbol>, nme: &String) -> bool {
//     for item in lst.iter() {
//         if item.name == *nme {
//             return true;
//         }
//     }
//     return false;
// }
