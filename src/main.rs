#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]
mod commands;
mod document;
mod editor;
mod mode;
mod row;
mod terminal;
mod util;

use editor::Editor;

pub use document::Document;
pub use editor::Position;
pub use editor::SearchDirection;
pub use mode::EditorMode;
pub use row::Row;
pub use terminal::Terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    Editor::default().await.run();

    Ok(())
}
