use crate::visit_all_children;
use log::info;
use tree_sitter::TreeCursor;

pub struct PrettyPrinter<'a> {
    src: &'a str,
}

impl<'a> PrettyPrinter<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src }
    }

    pub fn print_module(&self, cursor: &mut TreeCursor) {
        info!(
            "{} num child {}",
            cursor.node().kind(),
            cursor.node().child_count()
        );
        visit_all_children(cursor, &mut |cur| {
            self.print_visit(cur);
        });
    }

    pub fn print_visit(&self, cursor: &mut TreeCursor) {
        print_cursor_node_location(cursor, self.src);
    }
}

pub fn print_cursor_node_location(cursor: &mut TreeCursor, source: &str) {
    println!(
        "{} {:?} ({}) {}",
        " ".repeat(cursor.depth() as usize),
        cursor.node().kind(),
        cursor.node().utf8_text(source.as_bytes()).unwrap(),
        cursor.depth() as usize
    );
}
