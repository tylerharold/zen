use crate::Editor;
use crate::EditorMode;

use super::Command;

// Sets the editor mode
pub struct SetModeCommand {
    pub mode: EditorMode,
}

impl Command for SetModeCommand {
    fn execute(&self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let mode = self.mode.clone();
        editor.mode = mode;

        Ok(())
    }
}
