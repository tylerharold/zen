use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::Editor;
use crate::EditorMode;

pub mod cursor;
pub mod mode;
pub mod view;

pub enum Commands {
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

#[async_trait]
pub trait Command {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct CommandQueue {
    pub sender: mpsc::Sender<Box<dyn Command>>,
    pub receiver: mpsc::Receiver<Box<dyn Command>>,
}
