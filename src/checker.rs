use crate::{
    environment::Environment,
    type_var::{Place, TypeVar},
    visit_all_children,
};
use colored::Colorize;
use log::debug;
use std::{cmp::max, vec};
use tree_sitter::{Node, TreeCursor};

#[derive(Debug, Clone)]
pub struct CheckErr {
    msg: String,
    start_place: Place,
    end_place: Option<Place>,
}

impl std::fmt::Display for CheckErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CheckErr: {} @ {} to {:?}",
            self.msg, self.start_place, self.end_place
        )
    }
}

impl std::error::Error for CheckErr {}

impl CheckErr {
    pub fn new(msg: &str, start_place: Place, end_place: Option<Place>) -> Self {
        CheckErr {
            msg: msg.to_owned(),
            start_place,
            end_place,
        }
    }
}

pub struct Checker<'a> {
    //_env: HashMap<String, Place>,
    env: Environment,
    errors: Vec<CheckErr>,
    src: &'a str,
    file_name: &'a str,
}

impl<'a> Checker<'a> {
    pub fn new(src: &'a str, file_name: &'a str) -> Self {
        Checker {
            env: Environment::new(file_name),
            errors: Vec::<CheckErr>::new(),
            src,
            file_name,
        }
    }

    pub fn check_module(&mut self, cursor: &mut TreeCursor) {
        println!("Checking {}...", self.file_name);
        visit_all_children(cursor, &mut |cur| {
            self.check_visit(cur);
        });
        // self.print_bindings();
        // self.print_env();
        self.env.pretty_print();
        self.print_errors();
    }

    pub fn check_visit(&mut self, cursor: &mut TreeCursor) {
        match cursor.node().kind() {
            "expression_statement" => {
                debug!("EXPR_STMT   -");
            }
            "assignment" => {
                debug!(
                    "DEFINE      - {}",
                    cursor.node().child_by_field_name("left").unwrap()
                );
                self.check_assignment(cursor, self.src)
            }
            "binary_operator" => {
                self.check_binop(cursor).unwrap_or_else(|err| {
                    debug!("Type Error {}", err);
                    self.errors.push(err);
                });
            }
            "function_definition" => {
                self.check_function_def(cursor);
            }
            "call" => {
                self.check_fn_call(cursor).unwrap_or_else(|err| {
                    self.errors.push(err);
                });
            }
            "module" => {} // nodes to ignore
            _ => {
                debug!("UNSEEN NODE - {}", cursor.node());
            }
        }
    }

    pub fn infer_type_for_node(&mut self, node: &tree_sitter::Node) -> Option<TypeVar> {
        let infered_node_type = match node.kind() {
            "identifier" => {
                let node_id = node
                    .utf8_text(self.src.as_bytes())
                    .expect("couldnt decode id");
                self.env
                    .var_type(node_id)
                    .expect("couldnt find type for var")
            }
            "call" => {
                println!("call> {}", node);
                let sig = self.infer_type_for_node(
                    &(node
                        .child_by_field_name("function")
                        .expect("getting fn name")),
                )?;
                println!("found sig? {}", sig);
                if let TypeVar::Function(_, _, ret_val) = sig {
                    if ret_val.len() == 1 {
                        ret_val.first().cloned()?
                    } else {
                        TypeVar::Union(ret_val)
                    }
                } else {
                    TypeVar::None
                }
            }
            "integer" => {
                let int_val: usize = node
                    .utf8_text(self.src.as_bytes())
                    .map(|i| i.parse().expect("error parsing"))
                    .expect("issue getting int value");
                TypeVar::Integer(int_val)
            }
            "string" => TypeVar::String(),
            "return_statement" => {
                if let Some(n) = node.named_child(0) {
                    self.infer_type_for_node(&n)
                        .expect("invalid return statement")
                } else {
                    TypeVar::None
                }
            }
            "binary_operator" => {
                TypeVar::BinOp(Place::from_ts_point("binop", node.start_position()))
            }
            _ => TypeVar::Var(Place::exp_from_ts_point(node.start_position())),
        };
        Some(infered_node_type)
    }

    pub fn infer_fn_body(&mut self, node: &tree_sitter::Node) -> Vec<TypeVar> {
        let mut return_statement_types: Vec<TypeVar> = Vec::new();

        visit_all_children(&mut node.walk(), &mut |c| {
            if c.node().kind() == "return_statement" {
                println!("{}", c.node());
                return_statement_types.push(
                    self.infer_type_for_node(&c.node())
                        .expect("error infering return"),
                )
            };
        });

        return_statement_types
    }

    pub fn check_function_def(&mut self, cursor: &mut TreeCursor) {
        println!("{}", cursor.node());

        let mut param_types: Vec<TypeVar> = Vec::new();

        let fn_name = cursor
            .node()
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(self.src.as_bytes()).ok())
            .expect("no fn name");
        let fn_place = Place::from_ts_point(fn_name, cursor.node().start_position());

        let param_node = cursor
            .node()
            .child_by_field_name("parameters")
            .expect("no parameters");

        let body_node = cursor
            .node()
            .child_by_field_name("body")
            .expect("error getting fn body");

        self.env.enter_scope(fn_name);
        for node in param_node.named_children(&mut param_node.walk()) {
            println!("node {}", node);
            param_types.push(TypeVar::Any);
            let p_id = node
                .utf8_text(self.src.as_bytes())
                .expect("error getting param id");
            let param_place = Place::from_ts_point(p_id, node.start_position());
            self.env.insert_binding(param_place.clone(), TypeVar::Any);
            self.env.insert_var(p_id, param_place.clone());
        }
        println!("{}", cursor.node());
        let return_types = self.infer_fn_body(&body_node);

        println!("Handling fn {} {}", fn_name, param_node);
        self.env.leave_scope();
        self.env.insert_binding(
            fn_place.clone(),
            TypeVar::Function(fn_place.clone(), param_types, return_types),
        );
        self.env.insert_var(fn_name, fn_place.clone());
    }

    pub fn check_fn_call(&mut self, cursor: &mut TreeCursor) -> Result<(), CheckErr> {
        println!("fn call {}", cursor.node());
        let fn_call_node = cursor.node();
        let fn_name = cursor
            .node()
            .child_by_field_name("function")
            .and_then(|n| n.utf8_text(self.src.as_bytes()).ok())
            .expect("error getting fn name");

        self.env.enter_scope(fn_name);
        let fn_sig = self.env.var_type(fn_name);
        let fn_args_list = cursor
            .node()
            .child_by_field_name("arguments")
            .expect("error getting args");

        if let Some(TypeVar::Function(_, params, _)) = fn_sig {
            println!("found fnsig {:?} p {}", params, fn_args_list);
            let mut param_cursor = fn_args_list.walk();

            // convert all of the ast nodes for args to types
            let arg_types: Vec<(Node, Result<TypeVar, CheckErr>)> = fn_args_list
                .named_children(&mut param_cursor)
                .map(|n| {
                    println!("n:::{}", n);
                    (
                        n,
                        self.infer_type_for_node(&n).ok_or_else(|| {
                            CheckErr::new(
                                "no type available",
                                Place::from_ts_point("fnarg", n.start_position()),
                                None,
                            )
                        }),
                    )
                })
                .collect();

            // check the same amount of args was used for the fn signature
            if arg_types.len() != params.len() {
                return Err(CheckErr::new(
                    &format!(
                        "Fn called with {} args expected {}",
                        arg_types.len(),
                        params.len()
                    ),
                    Place::from_ts_point("fncall", fn_call_node.start_position()),
                    Some(Place::from_ts_point("fncall", fn_call_node.end_position())),
                ));
            }
            // compare function params and args
            for idx in 0..arg_types.len() {
                if let Some((n, Ok(arg_ty))) = arg_types.get(idx) {
                    let b = params.get(idx).unwrap();
                    if !arg_ty.type_check(b) {
                        self.errors.push(CheckErr::new(
                            &format!("Type mismatch {},{}", arg_ty, b),
                            Place::from_ts_point("arg", n.start_position()),
                            Some(Place::from_ts_point("arg", n.end_position())),
                        ));
                    }
                } else if let Some((_, Err(e))) = arg_types.get(idx) {
                    self.errors.push(e.clone());
                }
            }
        };

        Ok(())
    }

    pub fn check_binop(&mut self, cursor: &mut TreeCursor) -> Result<(), CheckErr> {
        let node = cursor.node();
        let binop_place = Place::from_ts_point("binop", node.start_position());

        let arg1 = node.child_by_field_name("left").expect("error getting lhs");
        let arg2 = node
            .child_by_field_name("right")
            .expect("error getting rhs");

        let a1_place = Place::from_ts_point("arg1", arg1.start_position());
        let a1_type = self.infer_type_for_node(&arg1).expect("no type infered");

        let a2_place = Place::from_ts_point("arg2", arg2.start_position()).clone();
        let a2_type = self.infer_type_for_node(&arg2).expect("no type infered");

        let return_place = Place::from_ts_point("return", node.start_position());
        let return_type = match (&a1_type, &a2_type) {
            (TypeVar::Integer(a), TypeVar::Integer(b)) => TypeVar::Integer(a + b),
            (TypeVar::String(), TypeVar::String()) => TypeVar::String(),
            err => {
                debug!("types not handled {:?}", err);
                return Err(CheckErr::new(
                    &format!("Invalid types {:?} for BinOp", err),
                    binop_place,
                    Some(Place::from_ts_point("binop", node.end_position())),
                ));
            }
        };

        let binop_type = TypeVar::Call(
            binop_place.clone(),
            vec![a1_type.clone(), a2_type.clone()],
            vec![return_type.clone()],
        );

        self.env
            .insert_binding(binop_place.clone(), binop_type.clone());
        self.env.insert_binding(a1_place.clone(), a1_type.clone());
        self.env.insert_binding(a2_place.clone(), a2_type);
        self.env
            .insert_binding(return_place.clone(), return_type.clone());
        Ok(())
    }

    pub fn check_assignment(&mut self, cursor: &mut TreeCursor, source: &str) {
        let node = cursor.node();
        let lhs = node
            .child_by_field_name("left")
            .expect("No lhs in assignment");
        let id = lhs
            .utf8_text(source.as_bytes())
            .expect("couldnt decode value");
        let left_place = Place::from_ts_point(id, lhs.start_position());
        let rhs = node
            .child_by_field_name("right")
            .expect("No rhs in assignment");
        let rhs_type = self.infer_type_for_node(&rhs).expect("couldnt infer rhs");

        debug!("assignment lhs {} -> {}", left_place, rhs_type);
        self.env.insert_binding(left_place.clone(), rhs_type);
        self.env.insert_var(id, left_place.clone());
    }

    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            println!("✅ {}", "Type Checks Passed!".bright_green());
            return;
        }
        let heading = format!("{} Error(s) found:", self.errors.len()).bright_magenta();
        println!("{}", heading);
        for err in &self.errors {
            let line = err.start_place.row;
            let col = err.start_place.column;

            // line needs +1 to account for zero index
            println!(
                "[{}] {}:{}:{} {} ",
                "Error".bright_red(),
                self.file_name,
                line + 1,
                col,
                err.msg,
            );
            // print context
            let ctx_line_start = max(0, line as i64 - 2);
            let prefix_len = err.start_place.row.to_string().len();
            for l in ctx_line_start..(line + 1) as i64 {
                let prefix = format!("{} | ", l + 1).cyan();
                println!(
                    "{}{}",
                    prefix,
                    self.src.lines().nth(l as usize).unwrap().cyan()
                );
            }

            if let Some(end_place) = &err.end_place {
                let num_carrots = end_place.column - col;

                let prefix = format!("{} | ", " ".repeat(prefix_len)).cyan();
                println!(
                    "{}{}{}",
                    prefix,
                    " ".repeat(col),
                    "^".repeat(num_carrots).bright_red()
                )
            } else {
                println!("{}{}", " ".repeat(col), "".red())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_add_error() {
        let src = "c = 1 + \"goo\"";
        let mut checker = Checker::new(src, "test.py");

        let tree = crate::ast::parse(src).expect("Issue parsing tree");

        checker.check_module(&mut tree.walk());

        assert_eq!(checker.errors.len(), 1);
    }
}
