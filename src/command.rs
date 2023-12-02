use crate::EditorMode;

pub enum Command {
    // Document
    DocumentInsert(char),
    DocumentPageUp,
    DocumentPageDown,
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

    // Editor
    EditorSwitchMode(EditorMode),
}
