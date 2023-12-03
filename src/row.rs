use crate::util::style_to_termion;
use crate::SearchDirection;

use std::cmp;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<Style>,
    pub is_highlighted: bool,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            is_highlighted: false,
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);

        let mut result = String::new();
        for (index, grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let default_style = Style::default();
                let style = self.highlighting.get(index).unwrap_or(&default_style);

                let formatted_grapheme = format!("{}{}", style_to_termion(style), c);
                result.push_str(&formatted_grapheme);
            }
        }
        result.push_str(&format!("{}", color::Fg(color::Reset)));
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

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
        self.is_highlighted = false;
        Self {
            string: splitted_row,
            len: splitted_length,
            is_highlighted: false,
            highlighting: Vec::new(),
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

    pub fn highlight(
        &mut self,
        syntax: &SyntaxReference,
        theme: &ThemeSet,
        syntax_set: &SyntaxSet,
        h: &mut HighlightLines,
    ) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(&self.string, &syntax_set).unwrap();

        self.highlighting = ranges.iter().map(|(style, _)| style.clone()).collect();
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
