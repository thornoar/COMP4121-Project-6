use rac_ast::{
    Name, NominalModule, SID, Symbol, SymbolKind as SK, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef, SymbolicProgram, Type
};
use rac_diagnostics::{Report, Stage};
use std::collections::{HashMap, VecDeque};

macro_rules! error {
    ($span:expr, $msg:expr) => {
        Err(Report {
            stage: Stage::Resolving,
            range: $span,
            msg: String::from($msg),
        })
    };
}

pub fn resolve(modules: &VecDeque<NominalModule>) -> Result<SymbolicProgram, Report> {
    let mut user_types: HashMap<&String, HashMap<&String, SymbolicAbstDef>> = HashMap::new();
    let mut class_defs: HashMap<&String, HashMap<&String, SymbolicClassDef>> = HashMap::new();
    let mut fun_defs: HashMap<&String, HashMap<&String, SymbolicFunDef>> = HashMap::new();
    let mut free_id = 0;

    macro_rules! fresh_sym {
        ($name:expr, $kind:expr) => {{
            let sym = Symbol {
                name: $name.clone(),
                kind: $kind,
                id: free_id,
            };
            free_id += 1;
            sym
        }};
    }

    // let fresh_symbol = |name, kind| {
    // };

    // Discovering user types
    for md in modules.iter() {
        let mut cur_types = HashMap::new();
        for def in md.abstract_defs.iter() {
            if cur_types.contains_key(&def.name) {
                return error!(
                    def.range,
                    format!("An abstract class named `{}` is already defined.", def.name)
                );
            }
            cur_types.insert(&def.name, SymbolicAbstDef {
                name: fresh_sym!(def.name, SK::Type),
                range: def.range,
            });
        }
        user_types.insert(&md.name, cur_types);
    }

    // Discovering class definitions
    for md in modules.iter() {
        let mut cur_cls_defs = HashMap::new();
        for def in md.class_defs.iter() {
            let sym_parent = match user_types[&md.name].get(&def.parent) {
                Some(df) => df.name.clone(),
                None => {
                    return error!(
                        def.range,
                        format!("Could not find an abstract class named `{}`.", def.parent)
                    );
                }
            };
            
            if cur_cls_defs.contains_key(&def.name) {
                return error!(
                    def.range,
                    format!("A case class named `{}` is already defined.", def.name)
                );
            }

            let mut sym_args = VecDeque::new();
            for (name, typ) in def.args.iter() {
                let sym_typ = resolve_type(typ, &md.name, &user_types)?;
                let sym_name = fresh_sym!(name, SK::Field);
                sym_args.push_back((sym_name, sym_typ));
            }

            cur_cls_defs.insert(&def.name, SymbolicClassDef {
                name: fresh_sym!(def.name, SK::Constructor),
                args: sym_args,
                parent: sym_parent,
                range: def.range
            });
        }
        class_defs.insert(&md.name, cur_cls_defs);
    }

    // Discovering function definitions
    for md in modules.iter() {
        let mut cur_fun_defs = HashMap::new();
        for def in md.fun_defs.iter() {
            if cur_fun_defs.contains_key(&def.name) {
                return error!(
                    def.range,
                    format!("A function named `{}` is already defined.", def.name)
                );
            }

            let mut sym_args = VecDeque::new();
            for (name, typ) in def.args.iter() {
                let sym_typ = resolve_type(typ, &md.name, &user_types)?;
                let sym_name = fresh_sym!(name, SK::Field);
                sym_args.push_back((sym_name, sym_typ));
            }
        
            let sym_rt = resolve_type(&def.rt, &md.name, &user_types)?;

            let sym_body = todo!();
        }

        fun_defs.insert(&md.name, cur_fun_defs);
    }

    todo!()
}

fn resolve_type (arg: &Type<Name>, cur_mod: &String, types: &HashMap<&String, HashMap<&String, SymbolicAbstDef>>) -> Result<Type<Symbol>, Report> {
    use rac_ast::Type::*;
    match arg {
        IntType(s) => Ok(IntType(*s)),
        BoolType(s) => Ok(BoolType(*s)),
        StringType(s) => Ok(StringType(*s)),
        UnitType(s) => Ok(UnitType(*s)),
        ClassType(qn, s) => match qn.owner.clone() {
            None => match types[cur_mod].get(&qn.name) {
                Some(df) => Ok(ClassType(df.name.clone(), *s)),
                None => error!(*s, format!("Type `{}` could not be found in the current module.", qn.name))
            },
            Some(parent) => match types.get(&parent) {
                Some(mp) => match mp.get(&qn.name) {
                    Some(df) => Ok(ClassType(df.name.clone(), *s)),
                    None => error!(*s, format!("Type `{}` could not be found in the module `{}`.", qn.name, parent))
                },
                None => error!(*s, format!("Module `{}` could not be found.", parent))
            }
        },
        Variable(qn, s) => todo!()
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
