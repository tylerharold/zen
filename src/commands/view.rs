use crate::editor::{Editor, Position};

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
