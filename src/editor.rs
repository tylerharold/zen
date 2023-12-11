use crate::commands;
use crate::commands::Command;
use crate::commands::CommandQueue;
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
use tokio::sync::mpsc;

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
    pub mode: EditorMode,

    // Command queue
    command_queue: CommandQueue,

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
    pub async fn run(&mut self) {
        let command_processor_handle = tokio::spawn(async { Ok::<(), tokio::task::JoinError>(()) });

        loop {
            if let Err(error) = self.refresh_screen() {
                print!("{}", termion::clear::All);
                panic!("{error:?}");
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_keypress().await {
                die(error);
            }
        }

        match command_processor_handle.await {
            Ok(inner_result) => {
                if let Err(e) = inner_result {
                    print!("{}", termion::clear::All);
                    panic!("{e:?}");
                }
            }
            Err(e) => {
                print!("{}", termion::clear::All);
                panic!("{e:?}");
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

        let (command_sender, command_receiver) = mpsc::channel(100);
        let command_queue: CommandQueue = CommandQueue {
            sender: command_sender,
            receiver: command_receiver,
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
            command_queue,
        }
    }

    // Processes keypresses in the active terminal.
    // Used by the main editor loop and checked after a frame has finished rendering.
    // TODO: These keymaps will be loaded through a configuration file.
    async fn process_keypress(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pressed_key = Terminal::read_key()?;

        let command: Option<Box<dyn Command>> = match self.mode {
            EditorMode::Normal => match pressed_key {
                Key::Char('i') => Some(Box::new(commands::mode::SetModeCommand {
                    mode: EditorMode::Insert,
                })),

                Key::Char('h') => Some(Box::new(commands::cursor::CursorMoveLeftCommand)),
                Key::Char('j') => Some(Box::new(commands::cursor::CursorMoveUpCommand)),
                Key::Char('k') => Some(Box::new(commands::cursor::CursorMoveDownCommand)),
                Key::Char('l') => Some(Box::new(commands::cursor::CursorMoveRightCommand)),

                Key::Left => Some(Box::new(commands::cursor::CursorMoveLeftCommand)),
                Key::Up => Some(Box::new(commands::cursor::CursorMoveUpCommand)),
                Key::Down => Some(Box::new(commands::cursor::CursorMoveDownCommand)),
                Key::Right => Some(Box::new(commands::cursor::CursorMoveRightCommand)),

                _ => None,
            },
            EditorMode::Insert => match pressed_key {
                Key::Char('i') => Some(Box::new(commands::mode::SetModeCommand {
                    mode: EditorMode::Normal,
                })),

                Key::Left => Some(Box::new(commands::cursor::CursorMoveLeftCommand)),
                Key::Up => Some(Box::new(commands::cursor::CursorMoveUpCommand)),
                Key::Down => Some(Box::new(commands::cursor::CursorMoveDownCommand)),
                Key::Right => Some(Box::new(commands::cursor::CursorMoveRightCommand)),

                _ => None,
            },
            EditorMode::Command => match pressed_key {
                _ => None,
            },
        };

        if let Some(cmd) = command {
            self.push_command(cmd)?;
        }

        self.scroll();

        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }

        Ok(())
    }

    pub async fn run_command_loop(&mut self) {
        while let Some(command) = self.command_queue.receiver.recv().await {
            command.execute(self).expect("Failed to execute command");
        }
    }

    fn push_command(&self, command: Box<dyn Command>) -> Result<(), Box<dyn std::error::Error>> {
        self.command_queue.sender.try_send(command)?;

        Ok(())
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
    async fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);

            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }

            self.document.file_name = new_name;
        }

        if self.document.save().await.is_ok() {
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
                            //editor.execute(Command::CursorMoveRight);
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
                        //editor.execute(Command::CursorMoveLeft);
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

fn die(e: Box<dyn std::error::Error>) {
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
