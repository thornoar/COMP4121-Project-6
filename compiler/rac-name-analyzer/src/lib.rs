use rac_ast::{
    Expr, Name, NominalModule, SID, Symbol, SymbolGenerator, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef, SymbolicProgram, Type
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
    let mut user_types: HashMap<MID, HashMap<SID, SymbolicAbstDef>> = HashMap::new();
    let mut class_defs: HashMap<MID, HashMap<SID, SymbolicClassDef>> = HashMap::new();
    let mut fun_defs: HashMap<MID, HashMap<SID, SymbolicFunDef>> = HashMap::new();
    let mut mod_ids: HashMap<&String, MID> = HashMap::new();

    // Discovering user types
    for md in modules.iter() {
        mod_ids.insert(&md.name, md.id);
        let mut cur_types: HashMap<SID, SymbolicAbstDef> = HashMap::new();
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

            cur_types.insert(sym_name.id, SymbolicAbstDef {
                name: sym_name,
                type_vars: sym_type_vars,
                range: def.range,
            });
        }
        user_types.insert(md.id, cur_types);
    }

    // Discovering class definitions
    for md in modules.iter() {
        let mut cur_cls_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
        for def in md.class_defs.iter() {
            // Check if the class is already defined
            check_unique!(def.name, cur_cls_defs, format!("A case class named `{}` is already defined.", def.name));
            
            // Generate the name
            let sym_name = sg.fresh(def.name.clone());

            // Resolve the parent
            let parent_id = find_type_symbol(&def.parent, def.range, &user_types[&md.id])?;
            let sym_parent = Symbol { name: def.parent.clone(), id: parent_id };

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ, s) in def.args.iter() {
                let sym_typ = resolve_type(typ, *s, &mod_ids, &user_types)?;
                let sym_name = sg.fresh(name.clone());
                sym_args.push_back((sym_name, sym_typ, *s));
            }

            cur_cls_defs.insert(sym_name.id, SymbolicClassDef {
                name: sym_name,
                args: sym_args,
                parent: sym_parent,
                range: def.range
            });
        }
        class_defs.insert(md.id, cur_cls_defs);
    }

    // Discovering function definitions
    for md in modules.iter() {
        let mut cur_fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();
        for def in md.fun_defs.iter() {
            // Check if the function is already defined
            check_unique!(def.name, cur_fun_defs, format!("A function `{}` is already defined.", def.name));
            // if cur_fun_defs.contains_key(&def.name) {
            //     return error!(
            //         def.range,
            //         format!("A function named `{}` is already defined.", def.name)
            //     );
            // }

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
                let sym_typ = resolve_type(typ, *s, &mod_ids, &user_types)?;
                let sym_name = sg.fresh(name.clone());
                sym_args.push_back((sym_name, sym_typ, *s));
            }
        
            // Resolve the return type
            let sym_rt = resolve_type(&def.rt.0, def.rt.1, &mod_ids, &user_types)?;

            // Resolve the body
            let sym_body = todo!();

            cur_fun_defs.insert(sym_name.id, SymbolicFunDef {
                name: sym_name,
                type_vars: sym_type_vars,
                args: sym_args,
                rt: sym_rt,
                body: sym_body,
                range: def.range
            });
        }

        fun_defs.insert(md.id, cur_fun_defs);
    }

    let mut exprs: VecDeque<Expr<Symbol>> = VecDeque::new();
    for md in modules.iter() {
        match &md.expr {
            None => {},
            Some(expr) => {

            }
        }
    }
    
    // let mut final_types = HashMap::new();
    // for mp in user_types.values() {
    //     for def in mp.values() {
    //         final_types.insert(def.name.id, *def);
    //     }
    // }
    //
    // let mut final_classes = HashMap::new();
    // for mp in class_defs.values() {
    //     for def in mp.values() {
    //         final_classes.insert(def.name.id, *def);
    //     }
    // }
    //
    // let mut final_functions = HashMap::new();
    // for mp in fun_defs.values() {
    //     for def in mp.values() {
    //         final_functions.insert(def.name.id, *def);
    //     }
    // }

    // Ok(SymbolicProgram {
    //     user_types: user_types,
    //     class_defs: class_defs,
    //     fun_defs: fun_defs,
    //     exprs: exprs
    // })
    todo!()
}

fn find_type_symbol(name: &String, range: Span, defs: &HashMap<SID, SymbolicAbstDef>) -> Result<SID, Report> {
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

fn resolve_type (arg: &Type<Name>, range: Span, mod_ids: &HashMap<&String, MID>, types: &HashMap<MID, HashMap<SID, SymbolicAbstDef>>) -> Result<Type<Symbol>, Report> {
    use rac_ast::Type::*;
    match arg {
        IntType => Ok(IntType),
        BoolType => Ok(BoolType),
        StringType => Ok(StringType),
        UnitType => Ok(UnitType),
        ClassType(qn) => match qn.owner.clone() {
            None => {
                let sid = find_type_symbol(&qn.name, range, &types[&range.tag])?;
                Ok(ClassType(Symbol::new(&qn.name, sid)))
            }
            Some(parent) => match mod_ids.get(&parent) {
                Some(mid) => 
                    match types.get(mid) {
                    Some(mp) => {
                        let sid = find_type_symbol(&qn.name, range, &mp)?;
                        Ok(ClassType(Symbol::new(&qn.name, sid)))
                    }
                    None => error!(range, format!("Type `{}` could not be found in the module `{}`.", qn.name, parent))
                },
                None => error!(range, format!("Module `{}` could not be found.", parent))
            }
        },
        Var(qn) => todo!()
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
