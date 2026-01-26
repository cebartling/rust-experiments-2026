use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::entry::{DiskEntry, EntryType};
use crate::error::DuskError;

struct FlatEntry {
    path: PathBuf,
    size: u64,
    entry_type: EntryType,
    depth: usize,
}

fn dir_entry_to_entry_type(de: &walkdir::DirEntry) -> EntryType {
    let ft = de.file_type();
    if ft.is_dir() {
        EntryType::Directory
    } else if ft.is_symlink() {
        EntryType::Symlink
    } else if ft.is_file() {
        EntryType::File
    } else {
        EntryType::Other
    }
}

fn build_tree(flat_entries: Vec<FlatEntry>) -> Result<DiskEntry, DuskError> {
    if flat_entries.is_empty() {
        return Err(DuskError::TraversalError(
            "no entries found during traversal".to_string(),
        ));
    }

    // Sort by depth descending so children are processed before parents
    let mut sorted = flat_entries;
    sorted.sort_by(|a, b| b.depth.cmp(&a.depth));

    let mut children_map: HashMap<PathBuf, Vec<DiskEntry>> = HashMap::new();
    let mut root = None;

    for entry in sorted {
        let children = children_map.remove(&entry.path).unwrap_or_default();
        let mut disk_entry = DiskEntry::new(
            entry.path.clone(),
            entry.size,
            entry.entry_type,
            entry.depth,
        );
        disk_entry.children = children;

        if entry.depth == 0 {
            root = Some(disk_entry);
        } else if let Some(parent) = entry.path.parent() {
            children_map
                .entry(parent.to_path_buf())
                .or_default()
                .push(disk_entry);
        }
    }

    root.ok_or_else(|| DuskError::TraversalError("no root entry found".to_string()))
}

pub fn traverse(path: &Path, max_depth: Option<usize>) -> Result<DiskEntry, DuskError> {
    let root = path
        .canonicalize()
        .map_err(|_| DuskError::PathNotFound(path.to_path_buf()))?;

    let walker = WalkDir::new(&root).follow_links(false);

    let flat_entries: Vec<FlatEntry> = walker
        .into_iter()
        .filter_map(|result| match result {
            Ok(dir_entry) => {
                let entry_type = dir_entry_to_entry_type(&dir_entry);
                let size = dir_entry.metadata().map(|m| m.len()).unwrap_or(0);
                let depth = dir_entry.depth();
                Some(FlatEntry {
                    path: dir_entry.into_path(),
                    size,
                    entry_type,
                    depth,
                })
            }
            Err(_) => None,
        })
        .collect();

    let mut tree = build_tree(flat_entries)?;
    if let Some(depth) = max_depth {
        tree.collapse_to_depth(depth);
    }
    Ok(tree)
}

pub fn traverse_parallel(
    path: &Path,
    max_depth: Option<usize>,
    num_threads: Option<usize>,
) -> Result<DiskEntry, DuskError> {
    let root = path
        .canonicalize()
        .map_err(|_| DuskError::PathNotFound(path.to_path_buf()))?;

    let walker = WalkDir::new(&root).follow_links(false);

    // Phase 1: Collect DirEntry objects single-threaded (fast readdir)
    let dir_entries: Vec<walkdir::DirEntry> = walker.into_iter().filter_map(|r| r.ok()).collect();

    // Phase 2: Parallel metadata collection using rayon
    let pool = match num_threads {
        Some(n) => rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .map_err(|e| DuskError::TraversalError(e.to_string()))?,
        None => rayon::ThreadPoolBuilder::new()
            .build()
            .map_err(|e| DuskError::TraversalError(e.to_string()))?,
    };

    let flat_entries: Vec<FlatEntry> = pool.install(|| {
        dir_entries
            .par_iter()
            .map(|dir_entry| {
                let entry_type = dir_entry_to_entry_type(dir_entry);
                let size = dir_entry.metadata().map(|m| m.len()).unwrap_or(0);
                let depth = dir_entry.depth();
                FlatEntry {
                    path: dir_entry.path().to_path_buf(),
                    size,
                    entry_type,
                    depth,
                }
            })
            .collect()
    });

    let mut tree = build_tree(flat_entries)?;
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
