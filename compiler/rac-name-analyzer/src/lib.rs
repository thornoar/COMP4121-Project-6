use std::collections::{HashMap, VecDeque};
use rac_ast::{NominalModule, SID, Symbol, SymbolKind as SK, SymbolicClassDef, SymbolicFunDef, SymbolicProgram};
use rac_diagnostics::{Stage, Report};

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
    let mut user_types: VecDeque<Symbol> = VecDeque::new();
    let mut class_defs: HashMap<SID, SymbolicClassDef> = HashMap::new();
    let mut fun_defs: HashMap<SID, SymbolicFunDef> = HashMap::new();
    let mut free_id = 0;

    macro_rules! fresh_sym {
        ($name:expr, $kind:expr) => {{
            let sym = Symbol { name: $name.clone(), kind: $kind, id: free_id };
            free_id += 1;
            sym
        }};
    }

    // let fresh_symbol = |name, kind| {
    // };

    // Discovering user types
    for md in modules.iter() {
        for def in md.abstract_defs.iter() {
            if !contains_with_name(&user_types, &def.name) {
                return error!(def.range, format!("An abstract class named `{}` is already defined.", def.name))
            }
            user_types.push_back(fresh_sym!(def.name, SK::Type));
        }
    }

    // Discovering class definitions
    for md in modules.iter() {
        for def in md.class_defs.iter() {
            
        }
    }


    todo!()
}

fn contains_with_name(lst: &VecDeque<Symbol>, nme: &String) -> bool {
    for item in lst.iter() {
        if item.name == *nme {
            return true
        }
    }
    return false
}
