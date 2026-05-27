use std::collections::{HashMap, VecDeque};

use rac_ast::{SID, Symbol, SymbolicAbstDef};

pub struct TypeEnv<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub type_defs: &'a HashMap<SID, SymbolicAbstDef>,
}

impl<'a> TypeEnv<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        type_defs: &'a HashMap<SID, SymbolicAbstDef>,
    ) -> Self {
        Self { cur_mod, type_vars, types_by_mod, type_defs }
    }
}

pub struct FullEnv<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub binds: &'a mut HashMap<String, SID>,
    pub types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
    pub type_defs: &'a HashMap<SID, SymbolicAbstDef>,
}

impl<'a> FullEnv<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        binds: &'a mut HashMap<String, SID>,
        types_by_mod: &'a HashMap<&'a String, VecDeque<SID>>,
        type_defs: &'a HashMap<SID, SymbolicAbstDef>,
    ) -> Self {
        Self { cur_mod, type_vars, binds, types_by_mod, type_defs }
    }
}

impl<'a> From<&FullEnv<'a>> for TypeEnv<'a> {
    fn from(value: &FullEnv<'a>) -> Self {
        Self::new(value.cur_mod, value.type_vars, value.types_by_mod, value.type_defs)
    }
}
