use std::collections::{HashMap, VecDeque};

use rac_ast::{SID, Symbol};

pub struct TypeTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub type_defs: &'a HashMap<String, HashMap<String, SID>>,
}

impl<'a> TypeTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        type_defs: &'a HashMap<String, HashMap<String, SID>>,
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
    pub class_defs: &'a HashMap<String, HashMap<String, SID>>,
    pub fun_defs: &'a HashMap<String, HashMap<String, SID>>,
}

impl<'a> CallTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        class_defs: &'a HashMap<String, HashMap<String, SID>>,
        fun_defs: &'a HashMap<String, HashMap<String, SID>>,
    ) -> Self {
        Self {
            cur_mod,
            class_defs,
            fun_defs,
        }
    }
}

pub struct SymbolTable<'a> {
    pub cur_mod: &'a String,
    pub type_vars: &'a VecDeque<Symbol>,
    pub type_defs: &'a HashMap<String, HashMap<String, SID>>,
    pub class_defs: &'a HashMap<String, HashMap<String, SID>>,
    pub fun_defs: &'a HashMap<String, HashMap<String, SID>>,
    pub fname: Option<(String, SID)>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(
        cur_mod: &'a String,
        type_vars: &'a VecDeque<Symbol>,
        type_defs: &'a HashMap<String, HashMap<String, SID>>,
        class_defs: &'a HashMap<String, HashMap<String, SID>>,
        fun_defs: &'a HashMap<String, HashMap<String, SID>>,
        fname: Option<(String, SID)>,
    ) -> Self {
        Self {
            cur_mod,
            type_vars,
            type_defs,
            class_defs,
            fun_defs,
            fname,
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
