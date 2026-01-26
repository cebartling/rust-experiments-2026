//! Terminal output rendering and colorization.
//!
//! This module handles converting `DiskEntry` trees into formatted,
//! optionally colorized terminal output.
//!
//! # Features
//!
//! - Size-based colorization (large = red, small = green)
//! - Type-based colorization (directories = blue)
//! - Human-readable formatting support
//! - Summarize mode (totals only)
//! - Show-all mode (files + directories)
//!
//! # Color Scheme
//!
//! ## Size Colors
//!
//! - **Red (bold)**: ≥ 1 GB - Large files, cleanup candidates
//! - **Yellow (bold)**: ≥ 100 MB - Notable size
//! - **Yellow**: ≥ 1 MB - Medium files
//! - **Green**: ≥ 1 KB - Normal files
//! - **Dimmed**: < 1 KB - Small files
//!
//! ## Type Colors
//!
//! - **Blue (bold)**: Directories
//! - **Cyan**: Symlinks
//! - **Default**: Files and other
//!
//! # Examples
//!
//! ```no_run
//! use disk_usage_clone::output::render_tree;
//! use disk_usage_clone::entry::{DiskEntry, EntryType};
//! use std::path::PathBuf;
//!
//! let entry = DiskEntry::new(
//!     PathBuf::from("/tmp"),
//!     4096,
//!     EntryType::Directory,
//!     0,
//! );
//!
//! // Render with color and human-readable sizes
//! let output = render_tree(&entry, true, false, false, true);
//! println!("{}", output);
//! ```

use colored::Colorize;

use crate::entry::{DiskEntry, EntryType};
use crate::formatter::format_size;

/// Kilobyte constant for size thresholds.
const KB: u64 = 1024;

/// Megabyte constant for size thresholds.
const MB: u64 = 1024 * KB;

/// Gigabyte constant for size thresholds.
const GB: u64 = 1024 * MB;

/// Applies color to a size string based on magnitude.
///
/// Uses a semantic color scheme where:
/// - Large sizes are red (urgent attention)
/// - Medium sizes are yellow (notable)
/// - Small sizes are green (OK)
///
/// # Color Thresholds
///
/// | Size | Color | Meaning |
/// |------|-------|---------|
/// | ≥ 1 GB | Red (bold) | Very large, investigate |
/// | ≥ 100 MB | Yellow (bold) | Large, notable |
/// | ≥ 1 MB | Yellow | Medium size |
/// | ≥ 1 KB | Green | Normal size |
/// | < 1 KB | Dimmed | Small size |
///
/// # Arguments
///
/// * `size_str` - Pre-formatted size string (e.g., "1.5K", "2.3M")
/// * `bytes` - Raw size in bytes (for threshold checking)
///
/// # Returns
///
/// String with ANSI color codes embedded.
///
/// # Examples
///
/// ```ignore
/// use disk_usage_clone::output::colorize_size;
///
/// // Large file - red
/// let colored = colorize_size("2.5G", 2_684_354_560);
///
/// // Medium file - yellow
/// let colored = colorize_size("5.0M", 5_242_880);
///
/// // Small file - green
/// let colored = colorize_size("10K", 10_240);
/// ```
///
/// # Note
///
/// The returned string contains ANSI escape codes. When printed to a
/// terminal, these produce colored output. When written to a file or
/// piped, the `colored` crate automatically omits the codes (unless
/// overridden).
fn colorize_size(size_str: &str, bytes: u64) -> String {
    // Check thresholds from largest to smallest
    if bytes >= GB {
        // Very large: red and bold for urgency
        size_str.red().bold().to_string()
    } else if bytes >= 100 * MB {
        // Large: yellow and bold for notice
        size_str.yellow().bold().to_string()
    } else if bytes >= MB {
        // Medium: yellow for attention
        size_str.yellow().to_string()
    } else if bytes >= KB {
        // Normal: green for "OK"
        size_str.green().to_string()
    } else {
        // Very small: dimmed to de-emphasize
        size_str.dimmed().to_string()
    }
}

/// Applies color to a path string based on entry type.
///
/// Follows common Unix conventions:
/// - Directories are blue (like `ls --color`)
/// - Symlinks are cyan
/// - Files are default color
///
/// # Arguments
///
/// * `path_str` - Path to colorize
/// * `entry_type` - Type of filesystem entry
///
/// # Returns
///
/// String with ANSI color codes (or plain string for files).
///
/// # Examples
///
/// ```ignore
/// use disk_usage_clone::output::colorize_path;
/// use disk_usage_clone::entry::EntryType;
///
/// // Directory - blue
/// let colored = colorize_path("/home/user", &EntryType::Directory);
///
/// // Symlink - cyan
/// let colored = colorize_path("/usr/bin/python", &EntryType::Symlink);
///
/// // File - no color
/// let plain = colorize_path("/tmp/data.txt", &EntryType::File);
/// ```
fn colorize_path(path_str: &str, entry_type: &EntryType) -> String {
    match entry_type {
        EntryType::Directory => {
            // Directories: blue and bold
            // Follows `ls --color` convention
            path_str.blue().bold().to_string()
        }
        EntryType::Symlink => {
            // Symlinks: cyan
            // Also follows `ls --color` convention
            path_str.cyan().to_string()
        }
        EntryType::File | EntryType::Other => {
            // Files: default color (no ANSI codes)
            path_str.to_string()
        }
    }
}

/// Renders a single entry as a formatted line.
///
/// Produces output like:
/// ```text
/// 1.5K    /tmp/file.txt
/// 2.3M    /var/log/messages
/// ```
///
/// With color:
/// ```text
/// \x1b[32m1.5K\x1b[0m    /tmp/file.txt
/// \x1b[33;1m2.3M\x1b[0m  \x1b[34;1m/var/log\x1b[0m
/// ```
///
/// # Arguments
///
/// * `entry` - Entry to render
/// * `human_readable` - Use K/M/G suffixes?
/// * `use_color` - Apply colorization?
///
/// # Returns
///
/// Formatted string: `<size>\t<path>`
///
/// # Examples
///
/// ```
/// use disk_usage_clone::output::render_entry;
/// use disk_usage_clone::entry::{DiskEntry, EntryType};
/// use std::path::PathBuf;
///
/// let entry = DiskEntry::new(
///     PathBuf::from("test.txt"),
///     1024,
///     EntryType::File,
///     0,
/// );
///
/// // No color, raw bytes
/// let output = render_entry(&entry, false, false);
/// assert_eq!(output, "1024\ttest.txt");
///
/// // Human-readable, no color
/// let output = render_entry(&entry, true, false);
/// assert_eq!(output, "1.0K\ttest.txt");
/// ```
///
/// # Total Size
///
/// Uses `entry.total_size()`, which includes all descendants for directories.
pub fn render_entry(entry: &DiskEntry, human_readable: bool, use_color: bool) -> String {
    // Calculate total size (includes children for directories)
    let size = entry.total_size();

    // Format the size as string
    let size_str = format_size(size, human_readable);

    // Convert path to string
    let path_str = entry.path.display().to_string();

    if use_color {
        // Apply semantic colorization
        let colored_size = colorize_size(&size_str, size);
        let colored_path = colorize_path(&path_str, &entry.entry_type);
        // Tab-separated: size <TAB> path
        format!("{colored_size}\t{colored_path}")
    } else {
        // Plain output (no ANSI codes)
        format!("{size_str}\t{path_str}")
    }
}

/// Renders a tree of entries as multi-line output.
///
/// Main rendering function that orchestrates output for an entire tree.
///
/// # Modes
///
/// ## Summarize Mode (`summarize = true`)
///
/// Shows only the root entry's total:
/// ```text
/// 123.4M  /var
/// ```
///
/// ## Normal Mode (`summarize = false`)
///
/// Shows all entries, recursively. Behavior depends on `show_all`:
///
/// - `show_all = false`: Only directories (like `du`)
/// - `show_all = true`: Directories and files (like `du -a`)
///
/// ## Output Order
///
/// Follows `du` convention: children before parents
///
/// ```text
/// 1.0K    /var/log/file.log
/// 5.0M    /var/log
/// 100M    /var
/// ```
///
/// This is useful for piping to `tail` to see largest entries.
///
/// # Arguments
///
/// * `entry` - Root of tree to render
/// * `human_readable` - Format sizes as K/M/G?
/// * `show_all` - Show files, or directories only?
/// * `summarize` - Show only total?
/// * `use_color` - Apply colorization?
///
/// # Returns
///
/// Newline-joined string of all entries.
///
/// # Examples
///
/// ```
/// use disk_usage_clone::output::render_tree;
/// use disk_usage_clone::entry::{DiskEntry, EntryType};
/// use std::path::PathBuf;
///
/// let mut dir = DiskEntry::new(
///     PathBuf::from("/tmp"),
///     4096,
///     EntryType::Directory,
///     0,
/// );
/// dir.children.push(DiskEntry::new(
///     PathBuf::from("/tmp/file.txt"),
///     1024,
///     EntryType::File,
///     1,
/// ));
///
/// // Summarize: show only total
/// let output = render_tree(&dir, true, false, true, false);
/// assert_eq!(output, "5.0K\t/tmp");
///
/// // Show all: directories and files
/// let output = render_tree(&dir, false, true, false, false);
/// let lines: Vec<&str> = output.lines().collect();
/// assert_eq!(lines.len(), 2);
/// assert_eq!(lines[0], "1024\t/tmp/file.txt");  // Child first
/// assert_eq!(lines[1], "5120\t/tmp");            // Parent second
/// ```
pub fn render_tree(
    entry: &DiskEntry,
    human_readable: bool,
    show_all: bool,
    summarize: bool,
    use_color: bool,
) -> String {
    let mut lines = Vec::new();

    if summarize {
        // Summarize mode: only show the root
        lines.push(render_entry(entry, human_readable, use_color));
    } else {
        // Normal mode: recursively collect lines
        collect_lines(entry, human_readable, show_all, use_color, &mut lines);
    }

    // Join all lines with newlines
    lines.join("\n")
}

/// Recursively collects output lines for a tree.
///
/// This is the workhorse function that traverses the tree and builds
/// the output line-by-line.
///
/// # Recursion Strategy
///
/// 1. Recurse into children first (depth-first)
/// 2. Then add current entry
///
/// This produces `du`-style output where children appear before parents.
///
/// # Filtering
///
/// - Directories always shown
/// - Files shown only if `show_all = true`
/// - Other entry types treated like files
///
/// # Arguments
///
/// * `entry` - Current entry to process
/// * `human_readable` - Format sizes?
/// * `show_all` - Show files?
/// * `use_color` - Colorize output?
/// * `lines` - Output accumulator (mutated)
///
/// # Examples
///
/// ```ignore
/// let mut lines = Vec::new();
/// collect_lines(&root, true, false, false, &mut lines);
/// for line in lines {
///     println!("{}", line);
/// }
/// ```
fn collect_lines(
    entry: &DiskEntry,
    human_readable: bool,
    show_all: bool,
    use_color: bool,
    lines: &mut Vec<String>,
) {
    // Determine if we should show this entry
    let should_show = match entry.entry_type {
        EntryType::Directory => true,      // Always show directories
        _ => show_all,                      // Files only if show_all
    };

    // Recurse into children first (depth-first traversal)
    // This ensures children appear before parents in output
    for child in &entry.children {
        collect_lines(child, human_readable, show_all, use_color, lines);
    }

    // After processing children, add this entry
    if should_show {
        lines.push(render_entry(entry, human_readable, use_color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_file(name: &str, size: u64) -> DiskEntry {
        DiskEntry::new(PathBuf::from(name), size, EntryType::File, 1)
    }

    fn make_dir(name: &str, size: u64, children: Vec<DiskEntry>) -> DiskEntry {
        let mut entry = DiskEntry::new(PathBuf::from(name), size, EntryType::Directory, 0);
        entry.children = children;
        entry
    }

    #[test]
    fn test_render_entry_no_color_raw() {
        let entry = make_file("test.txt", 1024);
        let result = render_entry(&entry, false, false);
        assert_eq!(result, "1024\ttest.txt");
    }

    #[test]
    fn test_render_entry_no_color_human() {
        let entry = make_file("test.txt", 1024);
        let result = render_entry(&entry, true, false);
        assert_eq!(result, "1.0K\ttest.txt");
    }

    #[test]
    fn test_render_entry_directory_total_size() {
        let dir = make_dir(
            "/mydir",
            4096,
            vec![
                make_file("/mydir/a.txt", 100),
                make_file("/mydir/b.txt", 200),
            ],
        );
        let result = render_entry(&dir, false, false);
        // total_size = 4096 + 100 + 200 = 4396
        assert_eq!(result, "4396\t/mydir");
    }

    #[test]
    fn test_render_tree_summarize() {
        let dir = make_dir("/mydir", 4096, vec![make_file("/mydir/a.txt", 100)]);
        let result = render_tree(&dir, false, false, true, false);
        assert_eq!(result, "4196\t/mydir");
    }

    #[test]
    fn test_render_tree_directories_only() {
        let dir = make_dir("/root", 100, vec![make_file("/root/file.txt", 50)]);
        let result = render_tree(&dir, false, false, false, false);
        // Only directory should be shown (not file), and du prints children before parent
        assert_eq!(result, "150\t/root");
    }

    #[test]
    fn test_render_tree_show_all() {
        let dir = make_dir("/root", 100, vec![make_file("/root/file.txt", 50)]);
        let result = render_tree(&dir, false, true, false, false);
        let lines: Vec<&str> = result.lines().collect();
        // du order: children first, then parent
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "50\t/root/file.txt");
        assert_eq!(lines[1], "150\t/root");
    }

    #[test]
    fn test_render_tree_nested_dirs() {
        let inner = make_dir(
            "/root/sub",
            200,
            vec![make_file("/root/sub/data.bin", 1000)],
        );
        let mut root = DiskEntry::new(PathBuf::from("/root"), 100, EntryType::Directory, 0);
        root.children.push(inner);

        let result = render_tree(&root, false, false, false, false);
        let lines: Vec<&str> = result.lines().collect();
        // sub printed before root (du order)
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("/root/sub"));
        assert!(lines[1].contains("/root"));
    }

    #[test]
    fn test_render_entry_with_color() {
        // Just verify it doesn't panic and produces non-empty output
        let entry = make_file("test.txt", 500);
        let result = render_entry(&entry, true, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_colorize_size_ranges() {
        // These just verify no panics; actual ANSI codes are hard to test
        let _ = colorize_size("1.0G", 2 * GB);
        let _ = colorize_size("150M", 150 * MB);
        let _ = colorize_size("5.0M", 5 * MB);
        let _ = colorize_size("10K", 10 * KB);
        let _ = colorize_size("500B", 500);
    }

    #[test]
    fn test_colorize_path_variants() {
        let _ = colorize_path("/dir", &EntryType::Directory);
        let _ = colorize_path("file.txt", &EntryType::File);
        let _ = colorize_path("link", &EntryType::Symlink);
        let _ = colorize_path("other", &EntryType::Other);
    }

    #[test]
    fn test_render_tree_empty_dir() {
        let dir = DiskEntry::new(PathBuf::from("/empty"), 4096, EntryType::Directory, 0);
        let result = render_tree(&dir, true, false, false, false);
        assert_eq!(result, "4.0K\t/empty");
    }

    #[test]
    fn test_render_tree_human_readable() {
        let dir = make_dir("/data", 0, vec![make_file("/data/big.bin", 5 * MB)]);
        let result = render_tree(&dir, true, true, false, false);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("5.0M"));
        assert!(lines[1].contains("5.0M")); // dir total includes child
    }
}
