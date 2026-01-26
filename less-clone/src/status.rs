//! Status line formatting for the pager.
//!
//! Renders a status line at the bottom of the terminal showing filename,
//! current line position, total lines, scroll percentage, and active
//! search pattern.

/// Format the status line for display.
///
/// # Arguments
/// * `filename` - Optional filename to display
/// * `top_line` - The 0-based index of the first visible line
/// * `visible_lines` - Number of lines visible on screen
/// * `total_lines` - Total number of lines in the buffer
/// * `search_pattern` - Optional active search pattern
pub fn format_status(
    filename: Option<&str>,
    top_line: usize,
    visible_lines: usize,
    total_lines: usize,
    search_pattern: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    // Filename or [stdin]
    let name = filename.unwrap_or("[stdin]");
    parts.push(name.to_string());

    if total_lines == 0 {
        parts.push("(empty)".to_string());
    } else {
        // Line position: display 1-based
        let first = top_line + 1;
        let last = (top_line + visible_lines).min(total_lines);
        parts.push(format!("lines {first}-{last}/{total_lines}"));

        // Percentage
        let pct = format_percentage(top_line, visible_lines, total_lines);
        parts.push(pct);
    }

    // Search pattern
    if let Some(pattern) = search_pattern {
        parts.push(format!("/{pattern}"));
    }

    parts.join(" ")
}

/// Calculate and format scroll percentage.
fn format_percentage(top_line: usize, visible_lines: usize, total_lines: usize) -> String {
    if total_lines == 0 {
        return "0%".to_string();
    }

    let bottom = top_line + visible_lines;

    if top_line == 0 && bottom >= total_lines {
        "(all)".to_string()
    } else if top_line == 0 {
        "(top)".to_string()
    } else if bottom >= total_lines {
        "(end)".to_string()
    } else {
        let pct = ((bottom as f64 / total_lines as f64) * 100.0) as usize;
        format!("{pct}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_with_filename() {
        let status = format_status(Some("test.txt"), 0, 24, 100, None);
        assert!(status.contains("test.txt"));
    }

    #[test]
    fn status_without_filename() {
        let status = format_status(None, 0, 24, 100, None);
        assert!(status.contains("[stdin]"));
    }

    #[test]
    fn status_empty_buffer() {
        let status = format_status(Some("empty.txt"), 0, 24, 0, None);
        assert!(status.contains("(empty)"));
    }

    #[test]
    fn status_line_position() {
        let status = format_status(Some("f.txt"), 9, 24, 100, None);
        // Lines 10-33 of 100
        assert!(status.contains("lines 10-33/100"));
    }

    #[test]
    fn status_at_top() {
        let status = format_status(Some("f.txt"), 0, 24, 100, None);
        assert!(status.contains("(top)"));
    }

    #[test]
    fn status_at_end() {
        let status = format_status(Some("f.txt"), 76, 24, 100, None);
        assert!(status.contains("(end)"));
    }

    #[test]
    fn status_all_visible() {
        let status = format_status(Some("f.txt"), 0, 24, 10, None);
        assert!(status.contains("(all)"));
    }

    #[test]
    fn status_with_search() {
        let status = format_status(Some("f.txt"), 0, 24, 100, Some("pattern"));
        assert!(status.contains("/pattern"));
    }

    #[test]
    fn percentage_middle() {
        let pct = format_percentage(50, 24, 100);
        assert_eq!(pct, "74%");
    }
}
