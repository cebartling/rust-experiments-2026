//! # less-clone
//!
//! A terminal pager similar to Unix `less` that reads files or stdin and
//! provides scrollable, searchable viewing in the terminal.
//!
//! ## Architecture
//!
//! - **cli** — Command-line argument parsing (clap derive)
//! - **error** — Unified error type [`error::LessError`]
//! - **buffer** — Text content storage with line-indexed access
//! - **input** — Keyboard event to action mapping
//! - **search** — Regex-based forward/backward search
//! - **screen** — Terminal abstraction trait and implementations
//! - **status** — Status line formatting
//! - **pager** — Event loop and state management

pub mod buffer;
pub mod cli;
pub mod error;
pub mod input;
pub mod pager;
pub mod screen;
pub mod search;
pub mod status;

use cli::CliArgs;
use error::LessError;
use std::io::{self, Read};

/// Run the pager with the given CLI arguments.
///
/// Loads the text buffer from a file or stdin, initializes the terminal,
/// and runs the pager event loop. Terminal state is always restored on exit.
pub fn run(args: &CliArgs) -> Result<(), LessError> {
    let buf = load_buffer(args)?;
    let mut terminal = screen::CrosstermTerminal::new()?;
    pager::run_pager(&mut terminal, buf, args)
}

/// Load text content into a buffer from the source specified by CLI args.
fn load_buffer(args: &CliArgs) -> Result<buffer::TextBuffer, LessError> {
    if let Some(ref path) = args.file {
        buffer::TextBuffer::from_file(path)
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        if input.is_empty() {
            return Err(LessError::NoInput);
        }
        Ok(buffer::TextBuffer::from_string(&input))
    }
}
