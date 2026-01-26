//! Core data structures for representing filesystem entries.
//!
//! This module defines the tree structure used to represent disk usage:
//! - [`DiskEntry`]: A node in the filesystem tree
//! - [`EntryType`]: File, directory, symlink, or other
//! - [`SortOrder`]: How to sort entries in the tree
//!
//! # Tree Structure
//!
//! The [`DiskEntry`] forms a tree where:
//! - Each node represents a file or directory
//! - Directories have children (files and subdirectories)
//! - The tree mirrors the filesystem hierarchy
//!
//! # Examples
//!
//! ```
//! use disk_usage_clone::entry::{DiskEntry, EntryType};
//! use std::path::PathBuf;
//!
//! // Create a directory entry
//! let mut dir = DiskEntry::new(
//!     PathBuf::from("/home/user"),
//!     4096,                    // Directory size (metadata)
//!     EntryType::Directory,
//!     0,                       // Depth in tree
//! );
//!
//! // Add children
//! dir.children.push(DiskEntry::new(
//!     PathBuf::from("/home/user/file.txt"),
//!     1024,
//!     EntryType::File,
//!     1,
//! ));
//!
//! // Calculate total size (includes children)
//! assert_eq!(dir.total_size(), 4096 + 1024);
//! ```

use std::cmp::Reverse;
use std::path::PathBuf;

/// The type of a filesystem entry.
///
/// Used to distinguish between different kinds of filesystem objects
/// for rendering (e.g., directories are colored differently).
///
/// # Variants
///
/// - `File` - Regular file
/// - `Directory` - Directory (can have children)
/// - `Symlink` - Symbolic link
/// - `Other` - Special files (devices, pipes, sockets, etc.)
///
/// # Examples
///
/// ```
/// use disk_usage_clone::entry::EntryType;
///
/// let file_type = EntryType::File;
/// let dir_type = EntryType::Directory;
///
/// assert_ne!(file_type, dir_type);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

/// Sort order for directory entries.
///
/// Controls how children are sorted within each directory.
/// Sorting is applied recursively throughout the tree.
///
/// # Variants
///
/// - `None` - Preserve filesystem order (no sorting)
/// - `SizeAscending` - Smallest first
/// - `SizeDescending` - Largest first (most useful)
/// - `Name` - Alphabetical by path
///
/// # Examples
///
/// ```
/// use disk_usage_clone::entry::SortOrder;
///
/// // Parse from CLI string
/// assert_eq!(SortOrder::parse("size"), Some(SortOrder::SizeDescending));
/// assert_eq!(SortOrder::parse("size-asc"), Some(SortOrder::SizeAscending));
/// assert_eq!(SortOrder::parse("name"), Some(SortOrder::Name));
/// assert_eq!(SortOrder::parse("invalid"), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    None,
    SizeAscending,
    SizeDescending,
    Name,
}

impl SortOrder {
    /// Parses a string into a `SortOrder`.
    ///
    /// Used to convert CLI arguments into type-safe enum values.
    ///
    /// # Arguments
    ///
    /// * `s` - String to parse ("size", "size-asc", "name", "none")
    ///
    /// # Returns
    ///
    /// - `Some(SortOrder)` if the string is valid
    /// - `None` for invalid/unknown strings
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::entry::SortOrder;
    ///
    /// assert_eq!(SortOrder::parse("size"), Some(SortOrder::SizeDescending));
    /// assert_eq!(SortOrder::parse("none"), Some(SortOrder::None));
    /// assert!(SortOrder::parse("bogus").is_none());
    /// ```
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

/// A node in the disk usage tree.
///
/// Represents a single filesystem entry (file or directory) with its size,
/// type, and optional children. Forms a tree structure mirroring the filesystem.
///
/// # Fields
///
/// - `path`: Full path to this entry
/// - `size_bytes`: Size in bytes (just this entry, not including children)
/// - `entry_type`: File, directory, symlink, or other
/// - `depth`: Depth in the tree (0 = root)
/// - `children`: Child entries (empty for files)
///
/// # Tree Structure
///
/// ```text
/// DiskEntry("/home")              size_bytes: 4096
///   ├─ DiskEntry("/home/user")    size_bytes: 4096
///   │  ├─ DiskEntry("file.txt")   size_bytes: 1024
///   │  └─ DiskEntry("data.bin")   size_bytes: 2048
///   └─ DiskEntry("/home/other")   size_bytes: 4096
/// ```
///
/// # Examples
///
/// ## Creating entries
///
/// ```
/// use disk_usage_clone::entry::{DiskEntry, EntryType};
/// use std::path::PathBuf;
///
/// let entry = DiskEntry::new(
///     PathBuf::from("/tmp/file.txt"),
///     1024,                   // 1KB file
///     EntryType::File,
///     0,                      // Root depth
/// );
///
/// assert_eq!(entry.size_bytes, 1024);
/// assert!(entry.children.is_empty());
/// ```
///
/// ## Building a tree
///
/// ```
/// use disk_usage_clone::entry::{DiskEntry, EntryType};
/// use std::path::PathBuf;
///
/// let mut dir = DiskEntry::new(
///     PathBuf::from("/home"),
///     4096,
///     EntryType::Directory,
///     0,
/// );
///
/// // Add children
/// dir.children.push(DiskEntry::new(
///     PathBuf::from("/home/file.txt"),
///     1024,
///     EntryType::File,
///     1,
/// ));
///
/// // Total includes directory + children
/// assert_eq!(dir.total_size(), 4096 + 1024);
/// ```
#[derive(Debug, Clone)]
pub struct DiskEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub entry_type: EntryType,
    pub depth: usize,
    pub children: Vec<DiskEntry>,
}

impl DiskEntry {
    /// Creates a new `DiskEntry`.
    ///
    /// # Arguments
    ///
    /// * `path` - Full path to this entry
    /// * `size_bytes` - Size in bytes (for files: file size, for dirs: metadata size)
    /// * `entry_type` - File, directory, symlink, or other
    /// * `depth` - Depth in the tree (0 = root)
    ///
    /// # Returns
    ///
    /// A new `DiskEntry` with no children.
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::entry::{DiskEntry, EntryType};
    /// use std::path::PathBuf;
    ///
    /// let file = DiskEntry::new(
    ///     PathBuf::from("/tmp/data.bin"),
    ///     2048,
    ///     EntryType::File,
    ///     0,
    /// );
    ///
    /// assert_eq!(file.size_bytes, 2048);
    /// assert_eq!(file.depth, 0);
    /// assert!(file.children.is_empty());
    /// ```
    pub fn new(path: PathBuf, size_bytes: u64, entry_type: EntryType, depth: usize) -> Self {
        DiskEntry {
            path,
            size_bytes,
            entry_type,
            depth,
            children: Vec::new(),
        }
    }

    /// Calculates the total size including all descendants.
    ///
    /// For files, returns `size_bytes`. For directories, recursively
    /// sums the directory's own size plus all children's total sizes.
    ///
    /// # Returns
    ///
    /// Total size in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::entry::{DiskEntry, EntryType};
    /// use std::path::PathBuf;
    ///
    /// // File: total_size = size_bytes
    /// let file = DiskEntry::new(
    ///     PathBuf::from("file.txt"),
    ///     500,
    ///     EntryType::File,
    ///     0,
    /// );
    /// assert_eq!(file.total_size(), 500);
    ///
    /// // Directory: total_size = own size + all children
    /// let mut dir = DiskEntry::new(
    ///     PathBuf::from("dir"),
    ///     4096,
    ///     EntryType::Directory,
    ///     0,
    /// );
    /// dir.children.push(DiskEntry::new(
    ///     PathBuf::from("dir/a.txt"),
    ///     100,
    ///     EntryType::File,
    ///     1,
    /// ));
    /// dir.children.push(DiskEntry::new(
    ///     PathBuf::from("dir/b.txt"),
    ///     200,
    ///     EntryType::File,
    ///     1,
    /// ));
    /// assert_eq!(dir.total_size(), 4096 + 100 + 200);
    /// ```
    ///
    /// # Performance
    ///
    /// This is a recursive operation. For deep trees, it may be called
    /// multiple times during rendering. Consider caching if performance
    /// becomes an issue.
    pub fn total_size(&self) -> u64 {
        if self.children.is_empty() {
            // Leaf node: just return own size
            self.size_bytes
        } else {
            // Directory: own size + sum of all children's total sizes
            self.size_bytes + self.children.iter().map(|c| c.total_size()).sum::<u64>()
        }
    }

    /// Collapses the tree to a maximum depth.
    ///
    /// Entries at `max_depth` have their descendants "collapsed" into them:
    /// - All child sizes are summed into `size_bytes`
    /// - The `children` vector is cleared
    ///
    /// This is useful for implementing the `--max-depth` flag, which limits
    /// how deep into the tree to show detail.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum depth to preserve
    ///   - `0` means only show the root
    ///   - `1` means show root and immediate children
    ///   - etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::entry::{DiskEntry, EntryType};
    /// use std::path::PathBuf;
    ///
    /// let mut root = DiskEntry::new(
    ///     PathBuf::from("/root"),
    ///     100,
    ///     EntryType::Directory,
    ///     0,
    /// );
    ///
    /// let mut sub = DiskEntry::new(
    ///     PathBuf::from("/root/sub"),
    ///     200,
    ///     EntryType::Directory,
    ///     1,
    /// );
    /// sub.children.push(DiskEntry::new(
    ///     PathBuf::from("/root/sub/deep"),
    ///     300,
    ///     EntryType::File,
    ///     2,
    /// ));
    /// root.children.push(sub);
    ///
    /// let original_total = root.total_size(); // 100 + 200 + 300 = 600
    ///
    /// // Collapse to depth 1 (show root and immediate children only)
    /// root.collapse_to_depth(1);
    ///
    /// // Root still has "sub" as a child
    /// assert_eq!(root.children.len(), 1);
    /// // But "sub" has no children (collapsed)
    /// assert!(root.children[0].children.is_empty());
    /// // "sub" absorbed its child's size
    /// assert_eq!(root.children[0].size_bytes, 200 + 300);
    /// // Total size preserved
    /// assert_eq!(root.total_size(), original_total);
    /// ```
    pub fn collapse_to_depth(&mut self, max_depth: usize) {
        self.collapse_recursive(0, max_depth);
    }

    /// Recursive helper for `collapse_to_depth`.
    ///
    /// Traverses the tree and collapses nodes at or beyond `max_depth`.
    ///
    /// # Arguments
    ///
    /// * `current_depth` - Depth of this node (0 = root)
    /// * `max_depth` - Maximum allowed depth
    fn collapse_recursive(&mut self, current_depth: usize, max_depth: usize) {
        if current_depth >= max_depth {
            // We're at or past max depth: collapse all descendants
            // Calculate total size including all descendants
            self.size_bytes = self.total_size();
            // Remove all children (they're now part of size_bytes)
            self.children.clear();
        } else {
            // Still within max depth: recurse into children
            for child in &mut self.children {
                child.collapse_recursive(current_depth + 1, max_depth);
            }
        }
    }

    /// Sorts children according to the specified order.
    ///
    /// Sorting is applied recursively to the entire tree.
    /// Each directory's children are sorted independently.
    ///
    /// # Arguments
    ///
    /// * `order` - How to sort children
    ///
    /// # Examples
    ///
    /// ```
    /// use disk_usage_clone::entry::{DiskEntry, EntryType, SortOrder};
    /// use std::path::PathBuf;
    ///
    /// let mut dir = DiskEntry::new(
    ///     PathBuf::from("/dir"),
    ///     0,
    ///     EntryType::Directory,
    ///     0,
    /// );
    ///
    /// // Add children in random order
    /// dir.children.push(DiskEntry::new(
    ///     PathBuf::from("/dir/small"),
    ///     100,
    ///     EntryType::File,
    ///     1,
    /// ));
    /// dir.children.push(DiskEntry::new(
    ///     PathBuf::from("/dir/large"),
    ///     9999,
    ///     EntryType::File,
    ///     1,
    /// ));
    /// dir.children.push(DiskEntry::new(
    ///     PathBuf::from("/dir/medium"),
    ///     500,
    ///     EntryType::File,
    ///     1,
    /// ));
    ///
    /// // Sort by size (largest first)
    /// dir.sort_entries(&SortOrder::SizeDescending);
    ///
    /// let sizes: Vec<u64> = dir.children.iter()
    ///     .map(|c| c.size_bytes)
    ///     .collect();
    /// assert_eq!(sizes, vec![9999, 500, 100]);
    /// ```
    ///
    /// # Performance
    ///
    /// Uses Rust's `sort_by_key` which is O(n log n). For trees with many
    /// children at each level, this may add noticeable overhead.
    pub fn sort_entries(&mut self, order: &SortOrder) {
        // First, recursively sort all descendants
        for child in &mut self.children {
            child.sort_entries(order);
        }

        // Then sort this node's children
        match order {
            SortOrder::None => {
                // No sorting: preserve filesystem order
            }
            SortOrder::SizeAscending => {
                // Smallest first (least common, but useful for finding small files)
                self.children.sort_by_key(|e| e.total_size());
            }
            SortOrder::SizeDescending => {
                // Largest first (most useful for finding large directories)
                // Uses Reverse to invert the comparison
                self.children.sort_by_key(|e| Reverse(e.total_size()));
            }
            SortOrder::Name => {
                // Alphabetical by full path
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
