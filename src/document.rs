use crate::editor::{Position, SearchDirection};
use crate::row::Row;

use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::ops::Range;
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Representation of a file, existing or new.
#[derive(Default)]
pub struct Document {
    // {name.extension} - Optional in the case of a new file that hasn't been saved.
    pub file_name: Option<String>,

    // {extension} - ex=rs,ts,go,md,toml
    file_type: String,

    // Represents the file's contents, can be seen as a vec of lines.
    rows: Vec<Row>,

    // Has the document been modified since opening?
    dirty: bool,

    // A guideline on how to highlight the document's filetype.
    syntax_set: SyntaxSet,

    // A set of themes, includes convenient methods for loading and discovering themes.
    theme_set: ThemeSet,
}

impl Document {
    // Creates a new document (opens a file) based on the filename/path given.
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        // Grab the contents of the file
        let contents = fs::read_to_string(filename)?;

        let file_type = Path::new(filename)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or(&"Unknown");

        let mut rows = Vec::new();

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        for value in contents.lines() {
            rows.push(Row::from(value));
        }

        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            file_type: file_type.to_string(),
            syntax_set: ss,
            theme_set: ts,
        })
    }

    pub fn file_type(&self) -> String {
        self.file_type.clone()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;

        if c == '\n' {
            self.insert_newline(at);
        } else if at.y == self.rows.len() {
            // Handle insertion at the end of the document
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            // Handle regular character insertion
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;

        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= self.len() {
            return;
        }

        self.dirty = true;
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = ".rs".to_string();

            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let mut position = Position { x: at.x, y: at.y };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }

                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn highlight(&mut self, visible_range: Range<usize>) {
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(&self.file_type) {
            let mut h = HighlightLines::new(&syntax, &self.theme_set.themes["base16-ocean.dark"]);

            for row_num in visible_range {
                if let Some(row) = self.rows.get_mut(row_num) {
                    row.highlight(&self.syntax_set, &mut h);
                }
            }
        } else {
            // Handle this at some point
        }
    }
}
