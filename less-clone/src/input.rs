//! Keyboard input mapping for the pager.
//!
//! Defines the [`Action`] enum representing all pager commands and
//! [`map_event`] which translates crossterm key events into actions.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

/// All possible pager actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Quit the pager.
    Quit,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one page.
    PageDown,
    /// Scroll up by one page.
    PageUp,
    /// Scroll down by half a page.
    HalfPageDown,
    /// Scroll up by half a page.
    HalfPageUp,
    /// Jump to the top of the file.
    GoToTop,
    /// Jump to the bottom of the file.
    GoToBottom,
    /// Enter forward search mode.
    SearchForward,
    /// Enter backward search mode.
    SearchBackward,
    /// Go to the next search match.
    NextMatch,
    /// Go to the previous search match.
    PrevMatch,
    /// Toggle line number display.
    ToggleLineNumbers,
    /// Show help screen.
    Help,
    /// Terminal was resized.
    Resize(u16, u16),
    /// No action (unrecognized key).
    None,
}

/// Map a crossterm [`Event`] to an [`Action`].
///
/// This handles the Normal mode key bindings. Search input mode is handled
/// separately by the pager.
pub fn map_event(event: &Event) -> Action {
    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => map_key(*code, *modifiers),
        Event::Resize(cols, rows) => Action::Resize(*cols, *rows),
        _ => Action::None,
    }
}

fn map_key(code: KeyCode, modifiers: KeyModifiers) -> Action {
    // Ctrl+C always quits
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        return Action::Quit;
    }

    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Char(' ') | KeyCode::Char('f') | KeyCode::PageDown => Action::PageDown,
        KeyCode::Char('b') | KeyCode::PageUp => Action::PageUp,
        KeyCode::Char('d') => Action::HalfPageDown,
        KeyCode::Char('u') => Action::HalfPageUp,
        KeyCode::Char('g') | KeyCode::Home => Action::GoToTop,
        KeyCode::Char('G') | KeyCode::End => Action::GoToBottom,
        KeyCode::Char('/') => Action::SearchForward,
        KeyCode::Char('?') => Action::SearchBackward,
        KeyCode::Char('n') => Action::NextMatch,
        KeyCode::Char('N') => Action::PrevMatch,
        KeyCode::Char('l') => Action::ToggleLineNumbers,
        KeyCode::Char('h') => Action::Help,
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn key(code: KeyCode) -> Event {
        key_event(code, KeyModifiers::NONE)
    }

    #[test]
    fn quit_q() {
        assert_eq!(map_event(&key(KeyCode::Char('q'))), Action::Quit);
    }

    #[test]
    fn quit_ctrl_c() {
        assert_eq!(
            map_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Action::Quit
        );
    }

    #[test]
    fn scroll_down_j() {
        assert_eq!(map_event(&key(KeyCode::Char('j'))), Action::ScrollDown);
    }

    #[test]
    fn scroll_down_arrow() {
        assert_eq!(map_event(&key(KeyCode::Down)), Action::ScrollDown);
    }

    #[test]
    fn scroll_down_enter() {
        assert_eq!(map_event(&key(KeyCode::Enter)), Action::ScrollDown);
    }

    #[test]
    fn scroll_up_k() {
        assert_eq!(map_event(&key(KeyCode::Char('k'))), Action::ScrollUp);
    }

    #[test]
    fn scroll_up_arrow() {
        assert_eq!(map_event(&key(KeyCode::Up)), Action::ScrollUp);
    }

    #[test]
    fn page_down_space() {
        assert_eq!(map_event(&key(KeyCode::Char(' '))), Action::PageDown);
    }

    #[test]
    fn page_down_f() {
        assert_eq!(map_event(&key(KeyCode::Char('f'))), Action::PageDown);
    }

    #[test]
    fn page_down_key() {
        assert_eq!(map_event(&key(KeyCode::PageDown)), Action::PageDown);
    }

    #[test]
    fn page_up_b() {
        assert_eq!(map_event(&key(KeyCode::Char('b'))), Action::PageUp);
    }

    #[test]
    fn page_up_key() {
        assert_eq!(map_event(&key(KeyCode::PageUp)), Action::PageUp);
    }

    #[test]
    fn half_page_down() {
        assert_eq!(map_event(&key(KeyCode::Char('d'))), Action::HalfPageDown);
    }

    #[test]
    fn half_page_up() {
        assert_eq!(map_event(&key(KeyCode::Char('u'))), Action::HalfPageUp);
    }

    #[test]
    fn go_to_top_g() {
        assert_eq!(map_event(&key(KeyCode::Char('g'))), Action::GoToTop);
    }

    #[test]
    fn go_to_top_home() {
        assert_eq!(map_event(&key(KeyCode::Home)), Action::GoToTop);
    }

    #[test]
    fn go_to_bottom_shift_g() {
        assert_eq!(map_event(&key(KeyCode::Char('G'))), Action::GoToBottom);
    }

    #[test]
    fn go_to_bottom_end() {
        assert_eq!(map_event(&key(KeyCode::End)), Action::GoToBottom);
    }

    #[test]
    fn search_forward() {
        assert_eq!(map_event(&key(KeyCode::Char('/'))), Action::SearchForward);
    }

    #[test]
    fn search_backward() {
        assert_eq!(map_event(&key(KeyCode::Char('?'))), Action::SearchBackward);
    }

    #[test]
    fn next_match() {
        assert_eq!(map_event(&key(KeyCode::Char('n'))), Action::NextMatch);
    }

    #[test]
    fn prev_match() {
        assert_eq!(map_event(&key(KeyCode::Char('N'))), Action::PrevMatch);
    }

    #[test]
    fn toggle_line_numbers() {
        assert_eq!(
            map_event(&key(KeyCode::Char('l'))),
            Action::ToggleLineNumbers
        );
    }

    #[test]
    fn help() {
        assert_eq!(map_event(&key(KeyCode::Char('h'))), Action::Help);
    }

    #[test]
    fn resize_event() {
        let event = Event::Resize(80, 24);
        assert_eq!(map_event(&event), Action::Resize(80, 24));
    }

    #[test]
    fn unknown_key_returns_none() {
        assert_eq!(map_event(&key(KeyCode::Char('z'))), Action::None);
    }

    #[test]
    fn mouse_event_returns_none() {
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(map_event(&event), Action::None);
    }
}
