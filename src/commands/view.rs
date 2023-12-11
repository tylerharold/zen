use std::cmp;

use crate::{editor::Editor, Position};

pub fn scroll_up(editor: &mut Editor) {
    let Position { mut y, mut x } = editor.cursor_position;
    let terminal_height = editor.terminal.size().height as usize;

    y = if y > terminal_height {
        y - terminal_height
    } else {
        0
    };

    editor.cursor_position = Position { x, y }
}

pub fn scroll_down(editor: &mut Editor) {
    let Position { mut y, mut x } = editor.cursor_position;
    let terminal_height = editor.terminal.size().height as usize;
    let height = editor.document.len();

    y = if y.saturating_add(terminal_height) < height {
        y.saturating_add(terminal_height)
    } else {
        height
    };

    editor.cursor_position = Position { x, y }
}

pub fn move_start_of_document(editor: &mut Editor) {
    editor.cursor_position = Position::default();
    editor.offset = Position::default();
}

pub fn move_end_of_document(editor: &mut Editor) {
    let y = editor.document.len().saturating_sub(1);
    editor.cursor_position = Position { x: 0, y };
    editor.offset.y = cmp::max(0, y.saturating_sub(editor.terminal.size().height as usize));
}
