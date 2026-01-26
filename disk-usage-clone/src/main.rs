//! Binary entry point for the disk usage analyzer CLI.
//!
//! This is the main executable that provides the command-line interface.
//! It handles:
//! - Parsing command-line arguments using `clap`
//! - Calling the library's `run()` function
//! - Error handling and exit code management
//!
//! # Architecture
//!
//! The binary is kept minimal - all business logic lives in the library (`lib.rs`).
//! This separation enables:
//! - Testing without spawning processes
//! - Reusing the library in other Rust projects
//! - Clear separation between CLI and core logic
//!
//! # Example Usage
//!
//! ```bash
//! # Analyze current directory
//! disk-usage-clone
//!
//! # Analyze with human-readable sizes
//! disk-usage-clone -H /path/to/dir
//!
//! # Show summary only, sorted by size
//! disk-usage-clone -s --sort size /tmp
//! ```

use std::process;

use clap::Parser;

use disk_usage_clone::cli::CliArgs;
use disk_usage_clone::run;

/// Main entry point for the disk usage analyzer.
///
/// This function orchestrates the CLI workflow:
/// 1. Parse command-line arguments using `CliArgs::parse()`
/// 2. Delegate to `disk_usage_clone::run()` for execution
/// 3. Handle errors by printing to stderr and setting exit code
///
/// # Exit Codes
///
/// - `0`: Success
/// - `1`: Error (printed to stderr)
///
/// # Error Handling
///
/// All errors are caught at this top level and formatted for the user.
/// We use `eprintln!` to write errors to stderr (not stdout), following
/// Unix conventions.
///
/// # Example
///
/// ```no_run
/// // When run from command line:
/// // $ disk-usage-clone /invalid/path
/// // dusk: Path not found: /invalid/path
/// // (exits with code 1)
/// ```
fn main() {
    // Parse CLI arguments - clap will handle --help, --version, and validation
    let args = CliArgs::parse();

    // Run the main logic and handle any errors
    // The ? operator can't be used in main, so we use if let
    if let Err(err) = run(&args) {
        // Print error to stderr with program name prefix
        eprintln!("dusk: {err}");
        // Exit with non-zero code to indicate failure
        process::exit(1);
    }
    // Successful execution exits with code 0 (implicit)
}
