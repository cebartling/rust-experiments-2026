//! Text buffer for pager content.
//!
//! [`TextBuffer`] stores file or stdin content as a vector of lines with
//! efficient indexed access. Supports construction from files, readers, or
//! strings.

use crate::error::LessError;
use std::fs;
use std::io::{BufRead, Read};
use std::path::Path;

/// Stores pager content as indexed lines.
#[derive(Debug, Clone)]
pub struct TextBuffer {
    lines: Vec<String>,
    filename: Option<String>,
}

impl TextBuffer {
    /// Load content from a file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LessError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let lines = content.lines().map(String::from).collect();
        Ok(TextBuffer {
            lines,
            filename: Some(path.display().to_string()),
        })
    }

    /// Load content from any reader (e.g., stdin).
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, LessError> {
        let buf_reader = std::io::BufReader::new(reader);
        let lines: Vec<String> = buf_reader.lines().collect::<Result<_, _>>()?;
        Ok(TextBuffer {
            lines,
            filename: None,
        })
    }

    /// Create a buffer from an in-memory string.
    pub fn from_string(content: &str) -> Self {
        let lines = content.lines().map(String::from).collect();
        TextBuffer {
            lines,
            filename: None,
        }
    }

    /// Return the total number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Return true if the buffer has no lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get a single line by index (0-based). Returns `None` if out of bounds.
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Get a range of lines. Clamps to valid bounds.
    pub fn lines_range(&self, start: usize, end: usize) -> &[String] {
        let start = start.min(self.lines.len());
        let end = end.min(self.lines.len());
        &self.lines[start..end]
    }

    /// Return the filename if the buffer was loaded from a file.
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Set a display name for the buffer.
    pub fn set_filename(&mut self, name: String) {
        self.filename = Some(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[test]
    fn from_string_basic() {
        let buf = TextBuffer::from_string("hello\nworld");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("hello"));
        assert_eq!(buf.line(1), Some("world"));
    }

    #[test]
    fn from_string_empty() {
        let buf = TextBuffer::from_string("");
        assert_eq!(buf.line_count(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn from_string_single_line() {
        let buf = TextBuffer::from_string("just one line");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line(0), Some("just one line"));
    }

    #[test]
    fn from_string_trailing_newline() {
        let buf = TextBuffer::from_string("line1\nline2\n");
        assert_eq!(buf.line_count(), 2);
    }

    #[test]
    fn line_out_of_bounds() {
        let buf = TextBuffer::from_string("one");
        assert_eq!(buf.line(1), None);
        assert_eq!(buf.line(100), None);
    }

    #[test]
    fn lines_range_basic() {
        let buf = TextBuffer::from_string("a\nb\nc\nd\ne");
        let range = buf.lines_range(1, 4);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0], "b");
        assert_eq!(range[2], "d");
    }

    #[test]
    fn lines_range_clamps() {
        let buf = TextBuffer::from_string("a\nb\nc");
        let range = buf.lines_range(1, 100);
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn lines_range_empty_when_start_past_end() {
        let buf = TextBuffer::from_string("a\nb");
        let range = buf.lines_range(5, 10);
        assert!(range.is_empty());
    }

    #[test]
    fn from_reader() {
        let data = "line1\nline2\nline3";
        let cursor = Cursor::new(data);
        let buf = TextBuffer::from_reader(cursor).unwrap();
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line(0), Some("line1"));
        assert!(buf.filename().is_none());
    }

    #[test]
    fn from_file() {
        use std::io::Write;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "file line 1\nfile line 2").unwrap();
        let buf = TextBuffer::from_file(tmp.path()).unwrap();
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("file line 1"));
        assert!(buf.filename().is_some());
    }

    #[test]
    fn from_file_not_found() {
        let result = TextBuffer::from_file("/nonexistent/path/to/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn filename_none_for_string() {
        let buf = TextBuffer::from_string("hi");
        assert!(buf.filename().is_none());
    }

    #[test]
    fn set_filename() {
        let mut buf = TextBuffer::from_string("hi");
        buf.set_filename("stdin".to_string());
        assert_eq!(buf.filename(), Some("stdin"));
    }

    #[test]
    fn is_empty() {
        assert!(TextBuffer::from_string("").is_empty());
        assert!(!TextBuffer::from_string("x").is_empty());
    }
}
