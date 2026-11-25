use std::fmt::{Display, Formatter};

pub(self) const DEFAULT_FILE_PATH: &str = "<unknown>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFileLocation {
    file_path: String,
    line: usize,
    column: usize,
}

impl SourceFileLocation {
    pub fn new(file_path: String, line: usize, column: usize) -> Self {
        Self {
            file_path,
            line,
            column,
        }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn is_default(&self) -> bool {
        self.file_path == DEFAULT_FILE_PATH && self.line == 0 && self.column == 0
    }

    pub fn set_line_col(&mut self, (line, column): (usize, usize)) {
        self.line = line;
        self.column = column;
    }
}

impl Default for SourceFileLocation {
    fn default() -> Self {
        Self::new(DEFAULT_FILE_PATH.to_string(), 0, 0)
    }
}

impl From<(usize, usize)> for SourceFileLocation {
    fn from(value: (usize, usize)) -> Self {
        Self {
            file_path: DEFAULT_FILE_PATH.to_string(),
            line: value.0,
            column: value.1,
        }
    }
}

impl Display for SourceFileLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file_path, self.line, self.column)
    }
}

impl PartialOrd for SourceFileLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.file_path != other.file_path {
            return None;
        }

        if self.line != other.line {
            return Some(self.line.cmp(&other.line));
        }

        Some(self.column.cmp(&other.column))
    }
}
