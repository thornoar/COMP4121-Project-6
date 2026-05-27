use std::collections::{HashMap, VecDeque};

use rac_ast::{SID, Symbol, SymbolicAbstDef, SymbolicClassDef, SymbolicFunDef};

pub struct TypeTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    // pub types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub type_defs: &'a HashMap<String, HashMap<SID, SymbolicAbstDef>>,
}

impl<'a> TypeTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        // types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        type_defs: &'a HashMap<String, HashMap<SID, SymbolicAbstDef>>,
    ) -> Self {
        Self {
            cur_mod,
            type_vars,
            type_defs,
        }
    }
}

pub struct CallTable<'a> {
    pub cur_mod: &'a String,
    // pub fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
    pub fun_defs: &'a HashMap<String, HashMap<SID, SymbolicFunDef>>,
}

impl<'a> CallTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        // fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
        fun_defs: &'a HashMap<String, HashMap<SID, SymbolicFunDef>>,
    ) -> Self {
        Self { cur_mod, class_defs, fun_defs }
    }
}

// pub struct ClassEnv<'a> {
//     pub cur_mod: &'a String,
//     // pub fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
//     pub class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
// }
//
// impl<'a> ClassEnv<'a> {
//     pub fn new(
//         cur_mod: &'a String,
//         // fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
//         class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
//     ) -> Self {
//         Self { cur_mod, class_defs }
//     }
// }

pub struct SymbolTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    // pub types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub type_defs: &'a HashMap<String, HashMap<SID, SymbolicAbstDef>>,
    // pub fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
    pub fun_defs: &'a HashMap<String, HashMap<SID, SymbolicFunDef>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        // types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        type_defs: &'a HashMap<String, HashMap<SID, SymbolicAbstDef>>,
        class_defs: &'a HashMap<String, HashMap<SID, SymbolicClassDef>>,
        // fun_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        fun_defs: &'a HashMap<String, HashMap<SID, SymbolicFunDef>>,
    ) -> Self {
        Self {
            cur_mod,
            type_vars,
            type_defs,
            class_defs,
            fun_defs,
        }
    }
}

impl<'a> From<&SymbolTable<'a>> for TypeTable<'a> {
    fn from(value: &SymbolTable<'a>) -> Self {
        Self::new(value.cur_mod, value.type_vars, value.type_defs)
    }
}
impl<'a> From<&SymbolTable<'a>> for CallTable<'a> {
    fn from(value: &SymbolTable<'a>) -> Self {
        Self::new(value.cur_mod, value.class_defs, value.fun_defs)
    }
}
// impl<'a> From<&SymbolTable<'a>> for ClassEnv<'a> {
//     fn from(value: &SymbolTable<'a>) -> Self {
//         Self::new(value.cur_mod, value.class_defs)
//     }
// }
