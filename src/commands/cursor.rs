use crate::editor::{Editor, Position};

use std::cmp;

pub fn move_up(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

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

pub fn move_next_word(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    if let Some(row) = editor.document.row(y) {
        if let Some((i, _)) = row.string[x..]
            .split_whitespace()
            .next()
            .map(|word| (x + word.len(), word))
        {
            x = i;
        }

        editor.cursor_position = Position { x, y }
    }
}

pub fn move_prev_word(editor: &mut Editor) {
    let Position { mut y, mut x } = get_cursor_position(editor);

    if x > 0 {
        if let Some(row) = editor.document.row(y) {
            let substring = &row.string[..x];
            let mut prev_space_index = None;

            for (i, c) in substring.char_indices().rev() {
                if c.is_whitespace() {
                    prev_space_index = Some(i);
                    break;
                }
            }

            x = match prev_space_index {
                Some(index) => index + 1,
                None => 0,
            }
        }
    }

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
