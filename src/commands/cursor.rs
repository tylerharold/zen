use crate::editor::Editor;
use crate::Position;

pub fn move_up(editor: &mut Editor) {
    let Position { mut y, mut x } = editor.cursor_position;

    y = y.saturating_sub(1);

    editor.cursor_position = Position { x, y }
}

pub fn move_down(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);
    let height = editor.document.len();

    if y < height {
        y = y.saturating_add(1);
    }

    editor.cursor_position = Position { x, y }
}

pub fn move_left(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    if x > 0 {
        x -= 1;
    } else if y > 0 {
        y -= 1;
        if let Some(row) = editor.document.row(y) {
            x = row.len();
        } else {
            x = 0;
        }
    }

    editor.cursor_position = Position { x, y }
}

pub fn move_right(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    let height = editor.document.len();
    let mut width = if let Some(row) = editor.document.row(y) {
        row.len()
    } else {
        0
    };

    if x < width {
        x += 1;
    } else if y < height {
        y += 1;
        x = 0;
    }

    editor.cursor_position = Position { x, y }
}

pub fn move_start_of_row(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    x = 0;

    editor.cursor_position = Position { x, y }
}

pub fn move_end_of_row(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    let mut width = if let Some(row) = editor.document.row(y) {
        row.len()
    } else {
        0
    };

    x = width;

    editor.cursor_position = Position { x, y }
}

pub fn get_cursor_position(editor: &mut Editor) -> Position {
    let Position { mut y, mut x } = editor.cursor_position;

    let mut width = if let Some(row) = editor.document.row(y) {
        row.len()
    } else {
        0
    };

    if x > width {
        x = width;
    }

    Position { x, y }
}
