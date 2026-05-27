use rac_ast::{
    Expr, NominalModule, SID, Symbol, SymbolGenerator, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef, SymbolicProgram, NominalType, SymbolicType
};
use rac_diagnostics::{MID, Report, Span, Stage};
use std::{collections::{HashMap, VecDeque}};

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
                return error!(
                    def.range,
                    $msg
                );
            }
        }
    };
}

pub fn resolve(modules: &VecDeque<NominalModule>, sg: &mut SymbolGenerator) -> Result<SymbolicProgram, Report> {
    let mut types_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();
    let mut classes_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();
    let mut fun_by_mod: HashMap<&String, VecDeque<SID>> = HashMap::new();

    let mut user_types: HashMap<SID, SymbolicAbstDef> = HashMap::new();
    let mut class_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
    let mut fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();

    // Discovering user types
    for md in modules.iter() {
        // mod_ids.insert(&md.name, md.id);
        let mut cur_types: HashMap<SID, SymbolicAbstDef> = HashMap::new();
        let mut cur_types_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.abstract_defs.iter() {
            // Check if type is already defined
            check_unique!(def.name, cur_types, format!("An abstract class named `{}` is already defined.", def.name));

            // Generate the name
            let sym_name = sg.fresh(def.name.clone());

            // Generate type variable names
            let mut sym_type_vars: VecDeque<Symbol> = VecDeque::new();
            for type_var in def.type_vars.iter() {
                // Checking if this type variable is already used
                for sym_var in sym_type_vars.iter() {
                    if sym_var.name == *type_var {
                        return error!(
                            def.range,
                            format!("A type variable named {} is already used in this abstract class.", def.name)
                        );
                    }
                }
                sym_type_vars.push_back(sg.fresh(type_var.clone()));
            }

            cur_types_by_mod.push_back(sym_name.id);

            cur_types.insert(
                sym_name.id, 
                SymbolicAbstDef {
                    name: sym_name,
                    type_vars: sym_type_vars,
                    range: def.range,
                }
            );
        }
        types_by_mod.insert(&md.name, cur_types_by_mod);
        user_types.extend(cur_types);
    }

    // Discovering class definitions
    for md in modules.iter() {
        let mut cur_cls_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
        let mut cur_cls_defs_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.class_defs.iter() {
            // Check if the class is already defined
            check_unique!(def.name, cur_cls_defs, format!("A case class named `{}` is already defined.", def.name));
            
            // Generate the name
            let sym_name = sg.fresh(def.name.clone());

            // Resolve the parent
            let parent_id = find_type_symbol(&def.parent, def.range, &types_by_mod[&md.name], &user_types)?;
            let sym_parent = Symbol { name: def.parent.clone(), id: parent_id };

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ, s) in def.args.iter() {
                let sym_typ = resolve_type(typ, &md.name, *s, &types_by_mod, &user_types)?;
                let sym_name = sg.fresh(name.clone());
                sym_args.push_back((sym_name, sym_typ, *s));
            }

            cur_cls_defs_by_mod.push_back(sym_name.id);

            let sym_def = SymbolicClassDef {
                name: sym_name,
                args: sym_args,
                parent: sym_parent,
                range: def.range
            };

            cur_cls_defs.insert(sym_def.name.id, sym_def);
        }
        // classes_by_mod.insert(md.id, cur_cls_defs);
        classes_by_mod.insert(&md.name, cur_cls_defs_by_mod);
        class_defs.extend(cur_cls_defs);
    }

    // Discovering function definitions
    for md in modules.iter() {
        let mut cur_fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();
        let mut cur_fun_defs_by_mod: VecDeque<SID> = VecDeque::new();
        for def in md.fun_defs.iter() {
            // Check if the function is already defined
            check_unique!(def.name, cur_fun_defs, format!("A function `{}` is already defined.", def.name));

            // Generate the name
            let sym_name = sg.fresh(def.name.clone());

            // Generate the type variables
            let mut sym_type_vars: VecDeque<Symbol> = VecDeque::new();
            for type_var in def.type_vars.iter() {
                // Checking if this type variable is already used
                for sym_var in sym_type_vars.iter() {
                    if sym_var.name == *type_var {
                        return error!(
                            def.range,
                            format!("A type variable named {} is already used in this abstract class.", def.name)
                        );
                    }
                }
                sym_type_vars.push_back(sg.fresh(type_var.clone()));
            }

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ, s) in def.args.iter() {
                let sym_typ = resolve_type(typ, &md.name, *s, &types_by_mod, &user_types)?;
                let sym_name = sg.fresh(name.clone());
                sym_args.push_back((sym_name, sym_typ, *s));
            }
        
            // Resolve the return type
            let sym_rt = resolve_type(&def.rt.0, &md.name, def.rt.1, &types_by_mod, &user_types)?;

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
                    range: def.range
                }
            );
        }
        fun_by_mod.insert(&md.name, cur_fun_defs_by_mod);
        fun_defs.extend(cur_fun_defs);
    }

    let mut exprs: VecDeque<Expr<Symbol,SymbolicType>> = VecDeque::new();
    for md in modules.iter() {
        match &md.expr {
            None => {},
            Some(expr) => {

            }
        }
    }

    Ok(SymbolicProgram {
        user_types,
        class_defs,
        fun_defs,
        exprs
    })
}

fn find_type_symbol(name: &String, range: Span, ids: &VecDeque<SID>, defs: &HashMap<SID, SymbolicAbstDef>) -> Result<SID, Report> {
    for sid in ids.iter() {
        let sym = &defs[sid].name;
        if sym.name == *name {
            return Ok(sym.id);
        }
    }
    return error!(
        range,
        format!("Could not find an abstract class named `{}`.", name)
    );
}

fn resolve_type (
    arg: &NominalType,
    cur_mod: &String,
    range: Span,
    ids_by_mod: &HashMap<&String, VecDeque<SID>>,
    class_defs: &HashMap<SID, SymbolicAbstDef>
) -> Result<SymbolicType, Report> {
    use rac_ast::NominalType::*;
    use SymbolicType as ST;
    match arg {
        IntType => Ok(ST::IntType),
        BoolType => Ok(ST::BoolType),
        StringType => Ok(ST::StringType),
        UnitType => Ok(ST::UnitType),
        IdType(qn) => match qn.owner.clone() {
            None => {
                let sid = find_type_symbol(&qn.name, range, &ids_by_mod[cur_mod], class_defs)?;
                Ok(ST::ClassType(Symbol::new(&qn.name, sid)))
            }
            Some(parent) => match ids_by_mod.get(&parent) {
                Some(ids) => {
                    let sid = find_type_symbol(&qn.name, range, ids, class_defs)?;
                    Ok(ST::ClassType(Symbol::new(&qn.name, sid)))
                }
                None => error!(range, format!("Type `{}` could not be found in the module `{}`.", qn.name, parent))
            }
        },
    }
}

fn contains_with_name(lst: &VecDeque<Symbol>, nme: &String) -> bool {
    for item in lst.iter() {
        if item.name == *nme {
            return true;
        }
    }
    return false;
}
