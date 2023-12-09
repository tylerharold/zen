use crate::commands;
use crate::commands::Command;
use crate::Document;
use crate::EditorMode;
use crate::Row;
use crate::Terminal;

use std::cmp;
use std::env;
use std::ops::Range;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);

const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

/// 2D Position
#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Used by the status bar to display text
struct StatusMessage {
    // Text displayed in the status bar
    text: String,

    // How long before the message should be displayed.
    time: Instant,
}

/// Used by the search functionality to dictate which direction we're looking for text
#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Entry point for our application/editor.
pub struct Editor {
    // Manages terminal impl
    pub terminal: Terminal,

    // Manages active document
    pub document: Document,

    // Current cursor Position (x, y)
    pub cursor_position: Position,

    // Horizontal & vertical offset (x, y)
    pub offset: Position,

    // Current Editor mode the user is in
    mode: EditorMode,

    // Highlighted word, for search, etc.
    highlighted_word: Option<String>,

    // Active status message for the status bar.
    status_message: StatusMessage,

    // How many times the user should hit the quit hotkey before exiting a dirty document.
    quit_times: u8,

    // Breaks the run loop when set to true.
    should_quit: bool,
}

impl Editor {
    // Main application loop. Used in main.rs to instantiate the editor.
    // Should quit check is called after the frame has finished initializing.
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }

    // Editor defaults.
    // Handles arguments and initial editor states.
    pub async fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = match Document::open(&file_name).await {
                Ok(doc) => doc,
                Err(_) => {
                    initial_status = format!("ERR: Could not open file: {}", file_name);
                    Document::default()
                }
            };

            doc
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
            mode: EditorMode::Normal,
        }
    }

    // Processes keypresses in the active terminal.
    // Used by the main editor loop and checked after a frame has finished rendering.
    // TODO: These keymaps will be loaded through a configuration file.
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match self.mode {
            EditorMode::Normal => match pressed_key {
                // Switch to Insert Mode
                Key::Char('i') => self.execute(Command::EditorSwitchMode(EditorMode::Insert)),

                Key::Char('h') => self.execute(Command::CursorMoveLeft),
                Key::Char('j') => self.execute(Command::CursorMoveUp),
                Key::Char('k') => self.execute(Command::CursorMoveDown),
                Key::Char('l') => self.execute(Command::CursorMoveRight),

                Key::Left => self.execute(Command::CursorMovePrevWord),
                Key::Right => self.execute(Command::CursorMoveNextWord),

                Key::Ctrl('J') => self.execute(Command::DocumentMoveStart),
                Key::Ctrl('K') => self.execute(Command::DocumentMoveEnd),
                Key::Char('J') => self.execute(Command::DocumentPageUp),
                Key::Char('K') => self.execute(Command::DocumentPageDown),
                Key::Char('H') => self.execute(Command::CursorMoveStart),
                Key::Char('L') => self.execute(Command::CursorMoveEnd),

                Key::Ctrl('q') => {
                    if self.quit_times > 0 && self.document.is_dirty() {
                        self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                        self.quit_times -= 1;
                        return Ok(());
                    }
                    self.should_quit = true
                }
                _ => (),
            },
            EditorMode::Insert => match pressed_key {
                // Switch to Normal mode
                Key::Esc => self.execute(Command::EditorSwitchMode(EditorMode::Normal)),

                Key::Ctrl('s') => self.execute(Command::DocumentSave),
                Key::Ctrl('f') => self.execute(Command::DocumentSearch),
                Key::Char(c) => self.execute(Command::DocumentInsert(c)),
                Key::Delete => self.document.delete(&self.cursor_position),
                Key::Backspace => {
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        self.execute(Command::CursorMoveLeft);
                        self.document.delete(&self.cursor_position);
                    }
                }
                Key::Up => self.execute(Command::CursorMoveUp),
                Key::Down => self.execute(Command::CursorMoveDown),
                Key::Left => self.execute(Command::CursorMoveLeft),
                Key::Right => self.execute(Command::CursorMoveRight),
                Key::PageUp => self.execute(Command::DocumentPageUp),
                Key::PageDown => self.execute(Command::DocumentPageDown),
                Key::Home => self.execute(Command::CursorMoveStart),
                Key::End => self.execute(Command::CursorMoveEnd),
                _ => (),
            },
            EditorMode::Command => match pressed_key {
                _ => (),
            },
        }

        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    // Executes a command given
    // Matches commands::Command to a public function found in the commands folder.
    // TODO: The goal of having the commands folder is for the potential use of a plugin
    // system that could utilize these functions to interact with with the editor.
    fn execute(&mut self, command: Command) {
        match command {
            Command::CursorMoveUp => commands::cursor::move_up(self),
            Command::CursorMoveDown => commands::cursor::move_down(self),
            Command::CursorMoveLeft => commands::cursor::move_left(self),
            Command::CursorMoveRight => commands::cursor::move_right(self),
            Command::CursorMoveStart => commands::cursor::move_start_of_row(self),
            Command::CursorMoveEnd => commands::cursor::move_end_of_row(self),
            Command::CursorMoveNextWord => commands::cursor::move_next_word(self),
            Command::CursorMovePrevWord => commands::cursor::move_prev_word(self),

            Command::DocumentInsert(c) => {
                self.document.insert(&self.cursor_position, c);
                self.execute(Command::CursorMoveRight);
            }
            Command::DocumentSave => self.save(),
            Command::DocumentSearch => self.search(),
            Command::DocumentPageUp => commands::view::scroll_up(self),
            Command::DocumentPageDown => commands::view::scroll_down(self),
            Command::DocumentMoveStart => commands::cursor::move_start_of_document(self),
            Command::DocumentMoveEnd => commands::cursor::move_end_of_document(self),

            Command::EditorSwitchMode(mode) => self.mode = mode,
            _ => (),
        }
    }

    // Handles terminal scrolling by adjusting the offset.
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;

        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    // Handles frame/screen refreshes.
    // Includes highlighting & redrawing the rows & TUI
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            println!("Goodbye.\r");
            Terminal::clear_screen();
        } else {
            let viewport = self.calculate_viewport();

            // It's important that we highlight before drawing
            // We will only be highlighting the rows visible in the viewport to improve performance
            self.document.highlight(viewport);
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    // Returns a range of the row indexes within the terminal's view.
    fn calculate_viewport(&self) -> Range<usize> {
        let height = self.terminal.size().height as usize;
        let start_row = self.offset.y;
        let end_row = cmp::min(self.offset.y + height, self.document.len());

        start_row..end_row
    }

    // Handles re-rendering all rows within the terminal's view.
    // Will account for cases such as an empty document, or an empty row.
    // (?) Might move that...
    //
    // In the case of a populated row, we will call self.draw_row(row),
    // which will then call row.render(), and self.draw_row will print
    // the string that returns from row.render().
    //
    // This is probably overcomplicated and will be rewritten.
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();

            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message()
            } else {
                println!("~\r");
            }
        }
    }

    // Draws a welcome message in the case of an empty document.
    // The case check can currently be found here, in self.draw_rows()
    // (As of pre-0.1)
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Zen {}\r", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    // Handles printing a row to the terminal with the String provided
    // by row.render()
    fn draw_row(&self, row: &Row) {
        let row = row.render();
        println!("{}\r", row)
    }

    // Draws a status bar to the terminal.
    // This is primarily used for information on the document, such
    // as the file opened, dirty status, document's language, etc.
    // TODO: Stylize this with the active theme.
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    // Message bar used to display text and command assistance.
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;

        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    // Used by search and command operations by providing an input state.
    // This uses the message bar.
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    // Saves the active document.
    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);

            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }

            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file".to_string());
        }
    }

    // Active document search functionality.
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();

        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                |editor, key, query| {
                    let mut moved = false;
                    match key {
                        Key::Right | Key::Down => {
                            direction = SearchDirection::Forward;
                            editor.execute(Command::CursorMoveRight);
                            moved = true;
                        }
                        Key::Left | Key::Up => direction = SearchDirection::Backward,
                        _ => direction = SearchDirection::Forward,
                    }
                    if let Some(position) =
                        editor
                            .document
                            .find(&query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.execute(Command::CursorMoveLeft);
                    }
                    editor.highlighted_word = Some(query.to_string());
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
    }
}

fn die(e: std::io::Error) {
    print!("{}", termion::clear::All);
    panic!("{e:?}");
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}
