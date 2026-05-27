use std::collections::{HashMap, VecDeque};

use rac_ast::{SID, Symbol, SymbolicTypeDef};

pub struct TypeTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub type_syms: &'a HashMap<String, HashMap<String, SID>>,
    // pub type_defs: &'a HashMap<SID, SymbolicTypeDef>,
}

impl<'a> TypeTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        type_syms: &'a HashMap<String, HashMap<String, SID>>,
        // type_defs: &'a HashMap<SID, SymbolicTypeDef>,
    ) -> Self {
        Self {
            cur_mod,
            type_vars,
            type_syms,
            // type_defs,
        }
    }
}

pub struct CallTable<'a> {
    pub cur_mod: &'a String,
    pub class_syms: &'a HashMap<String, HashMap<String, SID>>,
    pub fun_syms: &'a HashMap<String, HashMap<String, SID>>,
}

impl<'a> CallTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        class_syms: &'a HashMap<String, HashMap<String, SID>>,
        fun_syms: &'a HashMap<String, HashMap<String, SID>>,
    ) -> Self {
        Self {
            cur_mod,
            class_syms,
            fun_syms,
        }
    }
}

pub struct SymbolTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub type_syms: &'a HashMap<String, HashMap<String, SID>>,
    // pub type_defs: &'a HashMap<SID, SymbolicTypeDef>,
    pub class_syms: &'a HashMap<String, HashMap<String, SID>>,
    pub fun_syms: &'a HashMap<String, HashMap<String, SID>>,
    pub fname: Option<(String, SID)>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        type_syms: &'a HashMap<String, HashMap<String, SID>>,
        // type_defs: &'a HashMap<SID, SymbolicTypeDef>,
        class_syms: &'a HashMap<String, HashMap<String, SID>>,
        fun_syms: &'a HashMap<String, HashMap<String, SID>>,
        fname: Option<(String, SID)>,
    ) -> Self {
        Self {
            cur_mod,
            type_vars,
            type_syms,
            // type_defs,
            class_syms,
            fun_syms,
            fname,
        }
    }
}

impl<'a> From<&SymbolTable<'a>> for TypeTable<'a> {
    fn from(value: &SymbolTable<'a>) -> Self {
        Self::new(value.cur_mod, value.type_vars, value.type_syms)
    }
}
impl<'a> From<&SymbolTable<'a>> for CallTable<'a> {
    fn from(value: &SymbolTable<'a>) -> Self {
        Self::new(value.cur_mod, value.class_syms, value.fun_syms)
    }
}
