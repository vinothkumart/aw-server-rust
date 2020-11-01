use std::collections::HashMap;

use crate::QueryError;
use crate::datatype::DataType;

// TODO: Fix unwraps

struct Var {
    pub refs: u32,
    pub val: Option<DataType>,
}

pub struct VarEnv {
    vars: HashMap<String, Var>
}

impl VarEnv {
    pub fn new() -> Self {
        VarEnv { vars: HashMap::new() }
    }

    pub fn declare(&mut self, name: String) -> () {
        if !self.vars.contains_key(&name) {
            let var = Var { refs: 0, val: None };
            println!("declare {}", name);
            self.vars.insert(name, var);
        }
    }

    pub fn declare_static(&mut self, name: String, val: DataType) -> () {
        let var = Var { refs: std::u32::MAX, val: Some(val) };
        self.vars.insert(name, var);
    }

    // TODO: rename assign?
    pub fn insert(&mut self, name: String, val: DataType) -> () {
        match self.vars.get_mut(&name) {
            Some(var) => var.val = Some(val),
            None => panic!(format!("fail, not declared {}", name)), // TODO: Properly handle this
        };
    }

    pub fn add_ref(&mut self, name: &str) -> Result<(), QueryError> {
        match self.vars.get_mut(name) {
            Some(var) => {
                if var.refs != std::u32::MAX {
                    println!("add ref {}, {}", name, var.refs);
                    var.refs += 1
                }
            },
            None => return Err(QueryError::VariableNotDefined(name.to_string())),
        };
        Ok(())
    }

    pub fn take(&mut self, name: &str) -> Option<DataType> {
        let clone : bool = match self.vars.get_mut(name) {
            Some(var) => {
                println!("{}: {}", name, var.refs);
                var.refs -= 1;
                var.refs > 0
            },
            None => return None,
        };
        if clone {
            match self.vars.get(name) {
                Some(var) => Some(var.val.as_ref().unwrap().clone()),
                None => return None,
            }
        } else {
            match self.vars.remove(name) {
                Some(var) => Some(var.val.unwrap()),
                None => return None,
            }
        }
    }

    // TODO: Remove this completely, only needed for TIMEINTERVAL
    pub fn deprecated_get(&self, var: &str) -> Option<DataType> {
        match self.vars.get(var) {
            Some(var) => Some(var.val.as_ref().unwrap().clone()),
            None => None,
        }
    }
}
