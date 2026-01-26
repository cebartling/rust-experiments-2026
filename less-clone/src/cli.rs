//! Command-line argument parsing for the less-clone pager.
//!
//! Uses clap derive macros for automatic argument parsing.

use clap::Parser;

/// A terminal pager similar to Unix `less`.
///
/// Reads a file or stdin and provides scrollable, searchable viewing.
#[derive(Parser, Debug)]
#[command(name = "less-clone", version, about)]
pub struct CliArgs {
    /// File to display. If omitted, reads from stdin.
    pub file: Option<String>,

    /// Show line numbers.
    #[arg(short = 'N', long = "line-numbers")]
    pub line_numbers: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn no_args() {
        let args = CliArgs::try_parse_from(["less-clone"]).unwrap();
        assert!(args.file.is_none());
        assert!(!args.line_numbers);
    }

    #[test]
    fn file_arg() {
        let args = CliArgs::try_parse_from(["less-clone", "test.txt"]).unwrap();
        assert_eq!(args.file, Some("test.txt".to_string()));
    }

    #[test]
    fn line_numbers_short() {
        let args = CliArgs::try_parse_from(["less-clone", "-N", "file.txt"]).unwrap();
        assert!(args.line_numbers);
        assert_eq!(args.file, Some("file.txt".to_string()));
    }

    #[test]
    fn line_numbers_long() {
        let args = CliArgs::try_parse_from(["less-clone", "--line-numbers"]).unwrap();
        assert!(args.line_numbers);
    }

    #[test]
    fn unknown_flag_errors() {
        let result = CliArgs::try_parse_from(["less-clone", "--unknown"]);
        assert!(result.is_err());
    }
}
