use crate::{
    type_var::{Place, TypeVar},
    visit_all_children,
};
use colored::Colorize;
use log::{debug, info};
use std::cmp::max;
use std::collections::HashMap;
use tree_sitter::TreeCursor;

#[derive(Debug)]
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
    binding: HashMap<Place, TypeVar>,
    env: HashMap<String, Place>,
    errors: Vec<CheckErr>,
    src: &'a str,
    file_name: &'a str,
}

impl<'a> Checker<'a> {
    pub fn new(src: &'a str, file_name: &'a str) -> Self {
        Checker {
            binding: HashMap::default(),
            env: HashMap::default(),
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
        self.print_bindings();
        self.print_env();
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
                let node_place = self.env.get(node_id).expect("identifier not defined");
                self.binding
                    .get(node_place)
                    .expect("identifer doesnt have a type")
                    .clone()
            }
            "integer" => {
                let int_val: usize = node
                    .utf8_text(self.src.as_bytes())
                    .map(|i| i.parse().expect("error parsing"))
                    .expect("issue getting int value");
                TypeVar::Integer(int_val)
            }
            "string" => TypeVar::String(),
            "binary_operator" => {
                TypeVar::BinOp(Place::from_ts_point("binop", node.start_position()))
            }
            _ => TypeVar::Var(Place::exp_from_ts_point(node.start_position())),
        };
        Some(infered_node_type)
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

        self.binding.insert(binop_place.clone(), binop_type.clone());
        self.binding.insert(a1_place.clone(), a1_type.clone());
        self.binding.insert(a2_place.clone(), a2_type);
        self.binding
            .insert(return_place.clone(), return_type.clone());
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
        self.binding.insert(left_place.clone(), rhs_type);
        self.env.insert(id.to_owned(), left_place.clone());
    }

    pub fn print_bindings(&self) {
        for (l, r) in &self.binding {
            debug!("{} -> {}", l, r);
        }
    }
    pub fn print_env(&self) {
        for (l, r) in &self.env {
            debug!("{} -> {}", l, r);
        }
    }

    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            info!("✅ {}", "Type Checks Passed!".bright_green());
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
