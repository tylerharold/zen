use crate::editor::SearchDirection;

use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use unicode_segmentation::UnicodeSegmentation;

/// Implementation of a document's row/line.
#[derive(Default)]
pub struct Row {
    // The raw slice of a line's content.
    pub string: String,

    // A highlighted version of the string. This gets updated when the row is in view,
    // or on change, but will be initialized with the same value as self.string.
    highlighting: String,

    // String length with graphemes in consideration
    // Updated on change
    len: usize,
}

impl From<&str> for Row {
    // Typically called when instantiating a row through a loop of content lines in a document.
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            highlighting: String::from(slice),
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {
    // Returns a display-ready string for the terminal
    pub fn render(&self) -> String {
        self.highlighting.clone()
    }

    // Gets the length of a string with graphemes in consideration
    pub fn len(&self) -> usize {
        self.len
    }

    // Checks if the row's string is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // Keeps the string's length updated on change
    pub fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    // Handles row insertions
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at).collect();

            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    // Handles row insertions, alternative for a string.
    // Probably not necessary.
    pub fn insert_str(&mut self, at: usize, str: &str) {
        if at >= self.len() {
            self.string.push_str(str);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at).collect();

            result.push_str(str);
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    // Handles deletions to the row's string.
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();

            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut splitted_row: String = String::new();
        let mut splitted_length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        let highlighting = splitted_row.clone();
        Self {
            string: splitted_row,
            len: splitted_length,
            highlighting: highlighting,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
            return None;
        }

        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            0
        };

        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();

        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };

        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(start + grapheme_index);
                }
            }
        }

        None
    }

    pub fn highlight(&mut self, syntax_set: &SyntaxSet, highlighter: &mut HighlightLines) {
        let ranges: Vec<(Style, &str)> = highlighter
            .highlight_line(&self.string, syntax_set)
            .unwrap();

        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);

        self.highlighting = escaped;
    }

    pub fn whitespace_len(&self) -> usize {
        self.string
            .chars()
            .take_while(|c| c.is_whitespace())
            .count()
    }
}

fn is_separator(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_ascii_whitespace()
}
