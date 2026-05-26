use rac_ast::{
    Expr, Name, NominalModule, SID, Symbol, SymbolKind as SK, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef, SymbolicProgram, Type
};
use rac_diagnostics::{Report, Span, Stage};
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
    // Temporary maps. The first key is the module name, the second is the type/class/function name
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

    // Discovering user types
    for md in modules.iter() {
        let mut cur_types = HashMap::new();
        for def in md.abstract_defs.iter() {
            // Check if type is already defined
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
            // Check if the class is already defined
            if cur_cls_defs.contains_key(&def.name) {
                return error!(
                    def.range,
                    format!("A case class named `{}` is already defined.", def.name)
                );
            }

            // Resolve the parent
            let sym_parent = match user_types[&md.name].get(&def.parent) {
                Some(df) => df.name.clone(),
                None => {
                    return error!(
                        def.range,
                        format!("Could not find an abstract class named `{}`.", def.parent)
                    );
                }
            };

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ, s) in def.args.iter() {
                let sym_typ = resolve_type(typ, *s, &md.name, &user_types)?;
                let sym_name = fresh_sym!(name, SK::Field);
                sym_args.push_back((sym_name, sym_typ, *s));
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
            // Check if the function is already defined
            if cur_fun_defs.contains_key(&def.name) {
                return error!(
                    def.range,
                    format!("A function named `{}` is already defined.", def.name)
                );
            }

            // Resolve the arguments
            let mut sym_args = VecDeque::new();
            for (name, typ, s) in def.args.iter() {
                let sym_typ = resolve_type(typ, *s, &md.name, &user_types)?;
                let sym_name = fresh_sym!(name, SK::Field);
                sym_args.push_back((sym_name, sym_typ, *s));
            }
        
            // Resolve the return type
            let sym_rt = resolve_type(&def.rt.0, def.rt.1, &md.name, &user_types)?;

            // Resolve the body
            let sym_body = todo!();

            cur_fun_defs.insert(&def.name, SymbolicFunDef {
                name: fresh_sym!(def.name, SK::Function),
                args: sym_args,
                rt: sym_rt,
                body: sym_body,
                range: def.range
            });
        }

        fun_defs.insert(&md.name, cur_fun_defs);
    }

    let mut exprs: VecDeque<Expr<Symbol>> = VecDeque::new();
    for md in modules.iter() {
        match &md.expr {
            None => {},
            Some(expr) => {

            }
        }
    }
    
    let mut final_types = HashMap::new();
    for mp in user_types.values() {
        for def in mp.values() {
            final_types.insert(def.name.id, def);
        }
    }

    let mut final_classes = HashMap::new();
    for mp in class_defs.values() {
        for def in mp.values() {
            final_classes.insert(def.name.id, def);
        }
    }

    let mut final_functions = HashMap::new();
    for mp in fun_defs.values() {
        for def in mp.values() {
            final_functions.insert(def.name.id, def);
        }
    }

    // Ok(SymbolicProgram {
    //     user_types: final_types,
    //     class_defs: final_classes,
    //     fun_defs: final_functions,
    //     exprs: exprs
    // })
    todo!()
}

fn resolve_type (arg: &Type<Name>, range: Span, cur_mod: &String, types: &HashMap<&String, HashMap<&String, SymbolicAbstDef>>) -> Result<Type<Symbol>, Report> {
    use rac_ast::Type::*;
    match arg {
        IntType => Ok(IntType),
        BoolType => Ok(BoolType),
        StringType => Ok(StringType),
        UnitType => Ok(UnitType),
        ClassType(qn) => match qn.owner.clone() {
            None => match types[cur_mod].get(&qn.name) {
                Some(df) => Ok(ClassType(df.name.clone())),
                None => error!(range, format!("Type `{}` could not be found in the current module.", qn.name))
            },
            Some(parent) => match types.get(&parent) {
                Some(mp) => match mp.get(&qn.name) {
                    Some(df) => Ok(ClassType(df.name.clone())),
                    None => error!(range, format!("Type `{}` could not be found in the module `{}`.", qn.name, parent))
                },
                None => error!(range, format!("Module `{}` could not be found.", parent))
            }
        },
        Variable(qn) => todo!()
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
