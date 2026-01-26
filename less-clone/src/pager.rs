//! Pager event loop and state management.
//!
//! [`PagerState`] tracks the current scroll position, search state, display
//! mode, and other pager state. [`run_pager`] is the main event loop that
//! reads terminal events, maps them to actions, updates state, and renders.

use crate::buffer::TextBuffer;
use crate::cli::CliArgs;
use crate::error::LessError;
use crate::input::{self, Action};
use crate::screen::{self, Terminal};
use crate::search::{SearchDirection, SearchState};
use crate::status;

/// Modal input mode for the pager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Normal browsing mode.
    Normal,
    /// Entering a search pattern (forward).
    SearchInput(SearchDirection),
}

/// Pager display and navigation state.
pub struct PagerState {
    /// Index of the first visible line (0-based).
    pub top_line: usize,
    /// Current search state, if any.
    pub search: Option<SearchState>,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// Whether the help screen is showing.
    pub show_help: bool,
    /// Whether the pager should quit.
    pub quit: bool,
    /// Current input mode.
    pub mode: Mode,
    /// Search input buffer (while in SearchInput mode).
    pub search_input: String,
    /// Terminal width.
    pub cols: u16,
    /// Terminal height.
    pub rows: u16,
}

impl PagerState {
    /// Create initial state from CLI args and terminal size.
    pub fn new(args: &CliArgs, cols: u16, rows: u16) -> Self {
        PagerState {
            top_line: 0,
            search: None,
            show_line_numbers: args.line_numbers,
            show_help: false,
            quit: false,
            mode: Mode::Normal,
            search_input: String::new(),
            cols,
            rows,
        }
    }

    /// Number of content rows (total rows minus status line).
    pub fn content_rows(&self) -> usize {
        self.rows.saturating_sub(1) as usize
    }

    /// Apply an action to update pager state.
    pub fn apply_action(&mut self, action: Action, buffer: &TextBuffer) {
        match action {
            Action::Quit => self.quit = true,
            Action::ScrollDown => self.scroll_down(1, buffer),
            Action::ScrollUp => self.scroll_up(1),
            Action::PageDown => {
                let page = self.content_rows();
                self.scroll_down(page, buffer);
            }
            Action::PageUp => {
                let page = self.content_rows();
                self.scroll_up(page);
            }
            Action::HalfPageDown => {
                let half = self.content_rows() / 2;
                self.scroll_down(half, buffer);
            }
            Action::HalfPageUp => {
                let half = self.content_rows() / 2;
                self.scroll_up(half);
            }
            Action::GoToTop => self.top_line = 0,
            Action::GoToBottom => {
                let total = buffer.line_count();
                let visible = self.content_rows();
                self.top_line = total.saturating_sub(visible);
            }
            Action::SearchForward => {
                self.mode = Mode::SearchInput(SearchDirection::Forward);
                self.search_input.clear();
            }
            Action::SearchBackward => {
                self.mode = Mode::SearchInput(SearchDirection::Backward);
                self.search_input.clear();
            }
            Action::NextMatch => self.navigate_match(buffer, false),
            Action::PrevMatch => self.navigate_match(buffer, true),
            Action::ToggleLineNumbers => self.show_line_numbers = !self.show_line_numbers,
            Action::Help => self.show_help = !self.show_help,
            Action::Resize(cols, rows) => {
                self.cols = cols;
                self.rows = rows;
            }
            Action::None => {}
        }
    }

    fn scroll_down(&mut self, lines: usize, buffer: &TextBuffer) {
        let max_top = buffer.line_count().saturating_sub(self.content_rows());
        self.top_line = (self.top_line + lines).min(max_top);
    }

    fn scroll_up(&mut self, lines: usize) {
        self.top_line = self.top_line.saturating_sub(lines);
    }

    fn navigate_match(&mut self, buffer: &TextBuffer, reverse: bool) {
        if let Some(ref search) = self.search {
            let found = if reverse {
                match search.direction {
                    SearchDirection::Forward => search.find_backward(buffer, self.top_line),
                    SearchDirection::Backward => search.find_forward(buffer, self.top_line),
                }
            } else {
                match search.direction {
                    SearchDirection::Forward => search.find_forward(buffer, self.top_line),
                    SearchDirection::Backward => search.find_backward(buffer, self.top_line),
                }
            };

            if let Some(line) = found {
                self.top_line = line;
            }
        }
    }

    /// Submit search input and switch back to Normal mode.
    pub fn submit_search(&mut self, buffer: &TextBuffer) {
        let direction = match self.mode {
            Mode::SearchInput(dir) => dir,
            Mode::Normal => return,
        };

        if let Ok(search) = SearchState::new(&self.search_input, direction) {
            // Find first match from current position
            let found = match direction {
                SearchDirection::Forward => search.find_forward(buffer, self.top_line),
                SearchDirection::Backward => search.find_backward(buffer, self.top_line),
            };
            if let Some(line) = found {
                self.top_line = line;
            }
            self.search = Some(search);
        }

        self.mode = Mode::Normal;
        self.search_input.clear();
    }

    /// Cancel search input and return to Normal mode.
    pub fn cancel_search(&mut self) {
        self.mode = Mode::Normal;
        self.search_input.clear();
    }
}

/// Help text displayed when the user presses 'h'.
const HELP_TEXT: &str = "\
  NAVIGATION
    j / Down / Enter   Scroll down one line
    k / Up             Scroll up one line
    Space / f / PgDn   Scroll down one page
    b / PgUp           Scroll up one page
    d                  Scroll down half page
    u                  Scroll up half page
    g / Home           Go to top
    G / End            Go to bottom

  SEARCH
    /pattern           Search forward
    ?pattern           Search backward
    n                  Next match
    N                  Previous match

  OPTIONS
    l                  Toggle line numbers
    h                  Toggle this help

  QUIT
    q / Ctrl-C         Quit";

/// Run the pager event loop.
///
/// Enters the terminal, renders content, reads events, and loops until quit.
/// Always restores the terminal on exit, even if an error occurs.
pub fn run_pager(
    term: &mut dyn Terminal,
    buffer: TextBuffer,
    args: &CliArgs,
) -> Result<(), LessError> {
    term.enter()?;

    let result = run_pager_inner(term, &buffer, args);

    // Always restore terminal
    let leave_result = term.leave();

    // Return the first error
    result?;
    leave_result
}

fn run_pager_inner(
    term: &mut dyn Terminal,
    buffer: &TextBuffer,
    args: &CliArgs,
) -> Result<(), LessError> {
    let (cols, rows) = term.size()?;
    let mut state = PagerState::new(args, cols, rows);

    loop {
        render_frame(term, buffer, &state)?;

        let event = term.read_event()?;

        match state.mode {
            Mode::Normal => {
                let action = input::map_event(&event);
                state.apply_action(action, buffer);
            }
            Mode::SearchInput(_) => {
                handle_search_input(&event, &mut state, buffer);
            }
        }

        if state.quit {
            break;
        }
    }

    Ok(())
}

/// Handle key events while in search input mode.
fn handle_search_input(
    event: &crossterm::event::Event,
    state: &mut PagerState,
    buffer: &TextBuffer,
) {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    if let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event
    {
        if modifiers.contains(KeyModifiers::CONTROL) && *code == KeyCode::Char('c') {
            state.cancel_search();
            return;
        }
        match code {
            KeyCode::Enter => state.submit_search(buffer),
            KeyCode::Esc => state.cancel_search(),
            KeyCode::Backspace => {
                state.search_input.pop();
            }
            KeyCode::Char(c) => {
                state.search_input.push(*c);
            }
            _ => {}
        }
    }
}

/// Render a single frame (content or help).
fn render_frame(
    term: &mut dyn Terminal,
    buffer: &TextBuffer,
    state: &PagerState,
) -> Result<(), LessError> {
    if state.show_help {
        render_help(term, state)?;
    } else {
        let visible = buffer
            .lines_range(state.top_line, state.top_line + state.content_rows())
            .to_vec();

        let search_pattern = state.search.as_ref().map(|s| s.pattern());
        let status_text = match &state.mode {
            Mode::SearchInput(SearchDirection::Forward) => {
                format!("/{}", state.search_input)
            }
            Mode::SearchInput(SearchDirection::Backward) => {
                format!("?{}", state.search_input)
            }
            Mode::Normal => status::format_status(
                buffer.filename(),
                state.top_line,
                state.content_rows(),
                buffer.line_count(),
                search_pattern,
            ),
        };

        screen::render(
            term,
            &visible,
            state.top_line,
            &status_text,
            state.show_line_numbers,
            state.search.as_ref(),
        )?;
    }
    Ok(())
}

/// Render the help screen.
fn render_help(term: &mut dyn Terminal, state: &PagerState) -> Result<(), LessError> {
    let help_lines: Vec<String> = HELP_TEXT.lines().map(String::from).collect();
    screen::render(
        term,
        &help_lines,
        0,
        "HELP -- Press h or q to close",
        false,
        None,
    )?;
    let _ = state; // Used for terminal size via render
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::MockTerminal;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn default_args() -> CliArgs {
        CliArgs {
            file: None,
            line_numbers: false,
        }
    }

    fn make_key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn small_buffer() -> TextBuffer {
        TextBuffer::from_string("line 1\nline 2\nline 3\nline 4\nline 5")
    }

    fn large_buffer() -> TextBuffer {
        let lines: Vec<String> = (1..=100).map(|i| format!("line {i}")).collect();
        TextBuffer::from_string(&lines.join("\n"))
    }

    #[test]
    fn initial_state() {
        let args = default_args();
        let state = PagerState::new(&args, 80, 24);
        assert_eq!(state.top_line, 0);
        assert!(!state.quit);
        assert!(!state.show_line_numbers);
        assert!(!state.show_help);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn scroll_down() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.apply_action(Action::ScrollDown, &buf);
        assert_eq!(state.top_line, 1);
    }

    #[test]
    fn scroll_up() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.top_line = 5;
        state.apply_action(Action::ScrollUp, &buf);
        assert_eq!(state.top_line, 4);
    }

    #[test]
    fn scroll_up_at_top_stays() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.apply_action(Action::ScrollUp, &buf);
        assert_eq!(state.top_line, 0);
    }

    #[test]
    fn scroll_down_clamps_to_max() {
        let args = default_args();
        let buf = small_buffer(); // 5 lines
        let mut state = PagerState::new(&args, 80, 4); // 3 content rows
        state.top_line = 2; // max is 5-3=2
        state.apply_action(Action::ScrollDown, &buf);
        assert_eq!(state.top_line, 2); // stays at max
    }

    #[test]
    fn page_down() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24); // 23 content rows
        state.apply_action(Action::PageDown, &buf);
        assert_eq!(state.top_line, 23);
    }

    #[test]
    fn page_up() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.top_line = 50;
        state.apply_action(Action::PageUp, &buf);
        assert_eq!(state.top_line, 27);
    }

    #[test]
    fn half_page_down() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24); // 23 content rows, half = 11
        state.apply_action(Action::HalfPageDown, &buf);
        assert_eq!(state.top_line, 11);
    }

    #[test]
    fn go_to_top() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.top_line = 50;
        state.apply_action(Action::GoToTop, &buf);
        assert_eq!(state.top_line, 0);
    }

    #[test]
    fn go_to_bottom() {
        let args = default_args();
        let buf = large_buffer();
        let mut state = PagerState::new(&args, 80, 24); // 23 content rows
        state.apply_action(Action::GoToBottom, &buf);
        assert_eq!(state.top_line, 77); // 100 - 23
    }

    #[test]
    fn toggle_line_numbers() {
        let args = default_args();
        let buf = small_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        assert!(!state.show_line_numbers);
        state.apply_action(Action::ToggleLineNumbers, &buf);
        assert!(state.show_line_numbers);
        state.apply_action(Action::ToggleLineNumbers, &buf);
        assert!(!state.show_line_numbers);
    }

    #[test]
    fn toggle_help() {
        let args = default_args();
        let buf = small_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        assert!(!state.show_help);
        state.apply_action(Action::Help, &buf);
        assert!(state.show_help);
    }

    #[test]
    fn resize() {
        let args = default_args();
        let buf = small_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.apply_action(Action::Resize(120, 40), &buf);
        assert_eq!(state.cols, 120);
        assert_eq!(state.rows, 40);
    }

    #[test]
    fn search_forward_mode() {
        let args = default_args();
        let buf = small_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.apply_action(Action::SearchForward, &buf);
        assert_eq!(state.mode, Mode::SearchInput(SearchDirection::Forward));
    }

    #[test]
    fn search_submit_and_navigate() {
        let args = default_args();
        let buf = TextBuffer::from_string("apple\nbanana\ncherry\napricot");
        let mut state = PagerState::new(&args, 80, 24);

        // Enter search mode
        state.apply_action(Action::SearchForward, &buf);
        state.search_input = "cherry".to_string();
        state.submit_search(&buf);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.top_line, 2); // cherry is on line 2
        assert!(state.search.is_some());
    }

    #[test]
    fn search_cancel() {
        let args = default_args();
        let buf = small_buffer();
        let mut state = PagerState::new(&args, 80, 24);
        state.apply_action(Action::SearchForward, &buf);
        state.search_input = "partial".to_string();
        state.cancel_search();
        assert_eq!(state.mode, Mode::Normal);
        assert!(state.search_input.is_empty());
    }

    #[test]
    fn run_pager_quit_immediately() {
        let events = vec![make_key_event(KeyCode::Char('q'))];
        let mut term = MockTerminal::new(80, 24, events);
        let buf = TextBuffer::from_string("hello\nworld");
        let args = default_args();
        run_pager(&mut term, buf, &args).unwrap();
        assert!(term.entered);
        assert!(term.left);
    }

    #[test]
    fn run_pager_scroll_then_quit() {
        let events = vec![
            make_key_event(KeyCode::Char('j')),
            make_key_event(KeyCode::Char('q')),
        ];
        let mut term = MockTerminal::new(80, 5, events);
        let lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
        let buf = TextBuffer::from_string(&lines.join("\n"));
        let args = default_args();
        run_pager(&mut term, buf, &args).unwrap();
    }

    #[test]
    fn content_rows_calculation() {
        let args = default_args();
        let state = PagerState::new(&args, 80, 24);
        assert_eq!(state.content_rows(), 23);
    }
}
