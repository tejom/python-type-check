use crate::environment::scope::{Scope, ScopeStack};
use crate::type_var::{Place, TypeVar};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod scope;

pub struct Environment {
    /// stack of scopes
    //live_scopes: Vec<Rc<RefCell<Scope>>>,
    live_scopes: Rc<RefCell<ScopeStack>>,
    /// hold all scopes that have been used
    scopes: HashMap<String, Rc<RefCell<Scope>>>,
}

/// Track variables, places and their types
impl Environment {
    pub fn new(name: &str) -> Self {
        let scopes = HashMap::new();
        //let live_scopes = Vec::<Rc<RefCell<Scope>>>::new();
        let mut env = Self {
            live_scopes: Rc::new(RefCell::new(ScopeStack::new())),
            scopes,
        };
        env.create_scope(name);
        env
    }

    /// insert into current scope
    pub fn insert_binding(&mut self, pl: Place, ty: TypeVar) {
        if let Some(scope) = self.live_scopes.borrow().last() {
            scope.borrow_mut().insert_binding(pl, ty);
        }
    }

    pub fn lookup_binding(&self, pl: &Place) -> Option<TypeVar> {
        for scope in self.live_scopes.borrow().iter().rev() {
            if let Some(ty) = scope.borrow().lookup_place(pl) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn insert_var(&mut self, var: &str, pl: Place) {
        if let Some(scope) = self.live_scopes.borrow().last() {
            scope.borrow_mut().insert_var(var, pl);
        }
    }

    /// iterate through the live scopes looking for the var
    pub fn lookup_var(&self, var: &str) -> Option<Place> {
        for scope in self.live_scopes.borrow().iter().rev() {
            if let Some(pl) = scope.borrow().lookup_var(var) {
                return Some(pl.clone());
            }
        }
        None
    }

    /// Get the TypeVar for an Identifier like a variable or function name
    pub fn var_type(&self, var: &str) -> Option<TypeVar> {
        self.lookup_var(var).and_then(|p| self.lookup_binding(&p))
    }

    fn create_scope(&mut self, name: &str) {
        let new_scope = Rc::new(RefCell::new(Scope::new(name)));
        self.scopes.insert(name.to_owned(), new_scope.clone());
        self.live_scopes.borrow_mut().push(new_scope.clone());
    }

    /// Add a new scope to the stack by either creating it or loading an existing one
    /// ScopeGuard when dropped will pop the latest scope from the stack
    pub fn enter_scope(&mut self, name: &str) -> ScopeGuard {
        if let Some(sc) = self.scopes.get(name).cloned() {
            self.live_scopes.borrow_mut().push(sc);
        } else {
            self.create_scope(name);
        };

        ScopeGuard {
            stack: self.live_scopes.clone(),
        }
    }

    #[allow(dead_code)]
    pub(self) fn leave_scope(&mut self) {
        self.live_scopes.borrow_mut().pop();
    }

    pub fn pretty_print(&self) {
        for (name, scope) in &self.scopes {
            println!("{} {}", name, scope.borrow());
        }
    }
}

/// Returned when a scope is entered. When Dropped it'll pop one scope from the stack
#[clippy::has_significant_drop]
pub struct ScopeGuard {
    stack: Rc<RefCell<ScopeStack>>,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        self.stack.borrow_mut().pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_new() {
        let e = Environment::new("module_name");
        assert_eq!(1, e.live_scopes.borrow().len());
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

        let _g2 = e.enter_scope("next_level");
        let res = e.lookup_binding(&pl2).unwrap();

        assert_eq!(res, ty2)
    }
}
