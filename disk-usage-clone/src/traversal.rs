//! Filesystem traversal with parallel metadata collection.
//!
//! This module implements the core disk usage analysis logic using a hybrid approach:
//! 1. Single-threaded directory traversal with `walkdir`
//! 2. Parallel metadata collection with `rayon`
//! 3. Tree construction from flat entry list
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │  walkdir    │  Single-threaded directory enumeration (fast readdir)
//! │   (fast)    │  Collects all DirEntry objects
//! └──────┬──────┘
//!        │
//!        ├──▶ Vec<DirEntry>
//!        │
//! ┌──────▼──────┐
//! │   rayon     │  Parallel metadata collection (.metadata() syscalls)
//! │ (parallel)  │  Converts to FlatEntry with sizes
//! └──────┬──────┘
//!        │
//!        ├──▶ Vec<FlatEntry>
//!        │
//! ┌──────▼──────┐
//! │ build_tree  │  Single-threaded tree construction
//! │  (fast)     │  Assembles DiskEntry tree
//! └──────┬──────┘
//!        │
//!        └──▶ DiskEntry (root)
//! ```
//!
//! # Performance
//!
//! This approach optimizes for the common case where:
//! - Directory enumeration is fast (readdir is quick)
//! - Metadata collection is slow (many stat syscalls)
//! - Tree building is fast (in-memory data structure)
//!
//! Parallelizing metadata collection gives the biggest performance boost.
//!
//! # Examples
//!
//! ## Sequential traversal
//!
//! ```no_run
//! use disk_usage_clone::traversal::traverse;
//! use std::path::Path;
//!
//! let tree = traverse(Path::new("/tmp"), None).expect("Traversal failed");
//! println!("Total size: {} bytes", tree.total_size());
//! ```
//!
//! ## Parallel traversal
//!
//! ```no_run
//! use disk_usage_clone::traversal::traverse_parallel;
//! use std::path::Path;
//!
//! // Use 4 threads for parallel metadata collection
//! let tree = traverse_parallel(
//!     Path::new("/tmp"),
//!     None,       // No max depth limit
//!     Some(4),    // Use 4 threads
//! ).expect("Traversal failed");
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::entry::{DiskEntry, EntryType};
use crate::error::DuskError;

/// Flat representation of a filesystem entry during traversal.
///
/// Used as an intermediate representation before building the tree.
/// Stores all necessary information without parent/child relationships.
///
/// # Why Flat?
///
/// We collect entries in a flat list first because:
/// 1. Easier to parallelize (no shared state)
/// 2. Can process in any order
/// 3. Tree building is deferred until all metadata is collected
struct FlatEntry {
    path: PathBuf,
    size: u64,
    entry_type: EntryType,
    depth: usize,
}

/// Converts a `walkdir::DirEntry` into an `EntryType`.
///
/// Examines the file type and maps it to our enum.
///
/// # Arguments
///
/// * `de` - Directory entry from walkdir
///
/// # Returns
///
/// The corresponding `EntryType`.
///
/// # Examples
///
/// ```ignore
/// let dir_entry = ...; // from walkdir
/// let entry_type = dir_entry_to_entry_type(&dir_entry);
/// ```
fn dir_entry_to_entry_type(de: &walkdir::DirEntry) -> EntryType {
    let ft = de.file_type();
    if ft.is_dir() {
        EntryType::Directory
    } else if ft.is_symlink() {
        EntryType::Symlink
    } else if ft.is_file() {
        EntryType::File
    } else {
        // Special files: devices, pipes, sockets, etc.
        EntryType::Other
    }
}

/// Builds a tree from a flat list of entries.
///
/// This is a key function that converts the flat list collected during
/// traversal into a hierarchical tree structure.
///
/// # Algorithm
///
/// 1. Sort entries by depth (deepest first)
/// 2. Process each entry from deepest to shallowest:
///    - Create DiskEntry for this path
///    - Attach any children already processed
///    - Add to parent's children list (or mark as root)
/// 3. Return the root entry
///
/// # Why Process Deepest First?
///
/// By processing deepest entries first, we ensure children are created
/// before their parents. This allows us to attach children as we build parents.
///
/// # Example
///
/// ```text
/// Input (flat):
///   /home       (depth 0)
///   /home/user  (depth 1)
///   /home/user/file.txt (depth 2)
///
/// Processing order (deepest first):
///   1. file.txt (depth 2) → Create DiskEntry, add to "user" children
///   2. user (depth 1) → Create DiskEntry with file.txt child, add to "home" children
///   3. home (depth 0) → Create DiskEntry with user child, mark as root
/// ```
///
/// # Arguments
///
/// * `flat_entries` - Flat list of entries with sizes and types
///
/// # Returns
///
/// - `Ok(DiskEntry)` - Root of the constructed tree
/// - `Err(DuskError)` - If no entries or no root found
///
/// # Errors
///
/// Returns `TraversalError` if:
/// - Input list is empty
/// - No depth-0 root entry found
fn build_tree(flat_entries: Vec<FlatEntry>) -> Result<DiskEntry, DuskError> {
    if flat_entries.is_empty() {
        return Err(DuskError::TraversalError(
            "no entries found during traversal".to_string(),
        ));
    }

    // Sort by depth descending so children are processed before parents
    // This is critical for the bottom-up tree construction
    let mut sorted = flat_entries;
    sorted.sort_by(|a, b| b.depth.cmp(&a.depth));

    // Map: parent path → list of children
    // As we process deep entries, we add them to their parent's list
    let mut children_map: HashMap<PathBuf, Vec<DiskEntry>> = HashMap::new();

    // The root entry (depth 0) will be set here
    let mut root = None;

    // Process entries from deepest to shallowest
    for entry in sorted {
        // Check if we already have children for this entry (from deeper processing)
        let children = children_map.remove(&entry.path).unwrap_or_default();

        // Create the DiskEntry with its children
        let mut disk_entry = DiskEntry::new(
            entry.path.clone(),
            entry.size,
            entry.entry_type,
            entry.depth,
        );
        disk_entry.children = children;

        if entry.depth == 0 {
            // This is the root - save it
            root = Some(disk_entry);
        } else if let Some(parent) = entry.path.parent() {
            // Not root - add to parent's children list
            children_map
                .entry(parent.to_path_buf())
                .or_default()
                .push(disk_entry);
        }
    }

    // Return the root, or error if we never found a depth-0 entry
    root.ok_or_else(|| DuskError::TraversalError("no root entry found".to_string()))
}

/// Traverses a filesystem path sequentially.
///
/// This is the simpler, single-threaded version. Good for small directories
/// or when you don't want the overhead of thread coordination.
///
/// # Workflow
///
/// 1. Canonicalize path (resolve symlinks, make absolute)
/// 2. Walk directory tree with `walkdir` (single-threaded)
/// 3. Collect metadata for each entry
/// 4. Build tree structure
/// 5. Apply max-depth limit if specified
///
/// # Arguments
///
/// * `path` - Path to analyze (file or directory)
/// * `max_depth` - Optional depth limit (None = unlimited)
///
/// # Returns
///
/// - `Ok(DiskEntry)` - Root of the directory tree
/// - `Err(DuskError)` - Path not found, permission denied, etc.
///
/// # Examples
///
/// ```no_run
/// use disk_usage_clone::traversal::traverse;
/// use std::path::Path;
///
/// // Traverse entire directory
/// let tree = traverse(Path::new("/tmp"), None).unwrap();
/// println!("Total: {} bytes", tree.total_size());
///
/// // Limit depth
/// let tree = traverse(Path::new("/var"), Some(2)).unwrap();
/// println!("Up to 2 levels deep");
/// ```
///
/// # Performance
///
/// Sequential traversal is slower than [`traverse_parallel`] for large
/// directory trees with many files. Use parallel version when performance
/// matters.
///
/// # Errors
///
/// - `PathNotFound` if path doesn't exist
/// - `TraversalError` on permission errors (skips and continues)
pub fn traverse(path: &Path, max_depth: Option<usize>) -> Result<DiskEntry, DuskError> {
    // Canonicalize: convert to absolute path and resolve symlinks
    // This ensures we're working with a real, absolute path
    let root = path
        .canonicalize()
        .map_err(|_| DuskError::PathNotFound(path.to_path_buf()))?;

    // Create walkdir iterator
    // follow_links(false) prevents infinite loops from symlink cycles
    let walker = WalkDir::new(&root).follow_links(false);

    // Walk the tree and collect entries
    // filter_map: keep only successful entries, skip errors
    let flat_entries: Vec<FlatEntry> = walker
        .into_iter()
        .filter_map(|result| match result {
            Ok(dir_entry) => {
                // Successfully read this entry
                let entry_type = dir_entry_to_entry_type(&dir_entry);

                // Get file size (0 for directories, actual size for files)
                // Unwrap metadata, default to 0 on error
                let size = dir_entry.metadata().map(|m| m.len()).unwrap_or(0);

                let depth = dir_entry.depth();

                Some(FlatEntry {
                    path: dir_entry.into_path(),
                    size,
                    entry_type,
                    depth,
                })
            }
            Err(_) => {
                // Error reading this entry (permission denied, etc.)
                // Skip it and continue
                None
            }
        })
        .collect();

    // Build the tree from flat entries
    let mut tree = build_tree(flat_entries)?;

    // Apply depth limit if specified
    if let Some(depth) = max_depth {
        tree.collapse_to_depth(depth);
    }

    Ok(tree)
}

/// Traverses a filesystem path with parallel metadata collection.
///
/// This is the high-performance version that uses rayon to parallelize
/// metadata collection. Recommended for large directory trees.
///
/// # Two-Phase Approach
///
/// **Phase 1 (Sequential):**
/// - Use `walkdir` to enumerate all entries
/// - This is fast (just reading directory structures)
/// - Collect all `DirEntry` objects
///
/// **Phase 2 (Parallel):**
/// - Use `rayon` to call `.metadata()` on all entries in parallel
/// - This is slow (many stat syscalls) but parallelizes well
/// - Convert to `FlatEntry` with sizes
///
/// This approach gives the best of both worlds:
/// - Fast enumeration (walkdir is optimized for this)
/// - Parallel I/O (metadata collection benefits from concurrency)
///
/// # Arguments
///
/// * `path` - Path to analyze (file or directory)
/// * `max_depth` - Optional depth limit (None = unlimited)
/// * `num_threads` - Thread count (None = CPU count)
///
/// # Returns
///
/// Same as [`traverse`].
///
/// # Examples
///
/// ```no_run
/// use disk_usage_clone::traversal::traverse_parallel;
/// use std::path::Path;
///
/// // Auto-detect thread count
/// let tree = traverse_parallel(Path::new("/var"), None, None).unwrap();
///
/// // Explicit thread count
/// let tree = traverse_parallel(
///     Path::new("/var"),
///     Some(3),    // Max depth 3
///     Some(8),    // 8 threads
/// ).unwrap();
/// ```
///
/// # Performance
///
/// Significantly faster than [`traverse`] for:
/// - Large directory trees (1000+ files)
/// - Network filesystems (high latency)
/// - Slow storage (spinning disks)
///
/// May be slower for:
/// - Small directories (<100 files) due to thread overhead
/// - Very fast SSDs where single-threaded is sufficient
///
/// # Thread Pool
///
/// Creates a rayon thread pool with the specified size. If `num_threads`
/// is `None`, rayon uses the number of CPU cores.
///
/// # Errors
///
/// Same as [`traverse`].
pub fn traverse_parallel(
    path: &Path,
    max_depth: Option<usize>,
    num_threads: Option<usize>,
) -> Result<DiskEntry, DuskError> {
    // Canonicalize path
    let root = path
        .canonicalize()
        .map_err(|_| DuskError::PathNotFound(path.to_path_buf()))?;

    // Create walkdir iterator
    let walker = WalkDir::new(&root).follow_links(false);

    // Phase 1: Collect DirEntry objects single-threaded (fast readdir)
    // We keep the DirEntry objects (not consuming them yet) so we can
    // parallelize the metadata collection in phase 2
    let dir_entries: Vec<walkdir::DirEntry> = walker.into_iter().filter_map(|r| r.ok()).collect();

    // Phase 2: Parallel metadata collection using rayon
    // Build thread pool with specified size (or default to CPU count)
    let pool = match num_threads {
        Some(n) => rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .map_err(|e| DuskError::TraversalError(e.to_string()))?,
        None => rayon::ThreadPoolBuilder::new()
            .build()
            .map_err(|e| DuskError::TraversalError(e.to_string()))?,
    };

    // Use the thread pool to process entries in parallel
    let flat_entries: Vec<FlatEntry> = pool.install(|| {
        dir_entries
            .par_iter() // Rayon parallel iterator
            .map(|dir_entry| {
                // Each thread processes a subset of entries
                let entry_type = dir_entry_to_entry_type(dir_entry);

                // The expensive part: stat syscall to get metadata
                // This happens in parallel across all threads
                let size = dir_entry.metadata().map(|m| m.len()).unwrap_or(0);

                let depth = dir_entry.depth();

                FlatEntry {
                    path: dir_entry.path().to_path_buf(),
                    size,
                    entry_type,
                    depth,
                }
            })
            .collect() // Rayon collects in parallel
    });

    // Build the tree from flat entries (single-threaded, fast)
    let mut tree = build_tree(flat_entries)?;

    // Apply depth limit if specified
    if let Some(depth) = max_depth {
        tree.collapse_to_depth(depth);
    }

    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create directory structure:
        // root/
        //   file_a.txt (5 bytes)
        //   subdir/
        //     file_b.txt (10 bytes)
        //     nested/
        //       file_c.txt (20 bytes)
        fs::write(root.join("file_a.txt"), "hello").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join("subdir/file_b.txt"), "0123456789").unwrap();
        fs::create_dir(root.join("subdir/nested")).unwrap();
        fs::write(
            root.join("subdir/nested/file_c.txt"),
            "01234567890123456789",
        )
        .unwrap();

        tmp
    }

    #[test]
    fn test_traverse_returns_root() {
        let tmp = create_test_tree();
        let result = traverse(tmp.path(), None);
        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.entry_type, EntryType::Directory);
        assert_eq!(root.depth, 0);
    }

    #[test]
    fn test_traverse_finds_all_entries() {
        let tmp = create_test_tree();
        let root = traverse(tmp.path(), None).unwrap();

        fn count_entries(entry: &DiskEntry) -> usize {
            1 + entry.children.iter().map(count_entries).sum::<usize>()
        }

        // root + file_a + subdir + file_b + nested + file_c = 6
        assert_eq!(count_entries(&root), 6);
    }

    #[test]
    fn test_traverse_file_sizes() {
        let tmp = create_test_tree();
        let root = traverse(tmp.path(), None).unwrap();

        fn find_entry<'a>(entry: &'a DiskEntry, name: &str) -> Option<&'a DiskEntry> {
            if entry.path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(entry);
            }
            for child in &entry.children {
                if let Some(found) = find_entry(child, name) {
                    return Some(found);
                }
            }
            None
        }

        let file_a = find_entry(&root, "file_a.txt").unwrap();
        assert_eq!(file_a.size_bytes, 5);
        assert_eq!(file_a.entry_type, EntryType::File);

        let file_b = find_entry(&root, "file_b.txt").unwrap();
        assert_eq!(file_b.size_bytes, 10);

        let file_c = find_entry(&root, "file_c.txt").unwrap();
        assert_eq!(file_c.size_bytes, 20);
    }

    #[test]
    fn test_traverse_directory_has_children() {
        let tmp = create_test_tree();
        let root = traverse(tmp.path(), None).unwrap();

        fn find_entry<'a>(entry: &'a DiskEntry, name: &str) -> Option<&'a DiskEntry> {
            if entry.path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(entry);
            }
            for child in &entry.children {
                if let Some(found) = find_entry(child, name) {
                    return Some(found);
                }
            }
            None
        }

        let subdir = find_entry(&root, "subdir").unwrap();
        assert_eq!(subdir.entry_type, EntryType::Directory);
        // subdir has file_b.txt and nested/
        assert_eq!(subdir.children.len(), 2);
    }

    #[test]
    fn test_traverse_max_depth() {
        let tmp = create_test_tree();
        let root = traverse(tmp.path(), Some(1)).unwrap();

        fn max_depth(entry: &DiskEntry) -> usize {
            if entry.children.is_empty() {
                entry.depth
            } else {
                entry
                    .children
                    .iter()
                    .map(max_depth)
                    .max()
                    .unwrap_or(entry.depth)
            }
        }

        // With max_depth=1, we should only see root (depth 0) and immediate children (depth 1)
        assert!(max_depth(&root) <= 1);
    }

    #[test]
    fn test_traverse_nonexistent_path() {
        let result = traverse(Path::new("/nonexistent/path/that/does/not/exist"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_traverse_single_file() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("single.txt");
        fs::write(&file_path, "data").unwrap();

        let result = traverse(&file_path, None).unwrap();
        assert_eq!(result.entry_type, EntryType::File);
        assert_eq!(result.size_bytes, 4);
        assert!(result.children.is_empty());
    }

    #[test]
    fn test_traverse_parallel_returns_root() {
        let tmp = create_test_tree();
        let result = traverse_parallel(tmp.path(), None, Some(2));
        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.entry_type, EntryType::Directory);
        assert_eq!(root.depth, 0);
    }

    #[test]
    fn test_traverse_parallel_finds_all_entries() {
        let tmp = create_test_tree();
        let root = traverse_parallel(tmp.path(), None, Some(2)).unwrap();

        fn count_entries(entry: &DiskEntry) -> usize {
            1 + entry.children.iter().map(count_entries).sum::<usize>()
        }

        assert_eq!(count_entries(&root), 6);
    }

    #[test]
    fn test_traverse_parallel_file_sizes_match_sequential() {
        let tmp = create_test_tree();
        let seq = traverse(tmp.path(), None).unwrap();
        let par = traverse_parallel(tmp.path(), None, Some(4)).unwrap();

        assert_eq!(seq.total_size(), par.total_size());
    }

    #[test]
    fn test_traverse_parallel_with_max_depth() {
        let tmp = create_test_tree();
        let root = traverse_parallel(tmp.path(), Some(1), Some(2)).unwrap();

        fn max_depth(entry: &DiskEntry) -> usize {
            if entry.children.is_empty() {
                entry.depth
            } else {
                entry
                    .children
                    .iter()
                    .map(max_depth)
                    .max()
                    .unwrap_or(entry.depth)
            }
        }

        assert!(max_depth(&root) <= 1);
    }

    #[test]
    fn test_traverse_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let result = traverse(tmp.path(), None).unwrap();
        assert_eq!(result.entry_type, EntryType::Directory);
        assert!(result.children.is_empty());
    }

    #[test]
    fn test_traverse_parallel_nonexistent_path() {
        let result =
            traverse_parallel(Path::new("/nonexistent/path/does/not/exist"), None, Some(2));
        assert!(result.is_err());
    }
}
