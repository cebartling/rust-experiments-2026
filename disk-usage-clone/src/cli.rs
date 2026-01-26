//! Command-line interface argument parsing.
//!
//! This module defines the CLI structure using `clap`'s derive macros.
//! The [`CliArgs`] struct automatically generates:
//! - Argument parsing from `std::env::args()`
//! - `--help` and `--version` output
//! - Type validation and error messages
//!
//! # Examples
//!
//! ```
//! use disk_usage_clone::cli::CliArgs;
//! use clap::Parser;
//!
//! // Parse from command-line arguments
//! let args = CliArgs::parse_from(&["dusk", "-H", "/tmp"]);
//! assert!(args.human_readable);
//! assert_eq!(args.paths, vec!["/tmp"]);
//! ```

use clap::Parser;

use crate::entry::SortOrder;

/// Command-line arguments for the disk usage analyzer.
///
/// This struct uses `clap`'s derive macros to automatically generate
/// argument parsing. Each field corresponds to a CLI flag or argument.
///
/// # Derive Macros
///
/// - `#[derive(Parser)]` - Generates the parser from the struct
/// - `#[arg(...)]` - Configures individual arguments
/// - `#[command(...)]` - Configures program-level settings
///
/// # Flags
///
/// - `-H, --human-readable`: Human-readable sizes (1.5K vs 1536)
/// - `-s, --summarize`: Show only totals (like `du -s`)
/// - `-a, --all`: Show files, not just directories
/// - `-d, --max-depth <N>`: Limit traversal depth
/// - `-j, --threads <N>`: Control parallelization
/// - `--sort <ORDER>`: Sort by size/name
/// - `--no-color`: Disable color output
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use disk_usage_clone::cli::CliArgs;
/// use clap::Parser;
///
/// let args = CliArgs::parse_from(&["dusk", "."]);
/// assert_eq!(args.paths, vec!["."]);
/// ```
///
/// ## Multiple flags
///
/// ```
/// use disk_usage_clone::cli::CliArgs;
/// use clap::Parser;
///
/// let args = CliArgs::parse_from(&[
///     "dusk",
///     "-H",           // Human-readable
///     "-s",           // Summarize
///     "-d", "2",      // Max depth 2
///     "/var",
/// ]);
///
/// assert!(args.human_readable);
/// assert!(args.summarize);
/// assert_eq!(args.max_depth, Some(2));
/// ```
#[derive(Parser, Debug)]
#[command(name = "dusk", about = "Disk usage analysis tool", version)]
pub struct CliArgs {
    /// Paths to analyze (defaults to current directory)
    ///
    /// Multiple paths can be specified. Each is analyzed independently.
    ///
    /// # Examples
    ///
    /// ```bash
    /// dusk /tmp /var          # Analyze both directories
    /// dusk .                  # Analyze current directory
    /// dusk                    # Same (defaults to ".")
    /// ```
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// Print sizes in human-readable format (e.g., 1.5K, 2.3M, 4.1G)
    ///
    /// When enabled, sizes are formatted with SI prefixes:
    /// - `B` - Bytes
    /// - `K` - Kilobytes (1024 bytes)
    /// - `M` - Megabytes
    /// - `G` - Gigabytes
    /// - `T` - Terabytes
    ///
    /// Without this flag, sizes are shown as raw byte counts.
    #[arg(short = 'H', long = "human-readable")]
    pub human_readable: bool,

    /// Display only a total for each argument
    ///
    /// Shows only the grand total for each path, not individual files/directories.
    /// Similar to `du -s` or `du --summarize`.
    ///
    /// # Example
    ///
    /// ```bash
    /// dusk -s /var    # Shows only: "123.4M  /var"
    /// ```
    #[arg(short, long)]
    pub summarize: bool,

    /// Max depth of directory traversal
    ///
    /// Limits how deep into the directory tree to recurse:
    /// - `0` - Only the specified path
    /// - `1` - Path and immediate children
    /// - `2` - Two levels deep
    /// - etc.
    ///
    /// Deeper entries are collapsed into their parent's size.
    ///
    /// # Example
    ///
    /// ```bash
    /// dusk -d 1 /var  # Shows /var and immediate subdirs only
    /// ```
    #[arg(short = 'd', long = "max-depth")]
    pub max_depth: Option<usize>,

    /// Show all files, not just directories
    ///
    /// By default (like `du`), only directories are shown in the output.
    /// This flag includes individual files in the listing.
    ///
    /// Useful for finding large files within directories.
    #[arg(short, long)]
    pub all: bool,

    /// Sort order: size, size-asc, name, none
    ///
    /// Controls how entries are sorted:
    /// - `size` - Largest first (descending)
    /// - `size-asc` - Smallest first (ascending)
    /// - `name` - Alphabetical by path
    /// - `none` - Filesystem order (default)
    ///
    /// Sorting is applied recursively to all levels of the tree.
    ///
    /// # Example
    ///
    /// ```bash
    /// dusk --sort size /var  # Show largest directories first
    /// ```
    #[arg(long, default_value = "none")]
    pub sort: String,

    /// Number of threads for parallel traversal
    ///
    /// Controls the rayon thread pool size for parallel metadata collection.
    /// If not specified, defaults to the number of CPU cores.
    ///
    /// Lower values reduce CPU usage but increase runtime.
    /// Higher values improve performance on systems with many files.
    ///
    /// # Example
    ///
    /// ```bash
    /// dusk -j 4 /var  # Use 4 threads
    /// ```
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// Disable colorized output
    ///
    /// By default, output is colorized when writing to a terminal:
    /// - Sizes colored by magnitude (red for large, green for small)
    /// - Directories shown in blue
    ///
    /// This flag disables all colors. Useful for:
    /// - Scripting (ensures clean output)
    /// - Piping to files or other commands
    /// - Accessibility (screen readers, etc.)
    ///
    /// # Note
    ///
    /// Colors are automatically disabled when output is not a TTY.
    #[arg(long)]
    pub no_color: bool,
}

impl CliArgs {
    /// Parses the `sort` string into a `SortOrder` enum.
    ///
    /// Converts the CLI string argument into a type-safe enum.
    /// Invalid values default to `SortOrder::None`.
    ///
    /// # Returns
    ///
    /// - `SortOrder::SizeDescending` for "size"
    /// - `SortOrder::SizeAscending` for "size-asc"
    /// - `SortOrder::Name` for "name"
    /// - `SortOrder::None` for "none" or invalid values
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::cli::CliArgs;
    /// use disk_usage_clone::entry::SortOrder;
    /// use clap::Parser;
    ///
    /// let args = CliArgs::parse_from(&["dusk", "--sort", "size"]);
    /// assert_eq!(args.sort_order(), SortOrder::SizeDescending);
    ///
    /// let args = CliArgs::parse_from(&["dusk", "--sort", "invalid"]);
    /// assert_eq!(args.sort_order(), SortOrder::None); // Defaults to None
    /// ```
    pub fn sort_order(&self) -> SortOrder {
        SortOrder::parse(&self.sort).unwrap_or(SortOrder::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = CliArgs::parse_from(["dusk"]);
        assert_eq!(args.paths, vec!["."]);
        assert!(!args.human_readable);
        assert!(!args.summarize);
        assert_eq!(args.max_depth, None);
        assert!(!args.all);
        assert_eq!(args.sort, "none");
        assert_eq!(args.threads, None);
        assert!(!args.no_color);
    }

    #[test]
    fn test_human_readable_flag() {
        let args = CliArgs::parse_from(["dusk", "-H"]);
        assert!(args.human_readable);
    }

    #[test]
    fn test_summarize_flag() {
        let args = CliArgs::parse_from(["dusk", "-s"]);
        assert!(args.summarize);
    }

    #[test]
    fn test_max_depth() {
        let args = CliArgs::parse_from(["dusk", "-d", "3"]);
        assert_eq!(args.max_depth, Some(3));
    }

    #[test]
    fn test_all_flag() {
        let args = CliArgs::parse_from(["dusk", "-a"]);
        assert!(args.all);
    }

    #[test]
    fn test_sort_option() {
        let args = CliArgs::parse_from(["dusk", "--sort", "size"]);
        assert_eq!(args.sort, "size");
        assert_eq!(args.sort_order(), SortOrder::SizeDescending);
    }

    #[test]
    fn test_threads_option() {
        let args = CliArgs::parse_from(["dusk", "-j", "4"]);
        assert_eq!(args.threads, Some(4));
    }

    #[test]
    fn test_no_color_flag() {
        let args = CliArgs::parse_from(["dusk", "--no-color"]);
        assert!(args.no_color);
    }

    #[test]
    fn test_multiple_paths() {
        let args = CliArgs::parse_from(["dusk", "/tmp", "/var"]);
        assert_eq!(args.paths, vec!["/tmp", "/var"]);
    }

    #[test]
    fn test_combined_flags() {
        let args = CliArgs::parse_from(["dusk", "-H", "-s", "-a", "-d", "2", "--sort", "name"]);
        assert!(args.human_readable);
        assert!(args.summarize);
        assert!(args.all);
        assert_eq!(args.max_depth, Some(2));
        assert_eq!(args.sort_order(), SortOrder::Name);
    }

    #[test]
    fn test_sort_order_invalid_defaults_to_none() {
        let args = CliArgs::parse_from(["dusk", "--sort", "bogus"]);
        assert_eq!(args.sort_order(), SortOrder::None);
    }
}
