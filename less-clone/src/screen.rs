//! Terminal abstraction for testable rendering.
//!
//! Defines the [`Terminal`] trait for all terminal I/O operations, with a
//! production [`CrosstermTerminal`] implementation and a `MockTerminal`
//! for unit testing.

use crate::error::LessError;
use crate::search::SearchState;
use crossterm::event::Event;
use std::io::{self, Write};

/// Abstraction over terminal operations for testability.
pub trait Terminal {
    /// Enter alternate screen and raw mode.
    fn enter(&mut self) -> Result<(), LessError>;
    /// Leave alternate screen and raw mode.
    fn leave(&mut self) -> Result<(), LessError>;
    /// Get terminal size as (columns, rows).
    fn size(&self) -> Result<(u16, u16), LessError>;
    /// Clear the screen.
    fn clear(&mut self) -> Result<(), LessError>;
    /// Move cursor to position (col, row), 0-based.
    fn move_to(&mut self, col: u16, row: u16) -> Result<(), LessError>;
    /// Print text at the current cursor position.
    fn print(&mut self, text: &str) -> Result<(), LessError>;
    /// Print text with inverted colors (for status line, search highlights).
    fn print_inverted(&mut self, text: &str) -> Result<(), LessError>;
    /// Print highlighted search match text.
    fn print_highlight(&mut self, text: &str) -> Result<(), LessError>;
    /// Flush output.
    fn flush(&mut self) -> Result<(), LessError>;
    /// Read the next terminal event (blocking).
    fn read_event(&mut self) -> Result<Event, LessError>;
    /// Hide cursor.
    fn hide_cursor(&mut self) -> Result<(), LessError>;
    /// Show cursor.
    fn show_cursor(&mut self) -> Result<(), LessError>;
}

/// Production terminal using crossterm.
pub struct CrosstermTerminal {
    stdout: io::Stdout,
}

impl CrosstermTerminal {
    /// Create a new crossterm-backed terminal.
    pub fn new() -> Result<Self, LessError> {
        Ok(CrosstermTerminal {
            stdout: io::stdout(),
        })
    }
}

impl Terminal for CrosstermTerminal {
    fn enter(&mut self) -> Result<(), LessError> {
        crossterm::terminal::enable_raw_mode()
            .map_err(|e| LessError::TerminalError(e.to_string()))?;
        crossterm::execute!(
            self.stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn leave(&mut self) -> Result<(), LessError> {
        crossterm::execute!(
            self.stdout,
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))?;
        crossterm::terminal::disable_raw_mode().map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn size(&self) -> Result<(u16, u16), LessError> {
        crossterm::terminal::size().map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn clear(&mut self) -> Result<(), LessError> {
        crossterm::execute!(
            self.stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn move_to(&mut self, col: u16, row: u16) -> Result<(), LessError> {
        crossterm::execute!(self.stdout, crossterm::cursor::MoveTo(col, row))
            .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn print(&mut self, text: &str) -> Result<(), LessError> {
        write!(self.stdout, "{text}").map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn print_inverted(&mut self, text: &str) -> Result<(), LessError> {
        crossterm::execute!(
            self.stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reverse),
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))?;
        write!(self.stdout, "{text}").map_err(|e| LessError::TerminalError(e.to_string()))?;
        crossterm::execute!(
            self.stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn print_highlight(&mut self, text: &str) -> Result<(), LessError> {
        crossterm::execute!(
            self.stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Black),
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Yellow),
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))?;
        write!(self.stdout, "{text}").map_err(|e| LessError::TerminalError(e.to_string()))?;
        crossterm::execute!(
            self.stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
        )
        .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn flush(&mut self) -> Result<(), LessError> {
        self.stdout
            .flush()
            .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn read_event(&mut self) -> Result<Event, LessError> {
        crossterm::event::read().map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn hide_cursor(&mut self) -> Result<(), LessError> {
        crossterm::execute!(self.stdout, crossterm::cursor::Hide)
            .map_err(|e| LessError::TerminalError(e.to_string()))
    }

    fn show_cursor(&mut self) -> Result<(), LessError> {
        crossterm::execute!(self.stdout, crossterm::cursor::Show)
            .map_err(|e| LessError::TerminalError(e.to_string()))
    }
}

/// Mock terminal for unit testing. Records output and provides scripted events.
#[cfg(test)]
pub struct MockTerminal {
    pub width: u16,
    pub height: u16,
    pub output: Vec<String>,
    pub events: Vec<Event>,
    event_index: usize,
    pub entered: bool,
    pub left: bool,
}

#[cfg(test)]
impl MockTerminal {
    pub fn new(width: u16, height: u16, events: Vec<Event>) -> Self {
        MockTerminal {
            width,
            height,
            output: Vec::new(),
            events,
            event_index: 0,
            entered: false,
            left: false,
        }
    }
}

#[cfg(test)]
impl Terminal for MockTerminal {
    fn enter(&mut self) -> Result<(), LessError> {
        self.entered = true;
        Ok(())
    }

    fn leave(&mut self) -> Result<(), LessError> {
        self.left = true;
        Ok(())
    }

    fn size(&self) -> Result<(u16, u16), LessError> {
        Ok((self.width, self.height))
    }

    fn clear(&mut self) -> Result<(), LessError> {
        self.output.push("[CLEAR]".to_string());
        Ok(())
    }

    fn move_to(&mut self, col: u16, row: u16) -> Result<(), LessError> {
        self.output.push(format!("[MOVE {col},{row}]"));
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<(), LessError> {
        self.output.push(text.to_string());
        Ok(())
    }

    fn print_inverted(&mut self, text: &str) -> Result<(), LessError> {
        self.output.push(format!("[INV]{text}[/INV]"));
        Ok(())
    }

    fn print_highlight(&mut self, text: &str) -> Result<(), LessError> {
        self.output.push(format!("[HL]{text}[/HL]"));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LessError> {
        Ok(())
    }

    fn read_event(&mut self) -> Result<Event, LessError> {
        if self.event_index < self.events.len() {
            let event = self.events[self.event_index].clone();
            self.event_index += 1;
            Ok(event)
        } else {
            Err(LessError::TerminalError("No more events".to_string()))
        }
    }

    fn hide_cursor(&mut self) -> Result<(), LessError> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), LessError> {
        Ok(())
    }
}

/// Render a page of content to the terminal.
///
/// Draws visible lines, optionally with line numbers and search highlighting,
/// then draws the status line at the bottom.
pub fn render(
    term: &mut dyn Terminal,
    lines: &[String],
    top_line: usize,
    status_text: &str,
    show_line_numbers: bool,
    search: Option<&SearchState>,
) -> Result<(), LessError> {
    let (cols, rows) = term.size()?;
    let content_rows = rows.saturating_sub(1) as usize; // Reserve bottom row for status
    let width = cols as usize;

    term.clear()?;

    // Calculate line number gutter width
    let gutter_width = if show_line_numbers {
        let max_line = top_line + content_rows;
        digit_count(max_line) + 1 // +1 for space separator
    } else {
        0
    };

    // Draw content lines
    for (i, line) in lines.iter().enumerate() {
        if i >= content_rows {
            break;
        }
        term.move_to(0, i as u16)?;

        if show_line_numbers {
            let line_num = top_line + i + 1;
            let num_str = format!("{:>width$} ", line_num, width = gutter_width - 1);
            term.print(&num_str)?;
        }

        // Render with search highlighting if active
        let available_width = width.saturating_sub(gutter_width);
        let display_line = if line.len() > available_width {
            &line[..available_width]
        } else {
            line
        };

        if let Some(search_state) = search {
            render_highlighted_line(term, display_line, search_state)?;
        } else {
            term.print(display_line)?;
        }
    }

    // Draw status line (inverted) on the last row
    term.move_to(0, rows.saturating_sub(1))?;
    let status_display = if status_text.len() > width {
        &status_text[..width]
    } else {
        status_text
    };
    // Pad status line to fill the width
    let padded = format!("{status_display:<width$}", width = width);
    term.print_inverted(&padded)?;

    term.flush()?;
    Ok(())
}

/// Render a single line with search match highlighting.
fn render_highlighted_line(
    term: &mut dyn Terminal,
    line: &str,
    search: &SearchState,
) -> Result<(), LessError> {
    let matches = search.find_matches_in_line(line);
    if matches.is_empty() {
        term.print(line)?;
        return Ok(());
    }

    let mut pos = 0;
    for (start, end) in &matches {
        if *start > pos {
            term.print(&line[pos..*start])?;
        }
        term.print_highlight(&line[*start..*end])?;
        pos = *end;
    }
    if pos < line.len() {
        term.print(&line[pos..])?;
    }
    Ok(())
}

/// Count the number of digits in a number (for line number gutter width).
fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    ((n as f64).log10().floor() as usize) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_count_values() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
        assert_eq!(digit_count(999), 3);
        assert_eq!(digit_count(1000), 4);
    }

    #[test]
    fn render_basic_page() {
        let lines: Vec<String> = vec!["line 1".into(), "line 2".into(), "line 3".into()];
        let mut term = MockTerminal::new(80, 5, vec![]);
        render(&mut term, &lines, 0, "status", false, None).unwrap();

        // Should contain the lines
        assert!(term.output.iter().any(|s| s == "line 1"));
        assert!(term.output.iter().any(|s| s == "line 2"));
        assert!(term.output.iter().any(|s| s == "line 3"));
        // Should contain inverted status
        assert!(term.output.iter().any(|s| s.contains("[INV]")));
    }

    #[test]
    fn render_with_line_numbers() {
        let lines: Vec<String> = vec!["hello".into()];
        let mut term = MockTerminal::new(80, 3, vec![]);
        render(&mut term, &lines, 0, "status", true, None).unwrap();

        // Should contain "1 " prefix (line number)
        assert!(term.output.iter().any(|s| s.contains("1 ")));
    }

    #[test]
    fn render_with_search_highlight() {
        let lines: Vec<String> = vec!["hello world".into()];
        let search = SearchState::new("world", crate::search::SearchDirection::Forward).unwrap();
        let mut term = MockTerminal::new(80, 3, vec![]);
        render(&mut term, &lines, 0, "status", false, Some(&search)).unwrap();

        // Should contain highlighted text
        assert!(term.output.iter().any(|s| s.contains("[HL]world[/HL]")));
    }

    #[test]
    fn render_status_inverted() {
        let lines: Vec<String> = vec![];
        let mut term = MockTerminal::new(40, 2, vec![]);
        render(&mut term, &lines, 0, "test status", false, None).unwrap();

        assert!(
            term.output
                .iter()
                .any(|s| s.contains("[INV]") && s.contains("test status"))
        );
    }

    #[test]
    fn mock_terminal_enter_leave() {
        let mut term = MockTerminal::new(80, 24, vec![]);
        assert!(!term.entered);
        assert!(!term.left);
        term.enter().unwrap();
        assert!(term.entered);
        term.leave().unwrap();
        assert!(term.left);
    }

    #[test]
    fn mock_terminal_size() {
        let term = MockTerminal::new(120, 40, vec![]);
        assert_eq!(term.size().unwrap(), (120, 40));
    }

    #[test]
    fn mock_terminal_events() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
        let events = vec![Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })];
        let mut term = MockTerminal::new(80, 24, events);
        let event = term.read_event().unwrap();
        assert!(matches!(event, Event::Key(_)));
        // No more events
        assert!(term.read_event().is_err());
    }
}
