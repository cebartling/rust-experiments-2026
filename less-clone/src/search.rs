//! Regex-based search functionality for the pager.
//!
//! [`SearchState`] manages a compiled regex pattern and provides forward and
//! backward search through the text buffer, including wrap-around and
//! per-line match offset extraction for highlighting.

use crate::buffer::TextBuffer;
use crate::error::LessError;
use regex::Regex;

/// Direction of search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Manages search state: pattern, direction, and current match position.
#[derive(Debug, Clone)]
pub struct SearchState {
    pattern: String,
    regex: Regex,
    pub direction: SearchDirection,
}

impl SearchState {
    /// Compile a new search pattern.
    pub fn new(pattern: &str, direction: SearchDirection) -> Result<Self, LessError> {
        let regex = Regex::new(pattern)?;
        Ok(SearchState {
            pattern: pattern.to_string(),
            regex,
            direction,
        })
    }

    /// Return the pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Find the next match line starting from `from_line` (exclusive) in the
    /// forward direction, wrapping around if necessary.
    pub fn find_forward(&self, buffer: &TextBuffer, from_line: usize) -> Option<usize> {
        let count = buffer.line_count();
        if count == 0 {
            return None;
        }

        // Search from from_line+1 to end
        for i in (from_line + 1)..count {
            if let Some(line) = buffer.line(i)
                && self.regex.is_match(line)
            {
                return Some(i);
            }
        }

        // Wrap around: search from 0 to from_line (inclusive)
        for i in 0..=from_line.min(count - 1) {
            if let Some(line) = buffer.line(i)
                && self.regex.is_match(line)
            {
                return Some(i);
            }
        }

        None
    }

    /// Find the next match line starting from `from_line` (exclusive) in the
    /// backward direction, wrapping around if necessary.
    pub fn find_backward(&self, buffer: &TextBuffer, from_line: usize) -> Option<usize> {
        let count = buffer.line_count();
        if count == 0 {
            return None;
        }

        // Search backward from from_line-1 to 0
        if from_line > 0 {
            for i in (0..from_line).rev() {
                if let Some(line) = buffer.line(i)
                    && self.regex.is_match(line)
                {
                    return Some(i);
                }
            }
        }

        // Wrap around: search from end to from_line (inclusive)
        for i in (from_line..count).rev() {
            if let Some(line) = buffer.line(i)
                && self.regex.is_match(line)
            {
                return Some(i);
            }
        }

        None
    }

    /// Find all match byte ranges within a single line for highlighting.
    pub fn find_matches_in_line(&self, line: &str) -> Vec<(usize, usize)> {
        self.regex
            .find_iter(line)
            .map(|m| (m.start(), m.end()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_buffer() -> TextBuffer {
        TextBuffer::from_string("apple\nbanana\ncherry\napricot\nblueberry")
    }

    #[test]
    fn new_valid_pattern() {
        let state = SearchState::new("test", SearchDirection::Forward);
        assert!(state.is_ok());
        assert_eq!(state.unwrap().pattern(), "test");
    }

    #[test]
    fn new_invalid_pattern() {
        let state = SearchState::new("[invalid", SearchDirection::Forward);
        assert!(state.is_err());
    }

    #[test]
    fn find_forward_basic() {
        let buf = sample_buffer();
        let state = SearchState::new("cherry", SearchDirection::Forward).unwrap();
        assert_eq!(state.find_forward(&buf, 0), Some(2));
    }

    #[test]
    fn find_forward_wraps_around() {
        let buf = sample_buffer();
        let state = SearchState::new("apple", SearchDirection::Forward).unwrap();
        // Starting from line 2, should wrap to line 0
        assert_eq!(state.find_forward(&buf, 2), Some(0));
    }

    #[test]
    fn find_forward_no_match() {
        let buf = sample_buffer();
        let state = SearchState::new("mango", SearchDirection::Forward).unwrap();
        assert_eq!(state.find_forward(&buf, 0), None);
    }

    #[test]
    fn find_forward_regex() {
        let buf = sample_buffer();
        let state = SearchState::new("^ap", SearchDirection::Forward).unwrap();
        assert_eq!(state.find_forward(&buf, 0), Some(3)); // "apricot"
    }

    #[test]
    fn find_backward_basic() {
        let buf = sample_buffer();
        let state = SearchState::new("banana", SearchDirection::Backward).unwrap();
        assert_eq!(state.find_backward(&buf, 3), Some(1));
    }

    #[test]
    fn find_backward_wraps_around() {
        let buf = sample_buffer();
        let state = SearchState::new("blueberry", SearchDirection::Backward).unwrap();
        // Starting from line 0, should wrap to line 4
        assert_eq!(state.find_backward(&buf, 0), Some(4));
    }

    #[test]
    fn find_backward_no_match() {
        let buf = sample_buffer();
        let state = SearchState::new("mango", SearchDirection::Backward).unwrap();
        assert_eq!(state.find_backward(&buf, 2), None);
    }

    #[test]
    fn find_matches_in_line_none() {
        let state = SearchState::new("xyz", SearchDirection::Forward).unwrap();
        assert!(state.find_matches_in_line("hello world").is_empty());
    }

    #[test]
    fn find_matches_in_line_single() {
        let state = SearchState::new("world", SearchDirection::Forward).unwrap();
        let matches = state.find_matches_in_line("hello world");
        assert_eq!(matches, vec![(6, 11)]);
    }

    #[test]
    fn find_matches_in_line_multiple() {
        let state = SearchState::new("an", SearchDirection::Forward).unwrap();
        let matches = state.find_matches_in_line("banana");
        assert_eq!(matches, vec![(1, 3), (3, 5)]);
    }

    #[test]
    fn find_forward_empty_buffer() {
        let buf = TextBuffer::from_string("");
        let state = SearchState::new("x", SearchDirection::Forward).unwrap();
        assert_eq!(state.find_forward(&buf, 0), None);
    }
}
