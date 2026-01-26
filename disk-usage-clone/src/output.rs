use colored::Colorize;

use crate::entry::{DiskEntry, EntryType};
use crate::formatter::format_size;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

fn colorize_size(size_str: &str, bytes: u64) -> String {
    if bytes >= GB {
        size_str.red().bold().to_string()
    } else if bytes >= 100 * MB {
        size_str.yellow().bold().to_string()
    } else if bytes >= MB {
        size_str.yellow().to_string()
    } else if bytes >= KB {
        size_str.green().to_string()
    } else {
        size_str.dimmed().to_string()
    }
}

fn colorize_path(path_str: &str, entry_type: &EntryType) -> String {
    match entry_type {
        EntryType::Directory => path_str.blue().bold().to_string(),
        EntryType::Symlink => path_str.cyan().to_string(),
        EntryType::File | EntryType::Other => path_str.to_string(),
    }
}

pub fn render_entry(entry: &DiskEntry, human_readable: bool, use_color: bool) -> String {
    let size = entry.total_size();
    let size_str = format_size(size, human_readable);
    let path_str = entry.path.display().to_string();

    if use_color {
        let colored_size = colorize_size(&size_str, size);
        let colored_path = colorize_path(&path_str, &entry.entry_type);
        format!("{colored_size}\t{colored_path}")
    } else {
        format!("{size_str}\t{path_str}")
    }
}

pub fn render_tree(
    entry: &DiskEntry,
    human_readable: bool,
    show_all: bool,
    summarize: bool,
    use_color: bool,
) -> String {
    let mut lines = Vec::new();

    if summarize {
        lines.push(render_entry(entry, human_readable, use_color));
    } else {
        collect_lines(entry, human_readable, show_all, use_color, &mut lines);
    }

    lines.join("\n")
}

fn collect_lines(
    entry: &DiskEntry,
    human_readable: bool,
    show_all: bool,
    use_color: bool,
    lines: &mut Vec<String>,
) {
    let should_show = match entry.entry_type {
        EntryType::Directory => true,
        _ => show_all,
    };

    // Recurse into children first (du prints children before parent directories)
    for child in &entry.children {
        collect_lines(child, human_readable, show_all, use_color, lines);
    }

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
