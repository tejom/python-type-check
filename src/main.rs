use log::{trace};
use std::fs;
use tree_sitter::Parser;

mod arg;
mod ast;
mod checker;
mod pretty_printer;
mod type_var;

use crate::ast::visit_all_children;
use crate::checker::Checker;
use crate::pretty_printer::PrettyPrinter;

fn main() {
    env_logger::builder()
        .format_file(false)
        .format_timestamp(None)
        .format_source_path(false)
        .format_module_path(false)
        .format_target(false)
        .init();

    let args = crate::arg::get_args();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("Error loading Python grammar");

    let file_name = args
        .get_one::<String>("file_name")
        .expect("No file name to check");

    let source_code = fs::read_to_string(file_name).expect("error opening file");

    let tree = parser.parse(&source_code, None).unwrap();
    let root_node = tree.root_node();

    trace!("{}\n{}", &source_code, root_node);
    if args.get_flag("pretty-print") {
        PrettyPrinter::new(&source_code).print_module(&mut tree.walk());
    }
    Checker::new(&source_code, &file_name).check_module(&mut tree.walk());
}
