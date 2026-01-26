use std::cmp::Reverse;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    None,
    SizeAscending,
    SizeDescending,
    Name,
}

impl SortOrder {
    pub fn parse(s: &str) -> Option<SortOrder> {
        match s {
            "none" => Some(SortOrder::None),
            "size" => Some(SortOrder::SizeDescending),
            "size-asc" => Some(SortOrder::SizeAscending),
            "name" => Some(SortOrder::Name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub entry_type: EntryType,
    pub depth: usize,
    pub children: Vec<DiskEntry>,
}

impl DiskEntry {
    pub fn new(path: PathBuf, size_bytes: u64, entry_type: EntryType, depth: usize) -> Self {
        DiskEntry {
            path,
            size_bytes,
            entry_type,
            depth,
            children: Vec::new(),
        }
    }

    pub fn total_size(&self) -> u64 {
        if self.children.is_empty() {
            self.size_bytes
        } else {
            self.size_bytes + self.children.iter().map(|c| c.total_size()).sum::<u64>()
        }
    }

    /// Collapse the tree so no entry exceeds `max_depth`.
    /// Entries at `max_depth` absorb their descendants' sizes into `size_bytes`.
    pub fn collapse_to_depth(&mut self, max_depth: usize) {
        self.collapse_recursive(0, max_depth);
    }

    fn collapse_recursive(&mut self, current_depth: usize, max_depth: usize) {
        if current_depth >= max_depth {
            // Roll up all descendant sizes into this node
            self.size_bytes = self.total_size();
            self.children.clear();
        } else {
            for child in &mut self.children {
                child.collapse_recursive(current_depth + 1, max_depth);
            }
        }
    }

    pub fn sort_entries(&mut self, order: &SortOrder) {
        for child in &mut self.children {
            child.sort_entries(order);
        }
        match order {
            SortOrder::None => {}
            SortOrder::SizeAscending => {
                self.children.sort_by_key(|e| e.total_size());
            }
            SortOrder::SizeDescending => {
                self.children.sort_by_key(|e| Reverse(e.total_size()));
            }
            SortOrder::Name => {
                self.children.sort_by(|a, b| a.path.cmp(&b.path));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_variants() {
        assert_eq!(EntryType::File, EntryType::File);
        assert_eq!(EntryType::Directory, EntryType::Directory);
        assert_eq!(EntryType::Symlink, EntryType::Symlink);
        assert_eq!(EntryType::Other, EntryType::Other);
        assert_ne!(EntryType::File, EntryType::Directory);
    }

    #[test]
    fn test_sort_order_parse() {
        assert_eq!(SortOrder::parse("none"), Some(SortOrder::None));
        assert_eq!(SortOrder::parse("size"), Some(SortOrder::SizeDescending));
        assert_eq!(SortOrder::parse("size-asc"), Some(SortOrder::SizeAscending));
        assert_eq!(SortOrder::parse("name"), Some(SortOrder::Name));
        assert_eq!(SortOrder::parse("invalid"), None);
    }

    #[test]
    fn test_disk_entry_new() {
        let entry = DiskEntry::new(PathBuf::from("/tmp/test"), 1024, EntryType::File, 0);
        assert_eq!(entry.path, PathBuf::from("/tmp/test"));
        assert_eq!(entry.size_bytes, 1024);
        assert_eq!(entry.entry_type, EntryType::File);
        assert_eq!(entry.depth, 0);
        assert!(entry.children.is_empty());
    }

    #[test]
    fn test_total_size_leaf() {
        let entry = DiskEntry::new(PathBuf::from("file.txt"), 500, EntryType::File, 0);
        assert_eq!(entry.total_size(), 500);
    }

    #[test]
    fn test_total_size_with_children() {
        let mut dir = DiskEntry::new(PathBuf::from("/dir"), 4096, EntryType::Directory, 0);
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/a.txt"),
            100,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/b.txt"),
            200,
            EntryType::File,
            1,
        ));
        assert_eq!(dir.total_size(), 4096 + 100 + 200);
    }

    #[test]
    fn test_total_size_nested() {
        let mut root = DiskEntry::new(PathBuf::from("/root"), 4096, EntryType::Directory, 0);
        let mut sub = DiskEntry::new(PathBuf::from("/root/sub"), 4096, EntryType::Directory, 1);
        sub.children.push(DiskEntry::new(
            PathBuf::from("/root/sub/file.txt"),
            1000,
            EntryType::File,
            2,
        ));
        root.children.push(sub);
        assert_eq!(root.total_size(), 4096 + 4096 + 1000);
    }

    #[test]
    fn test_sort_entries_by_size_descending() {
        let mut dir = DiskEntry::new(PathBuf::from("/dir"), 0, EntryType::Directory, 0);
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/small"),
            100,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/large"),
            9999,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/medium"),
            500,
            EntryType::File,
            1,
        ));

        dir.sort_entries(&SortOrder::SizeDescending);
        let sizes: Vec<u64> = dir.children.iter().map(|c| c.size_bytes).collect();
        assert_eq!(sizes, vec![9999, 500, 100]);
    }

    #[test]
    fn test_sort_entries_by_size_ascending() {
        let mut dir = DiskEntry::new(PathBuf::from("/dir"), 0, EntryType::Directory, 0);
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/large"),
            9999,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/small"),
            100,
            EntryType::File,
            1,
        ));

        dir.sort_entries(&SortOrder::SizeAscending);
        let sizes: Vec<u64> = dir.children.iter().map(|c| c.size_bytes).collect();
        assert_eq!(sizes, vec![100, 9999]);
    }

    #[test]
    fn test_sort_entries_by_name() {
        let mut dir = DiskEntry::new(PathBuf::from("/dir"), 0, EntryType::Directory, 0);
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/charlie"),
            100,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/alpha"),
            200,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/bravo"),
            300,
            EntryType::File,
            1,
        ));

        dir.sort_entries(&SortOrder::Name);
        let names: Vec<&str> = dir
            .children
            .iter()
            .map(|c| c.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn test_sort_entries_none_preserves_order() {
        let mut dir = DiskEntry::new(PathBuf::from("/dir"), 0, EntryType::Directory, 0);
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/b"),
            200,
            EntryType::File,
            1,
        ));
        dir.children.push(DiskEntry::new(
            PathBuf::from("/dir/a"),
            100,
            EntryType::File,
            1,
        ));

        dir.sort_entries(&SortOrder::None);
        let names: Vec<&str> = dir
            .children
            .iter()
            .map(|c| c.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["b", "a"]);
    }

    #[test]
    fn test_collapse_to_depth_zero() {
        let mut root = DiskEntry::new(PathBuf::from("/root"), 100, EntryType::Directory, 0);
        root.children.push(DiskEntry::new(
            PathBuf::from("/root/child"),
            200,
            EntryType::File,
            1,
        ));
        let total = root.total_size();
        root.collapse_to_depth(0);
        assert!(root.children.is_empty());
        assert_eq!(root.size_bytes, total);
    }

    #[test]
    fn test_collapse_to_depth_preserves_sizes() {
        let mut root = DiskEntry::new(PathBuf::from("/root"), 100, EntryType::Directory, 0);
        let mut sub = DiskEntry::new(PathBuf::from("/root/sub"), 200, EntryType::Directory, 1);
        sub.children.push(DiskEntry::new(
            PathBuf::from("/root/sub/deep"),
            300,
            EntryType::File,
            2,
        ));
        root.children.push(sub);

        let total_before = root.total_size(); // 100 + 200 + 300 = 600
        root.collapse_to_depth(1);

        // root still has sub as child, but sub has no children
        assert_eq!(root.children.len(), 1);
        assert!(root.children[0].children.is_empty());
        // sub absorbed its descendants' sizes
        assert_eq!(root.children[0].size_bytes, 200 + 300);
        // Total size is preserved
        assert_eq!(root.total_size(), total_before);
    }

    #[test]
    fn test_collapse_to_depth_no_op_when_shallow() {
        let mut root = DiskEntry::new(PathBuf::from("/root"), 100, EntryType::Directory, 0);
        root.children.push(DiskEntry::new(
            PathBuf::from("/root/a"),
            50,
            EntryType::File,
            1,
        ));
        root.collapse_to_depth(5);
        // Tree is shallower than max_depth, so nothing changes
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].size_bytes, 50);
    }
}
