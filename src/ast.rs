use tree_sitter::TreeCursor;

pub fn visit_all_children(cursor: &mut TreeCursor, visit_cb: &mut dyn FnMut(&mut TreeCursor)) {
    visit_cb(cursor);
    if cursor.goto_first_child() {
        visit_all_children(cursor, visit_cb);
    } else {
        return;
    }
    loop {
        if !cursor.goto_next_sibling() {
            cursor.goto_parent();
            break;
        }
        visit_all_children(cursor, visit_cb);
    }
}
