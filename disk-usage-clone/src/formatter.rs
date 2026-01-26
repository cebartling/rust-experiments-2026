const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;
const TB: u64 = 1024 * GB;

pub fn format_size(bytes: u64, human_readable: bool) -> String {
    if !human_readable {
        return bytes.to_string();
    }

    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

pub fn format_size_padded(bytes: u64, human_readable: bool, width: usize) -> String {
    let size_str = format_size(bytes, human_readable);
    format!("{size_str:>width$}")
}

pub fn format_entry_line(
    size_str: &str,
    path_str: &str,
    depth: usize,
    indent_width: usize,
) -> String {
    let indent = " ".repeat(depth * indent_width);
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
