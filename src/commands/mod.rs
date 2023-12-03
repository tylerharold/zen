use crate::EditorMode;

pub mod cursor;
pub mod view;

pub enum Command {
    // Document
    DocumentInsert(char),
    DocumentPageUp,
    DocumentPageDown,
    DocumentMoveStart,
    DocumentMoveEnd,
    DocumentSave,
    DocumentSearch,
    DocumentQuit,

    // Cursor
    CursorMoveUp,
    CursorMoveDown,
    CursorMoveLeft,
    CursorMoveRight,
    CursorMoveStart,
    CursorMoveEnd,
    CursorMoveNextWord,
    CursorMovePrevWord,

    // Editor
    EditorSwitchMode(EditorMode),
}
