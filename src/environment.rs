use crate::environment::scope::Scope;
use crate::type_var::{Place, TypeVar};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod scope;

pub struct Environment {
    /// stack of scopes
    live_scopes: Vec<Rc<RefCell<Scope>>>,
    /// hold all scopes that have been used
    scopes: HashMap<String, Rc<RefCell<Scope>>>,
}

/// Track variables, places and their types
impl Environment {
    pub fn new(name: &str) -> Self {
        let scopes = HashMap::new();
        let live_scopes = Vec::<Rc<RefCell<Scope>>>::new();
        let mut env = Environment {
            live_scopes,
            scopes,
        };
        env.create_scope(name);
        env
    }

    // insert into current scope
    pub fn insert_binding(&mut self, pl: Place, ty: TypeVar) {
        if let Some(scope) = self.live_scopes.last() {
            scope.borrow_mut().insert_binding(pl, ty);
        }
    }

    pub fn lookup_binding(&self, pl: &Place) -> Option<TypeVar> {
        for scope in self.live_scopes.iter().rev() {
            if let Some(ty) = scope.borrow().lookup_place(pl) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn insert_var(&mut self, var: &str, pl: Place) {
        if let Some(scope) = self.live_scopes.last() {
            scope.borrow_mut().insert_var(var, pl);
        }
    }

    pub fn lookup_var(&self, var: &str) -> Option<Place> {
        for scope in self.live_scopes.iter().rev() {
            if let Some(pl) = scope.borrow().lookup_var(var) {
                return Some(pl.clone());
            }
        }
        None
    }

    pub fn var_type(&self, var: &str) -> Option<TypeVar> {
        self.lookup_var(var).and_then(|p| self.lookup_binding(&p))
    }

    fn create_scope(&mut self, name: &str) {
        let new_scope = Rc::new(RefCell::new(Scope::new(name)));
        self.scopes.insert(name.to_owned(), new_scope.clone());
        self.live_scopes.push(new_scope.clone());
    }

    pub fn enter_scope(&mut self, name: &str) -> ScopeGuard<'_>{
        if let Some(sc) = self.scopes.get(name).cloned() {
            self.live_scopes.push(sc);
        } else {
            self.create_scope(name);
        };
        
        ScopeGuard{on_drop: &mut || {self.leave_scope();}}
    }

    pub(self) fn leave_scope(&mut self) {
        self.live_scopes.pop();
    }

    pub fn pretty_print(&self) {
        for (name, scope) in &self.scopes {
            println!("{} {}", name, scope.borrow());
        }
    }
}

pub struct ScopeGuard<'a> {
    //env: &'a mut Environment,
    on_drop: &'a mut dyn FnMut(),
}

impl Drop for ScopeGuard <'_>{
    fn drop(&mut self){
        println!("leaving scope");
        (self.on_drop)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_new() {
        let e = Environment::new("module_name");
        assert_eq!(1, e.live_scopes.len());
        assert_eq!(1, e.scopes.len());
    }

    #[test]
    fn insert_one_level() {
        let mut e = Environment::new("module_name");
        let pl = Place {
            name: "a".to_owned(),
            row: 1,
            column: 3,
        };
        let ty = TypeVar::String();
        e.insert_binding(pl.clone(), ty.clone());

        let res = e.lookup_binding(&pl).unwrap();

        assert_eq!(res, ty);
    }

    #[test]
    fn insert_two_level() {
        let mut e = Environment::new("module_name");
        let pl = Place {
            name: "a".to_owned(),
            row: 1,
            column: 3,
        };
        let ty = TypeVar::String();
        e.insert_binding(pl.clone(), ty.clone());
        e.enter_scope("next_level");
        let res = e.lookup_binding(&pl).unwrap();

        assert_eq!(res, ty);
    }

    #[test]
    fn insert_enter_leave_renter() {
        let mut e = Environment::new("module_name");
        let pl = Place {
            name: "a".to_owned(),
            row: 1,
            column: 3,
        };
        let ty = TypeVar::String();
        e.insert_binding(pl.clone(), ty.clone());
        let _g = e.enter_scope("next_level");

        let pl2 = Place {
            name: "b".to_owned(),
            row: 6,
            column: 8,
        };
        let ty2 = TypeVar::Any;
        e.insert_binding(pl2.clone(), ty2.clone());
        let res = e.lookup_binding(&pl2).unwrap();

        assert_eq!(res, ty2);

        e.leave_scope();
        let res = e.lookup_binding(&pl2);
        assert_eq!(res, None);

        _ = e.enter_scope("next_level");
        let res = e.lookup_binding(&pl2).unwrap();

        assert_eq!(res, ty2)
    }
}
