//! Disk usage analyzer library.
//!
//! This crate provides a multi-threaded disk usage analysis tool similar to the Unix `du` command.
//! It traverses filesystems in parallel, calculates disk usage, and presents results with
//! colorized output and flexible formatting options.
//!
//! # Architecture
//!
//! The crate is organized into focused modules:
//! - [`cli`]: Command-line argument parsing
//! - [`entry`]: Core data structures (DiskEntry tree)
//! - [`error`]: Error types and handling
//! - [`formatter`]: Size formatting utilities
//! - [`output`]: Terminal rendering and colorization
//! - [`traversal`]: Filesystem traversal with parallelization
//!
//! # Quick Start
//!
//! ```no_run
//! use disk_usage_clone::{cli::CliArgs, run};
//! use clap::Parser;  // Required for parse_from
//!
//! // Parse arguments
//! let args = CliArgs::parse_from(&["dusk", "-H", "/tmp"]);
//!
//! // Run analysis
//! run(&args).expect("Analysis failed");
//! ```
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use disk_usage_clone::{cli::CliArgs, run_to_string};
//! use clap::Parser;  // Required for parse_from
//!
//! let args = CliArgs::parse_from(&["dusk", "."]);
//! let output = run_to_string(&args).expect("Failed to analyze");
//! println!("{}", output);
//! ```
//!
//! ## With Options
//!
//! ```no_run
//! use disk_usage_clone::{cli::CliArgs, run_to_string};
//! use clap::Parser;  // Required for parse_from
//!
//! // Human-readable sizes, sorted by size, summarize only
//! let args = CliArgs::parse_from(&["dusk", "-H", "-s", "--sort", "size", "/var"]);
//! let output = run_to_string(&args).expect("Failed to analyze");
//! ```

pub mod cli;
pub mod entry;
pub mod error;
pub mod formatter;
pub mod output;
pub mod traversal;

use std::path::Path;

use cli::CliArgs;
use error::DuskError;
use output::render_tree;
use traversal::traverse_parallel;

/// Runs disk usage analysis and prints results to stdout.
///
/// This is the primary entry point for the CLI tool. It:
/// 1. Analyzes all paths specified in `args`
/// 2. Applies sorting and depth limiting
/// 3. Renders output with colorization
/// 4. Prints to stdout
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments containing paths and options
///
/// # Returns
///
/// - `Ok(())` - Analysis completed successfully
/// - `Err(DuskError)` - Filesystem error, permission denied, etc.
///
/// # Examples
///
/// ```no_run
/// use disk_usage_clone::{cli::CliArgs, run};
/// use clap::Parser;
///
/// let args = CliArgs::parse_from(&["dusk", "-H", "/tmp"]);
/// run(&args).expect("Analysis failed");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Any path doesn't exist
/// - Permission denied on any directory
/// - I/O error during traversal
pub fn run(args: &CliArgs) -> Result<(), DuskError> {
    // Delegate to run_to_string for the actual work
    let output = run_to_string(args)?;
    // Print result to stdout
    println!("{output}");
    Ok(())
}

/// Runs disk usage analysis and returns formatted output as a string.
///
/// This function performs the same analysis as [`run`] but returns the output
/// as a `String` instead of printing it. This is useful for:
/// - Testing (assertions on output)
/// - Programmatic usage
/// - Capturing output for further processing
///
/// # Workflow
///
/// 1. For each path in `args.paths`:
///    - Traverse filesystem (parallel metadata collection)
///    - Build `DiskEntry` tree
///    - Apply sorting if requested
///    - Render to string
/// 2. Join all outputs with newlines
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
///
/// # Returns
///
/// - `Ok(String)` - Formatted output ready for display
/// - `Err(DuskError)` - Analysis failed
///
/// # Examples
///
/// ```no_run
/// use disk_usage_clone::{cli::CliArgs, run_to_string};
/// use clap::Parser;  // Required for parse_from
///
/// let args = CliArgs::parse_from(&["dusk", "-H", "-s", "."]);
/// let output = run_to_string(&args).unwrap();
/// assert!(output.contains("B") || output.contains("K")); // Human-readable
/// ```
///
/// # Multi-Threading
///
/// Uses rayon for parallel metadata collection. Thread count can be controlled
/// via `args.threads`, otherwise defaults to number of CPU cores.
///
/// # Errors
///
/// Same error conditions as [`run`].
pub fn run_to_string(args: &CliArgs) -> Result<String, DuskError> {
    // Determine if color should be used (inverted from --no-color flag)
    let use_color = !args.no_color;

    // Parse the sort order string into an enum
    let sort_order = args.sort_order();

    // Collect results for all requested paths
    let mut results = Vec::new();

    // Process each path independently
    for path_str in &args.paths {
        let path = Path::new(path_str);

        // Traverse filesystem and build DiskEntry tree
        // Uses parallel metadata collection for performance
        let mut tree = traverse_parallel(path, args.max_depth, args.threads)?;

        // Apply sorting if requested (recursive on entire tree)
        tree.sort_entries(&sort_order);

        // Render the tree to a string with requested formatting
        let output = render_tree(
            &tree,
            args.human_readable,  // Format as K, M, G or raw bytes?
            args.all,              // Show files or directories only?
            args.summarize,        // Show only totals?
            use_color,             // Colorize output?
        );
        results.push(output);
    }

    // Join all path outputs with newlines
    Ok(results.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("hello.txt"), "hello world").unwrap();
        fs::create_dir(tmp.path().join("subdir")).unwrap();
        fs::write(tmp.path().join("subdir/data.bin"), "0123456789").unwrap();
        tmp
    }

    #[test]
    fn test_run_to_string_basic() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "--no-color", tmp.path().to_str().unwrap()]);
        let result = run_to_string(&args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_run_to_string_human_readable() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-H", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Should contain human-readable sizes (ending with B, K, M, etc.)
        assert!(
            output.contains('B') || output.contains('K'),
            "Expected human-readable size in output: {output}"
        );
    }

    #[test]
    fn test_run_to_string_summarize() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-s", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Summarize should produce a single line
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn test_run_to_string_show_all() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-a", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Should show files too
        assert!(output.contains("hello.txt"));
        assert!(output.contains("data.bin"));
    }

    #[test]
    fn test_run_to_string_nonexistent_path() {
        let args = CliArgs::parse_from(["dusk", "/nonexistent/path/does/not/exist"]);
        let result = run_to_string(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_to_string_sorted_by_name() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from([
            "dusk",
            "-a",
            "--no-color",
            "--sort",
            "name",
            tmp.path().to_str().unwrap(),
        ]);
        let output = run_to_string(&args).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_run_to_string_max_depth() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from([
            "dusk",
            "-d",
            "0",
            "--no-color",
            tmp.path().to_str().unwrap(),
        ]);
        let output = run_to_string(&args).unwrap();
        // depth 0 = only root, so single line
        assert_eq!(output.lines().count(), 1);
    }

    use clap::Parser;
}
