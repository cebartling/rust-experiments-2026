//! Size formatting utilities.
//!
//! This module provides functions for formatting file sizes in both
//! human-readable (K, M, G, T) and raw byte formats.
//!
//! # Human-Readable Formats
//!
//! Uses binary prefixes (powers of 1024):
//! - 1 KB = 1024 bytes
//! - 1 MB = 1024 KB = 1,048,576 bytes
//! - 1 GB = 1024 MB = 1,073,741,824 bytes
//! - 1 TB = 1024 GB = 1,099,511,627,776 bytes
//!
//! Sizes are displayed with one decimal place for readability.
//!
//! # Examples
//!
//! ```
//! use disk_usage_clone::formatter::format_size;
//!
//! // Human-readable
//! assert_eq!(format_size(1536, true), "1.5K");
//! assert_eq!(format_size(2_097_152, true), "2.0M");
//!
//! // Raw bytes
//! assert_eq!(format_size(1536, false), "1536");
//! ```

/// Kilobyte: 1024 bytes.
///
/// Using binary prefix (KiB) but displaying as "K" for brevity,
/// following the convention of `du` and similar tools.
const KB: u64 = 1024;

/// Megabyte: 1024 kilobytes.
const MB: u64 = 1024 * KB;

/// Gigabyte: 1024 megabytes.
const GB: u64 = 1024 * MB;

/// Terabyte: 1024 gigabytes.
const TB: u64 = 1024 * GB;

/// Formats a size in bytes as a string.
///
/// Can format in two modes:
/// - Human-readable: Uses K, M, G, T suffixes (e.g., "1.5K", "2.3M")
/// - Raw: Plain byte count (e.g., "1536")
///
/// # Arguments
///
/// * `bytes` - Size in bytes to format
/// * `human_readable` - If true, use K/M/G/T suffixes; if false, show raw bytes
///
/// # Returns
///
/// Formatted string ready for display.
///
/// # Human-Readable Format
///
/// - Uses largest applicable unit (TB > GB > MB > KB > B)
/// - Shows one decimal place (e.g., "1.5K")
/// - Sizes < 1KB shown as bytes with "B" suffix (e.g., "512B")
///
/// # Examples
///
/// ```
/// use disk_usage_clone::formatter::format_size;
///
/// // Bytes
/// assert_eq!(format_size(512, true), "512B");
/// assert_eq!(format_size(512, false), "512");
///
/// // Kilobytes
/// assert_eq!(format_size(1024, true), "1.0K");
/// assert_eq!(format_size(1536, true), "1.5K");
///
/// // Megabytes
/// assert_eq!(format_size(1_048_576, true), "1.0M");
/// assert_eq!(format_size(5_242_880, true), "5.0M");
///
/// // Gigabytes
/// assert_eq!(format_size(1_073_741_824, true), "1.0G");
/// assert_eq!(format_size(2_684_354_560, true), "2.5G");
///
/// // Raw bytes
/// assert_eq!(format_size(123456, false), "123456");
/// ```
///
/// # Precision
///
/// Decimal values are formatted with `.1` precision (one decimal place).
/// This balances readability with accuracy:
///
/// ```
/// use disk_usage_clone::formatter::format_size;
///
/// assert_eq!(format_size(1536, true), "1.5K");   // 1.5 KB
/// assert_eq!(format_size(1638, true), "1.6K");   // 1.599... KB
/// assert_eq!(format_size(1740, true), "1.7K");   // 1.699... KB
/// ```
pub fn format_size(bytes: u64, human_readable: bool) -> String {
    // If not human-readable, just return the raw byte count
    if !human_readable {
        return bytes.to_string();
    }

    // Select the appropriate unit and format with one decimal place
    // Check from largest to smallest unit
    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        // Small sizes: show exact bytes with "B" suffix
        format!("{bytes}B")
    }
}

/// Formats a size with padding for alignment.
///
/// Useful for creating columnar output where sizes should align:
///
/// ```text
///     1.5K  /path/one
///   123.4M  /path/two
///     2.3G  /path/three
/// ```
///
/// # Arguments
///
/// * `bytes` - Size in bytes
/// * `human_readable` - Use K/M/G/T suffixes?
/// * `width` - Minimum width (pads with spaces on left)
///
/// # Returns
///
/// Right-aligned string padded to `width` characters.
///
/// # Examples
///
/// ```
/// use disk_usage_clone::formatter::format_size_padded;
///
/// // Right-aligned in 8-character field
/// assert_eq!(format_size_padded(1024, true, 8), "    1.0K");
/// assert_eq!(format_size_padded(1048576, true, 8), "    1.0M");
///
/// // If size is wider than width, no truncation occurs
/// assert_eq!(format_size_padded(12345678, false, 4), "12345678");
/// ```
///
/// # Use Case
///
/// Used for aligning sizes in terminal output:
///
/// ```no_run
/// use disk_usage_clone::formatter::format_size_padded;
///
/// let sizes = vec![1024, 1048576, 512];
/// for size in sizes {
///     let formatted = format_size_padded(size, true, 8);
///     println!("{}  /some/path", formatted);
/// }
/// // Output (aligned):
/// //     1.0K  /some/path
/// //     1.0M  /some/path
/// //     512B  /some/path
/// ```
pub fn format_size_padded(bytes: u64, human_readable: bool, width: usize) -> String {
    // Format the size first
    let size_str = format_size(bytes, human_readable);
    // Right-align within the specified width
    // If size_str is longer than width, no truncation occurs
    format!("{size_str:>width$}")
}

/// Formats a complete entry line with size, indentation, and path.
///
/// Creates output like:
/// ```text
/// 1.5K    ./src/main.rs
/// 2.3M      ./target/debug/app
/// ```
///
/// # Arguments
///
/// * `size_str` - Pre-formatted size string (from `format_size`)
/// * `path_str` - Path to display
/// * `depth` - Depth in tree (0 = root)
/// * `indent_width` - Spaces per level of indentation
///
/// # Returns
///
/// Formatted line with tab-separated size and indented path.
///
/// # Format
///
/// ```text
/// <size_str>\t<indent><path_str>
/// ```
///
/// Where `indent` is `depth * indent_width` spaces.
///
/// # Examples
///
/// ```
/// use disk_usage_clone::formatter::format_entry_line;
///
/// // No indentation (depth 0)
/// let line = format_entry_line("1.0K", "./src", 0, 2);
/// assert_eq!(line, "1.0K\t./src");
///
/// // Indented (depth 2, indent_width 2 = 4 spaces)
/// let line = format_entry_line("512B", "file.txt", 2, 2);
/// assert_eq!(line, "512B\t    file.txt");
///
/// // Custom indent width
/// let line = format_entry_line("1.0M", "data", 1, 4);
/// assert_eq!(line, "1.0M\t    data");
/// ```
///
/// # Visual Example
///
/// With `indent_width = 2`:
///
/// ```text
/// depth=0:  1.5K\t./root
/// depth=1:  512B\t  ./root/child
/// depth=2:  256B\t    ./root/child/grandchild
/// ```
pub fn format_entry_line(
    size_str: &str,
    path_str: &str,
    depth: usize,
    indent_width: usize,
) -> String {
    // Calculate indentation: depth * spaces_per_level
    // E.g., depth=2, indent_width=2 → "    " (4 spaces)
    let indent = " ".repeat(depth * indent_width);

    // Format: size, tab, indented path
    // The tab ensures size and path are clearly separated
    format!("{size_str}\t{indent}{path_str}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_raw_bytes() {
        assert_eq!(format_size(12345, false), "12345");
        assert_eq!(format_size(0, false), "0");
    }

    #[test]
    fn test_format_size_human_bytes() {
        assert_eq!(format_size(0, true), "0B");
        assert_eq!(format_size(512, true), "512B");
        assert_eq!(format_size(1023, true), "1023B");
    }

    #[test]
    fn test_format_size_human_kilobytes() {
        assert_eq!(format_size(1024, true), "1.0K");
        assert_eq!(format_size(1536, true), "1.5K");
        assert_eq!(format_size(10240, true), "10.0K");
    }

    #[test]
    fn test_format_size_human_megabytes() {
        assert_eq!(format_size(1048576, true), "1.0M");
        assert_eq!(format_size(5 * MB, true), "5.0M");
    }

    #[test]
    fn test_format_size_human_gigabytes() {
        assert_eq!(format_size(GB, true), "1.0G");
        assert_eq!(format_size(2 * GB + GB / 2, true), "2.5G");
    }

    #[test]
    fn test_format_size_human_terabytes() {
        assert_eq!(format_size(TB, true), "1.0T");
    }

    #[test]
    fn test_format_size_padded() {
        let result = format_size_padded(1024, true, 8);
        assert_eq!(result, "    1.0K");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_format_size_padded_raw() {
        let result = format_size_padded(12345, false, 10);
        assert_eq!(result, "     12345");
    }

    #[test]
    fn test_format_entry_line_no_indent() {
        let line = format_entry_line("1.0K", "./src", 0, 2);
        assert_eq!(line, "1.0K\t./src");
    }

    #[test]
    fn test_format_entry_line_with_indent() {
        let line = format_entry_line("512B", "file.txt", 2, 2);
        assert_eq!(line, "512B\t    file.txt");
    }

    #[test]
    fn test_format_entry_line_custom_indent_width() {
        let line = format_entry_line("1.0M", "data", 1, 4);
        assert_eq!(line, "1.0M\t    data");
    }
}
