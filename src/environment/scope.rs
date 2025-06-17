use std::collections::HashMap;

use crate::type_var::{Place, TypeVar};

pub struct Scope {
    name: String,
    /// Maps a Place in the ast/source to a type
    bindings: HashMap<Place, TypeVar>,
    /// Maps the identifier to a place of its current value
    var_place_map: HashMap<String, Place>,
}

impl Scope {
    pub fn new(name: &str) -> Self {
        Scope {
            name: name.to_owned(),
            bindings: HashMap::new(),
            var_place_map: HashMap::new(),
        }
    }

    pub fn insert_binding(&mut self, pl: Place, ty: TypeVar) {
        self.bindings.insert(pl, ty);
    }

    pub fn lookup_place(&self, pl: &Place) -> Option<TypeVar> {
        self.bindings.get(pl).cloned()
    }

    pub fn insert_var(&mut self, var: &str, pl: Place) {
        self.var_place_map.insert(var.to_owned(), pl);
    }

    pub fn lookup_var(&self, var: &str) -> Option<Place> {
        self.var_place_map.get(var).cloned()
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Scope [name: {}]---", self.name)?;
        writeln!(f, "Bindings")?;
        for (pl, ty) in &self.bindings {
            writeln!(f, "{} -> {}", pl, ty)?;
        }
        writeln!(f, "Var Place Map")?;
        for (var, pl) in &self.var_place_map {
            writeln!(f, "{} -> {}", var, pl)?;
        }
        Ok(())
    }
}
