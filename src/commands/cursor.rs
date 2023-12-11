use async_trait::async_trait;

use crate::editor::Editor;
use crate::Position;

use super::Command;

// Moves the cursor up 1
pub struct CursorMoveUpCommand;

// Moves the cursor down 1
pub struct CursorMoveDownCommand;

// Moves the cursor left 1
pub struct CursorMoveLeftCommand;

// Moves the cursor right 1
pub struct CursorMoveRightCommand;

// Moves the cursor to the beginning of the row
pub struct CursorMoveStartCommand;

// Moves the cursor to the end of the row
pub struct CursorMoveEndCommand;

// Moves the cursor to the next word
pub struct CursorMoveNextWordCommand;

// Moves the cursor to the previous word
pub struct CursorMovePrevWordCommand;

#[async_trait]
impl Command for CursorMoveUpCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let Position { mut y, mut x } = get_cursor_position(editor);

        y = y.saturating_sub(1);

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveDownCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let Position { mut y, mut x } = get_cursor_position(editor);
        let height = editor.document.len();

        if y < height {
            y = y.saturating_add(1);
        }

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveLeftCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
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

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveRightCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
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

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveStartCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let Position { mut y, mut x } = get_cursor_position(editor);

        x = 0;

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveEndCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let Position { mut y, mut x } = get_cursor_position(editor);

        let mut width = if let Some(row) = editor.document.row(y) {
            row.len()
        } else {
            0
        };

        x = width;

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMoveNextWordCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let Position { mut y, mut x } = get_cursor_position(editor);

        if let Some(row) = editor.document.row(y) {
            if let Some((i, _)) = row.string[x..]
                .split_whitespace()
                .next()
                .map(|word| (x + word.len(), word))
            {
                x = i;
            }

            editor.cursor_position = Position { x, y };
        }

        Ok(())
    }
}

#[async_trait]
impl Command for CursorMovePrevWordCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
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

        editor.cursor_position = Position { x, y };

        Ok(())
    }
}

fn get_cursor_position(editor: &mut Editor) -> Position {
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
