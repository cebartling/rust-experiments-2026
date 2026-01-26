pub mod cli;
pub mod entry;
pub mod error;
pub mod formatter;
pub mod output;
pub mod traversal;

use std::path::Path;

use cli::CliArgs;
use error::DuskError;
use output::render_tree;
use traversal::traverse_parallel;

pub fn run(args: &CliArgs) -> Result<(), DuskError> {
    let output = run_to_string(args)?;
    println!("{output}");
    Ok(())
}

pub fn run_to_string(args: &CliArgs) -> Result<String, DuskError> {
    let use_color = !args.no_color;
    let sort_order = args.sort_order();
    let mut results = Vec::new();

    for path_str in &args.paths {
        let path = Path::new(path_str);
        let mut tree = traverse_parallel(path, args.max_depth, args.threads)?;
        tree.sort_entries(&sort_order);

        let output = render_tree(
            &tree,
            args.human_readable,
            args.all,
            args.summarize,
            use_color,
        );
        results.push(output);
    }

    Ok(results.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("hello.txt"), "hello world").unwrap();
        fs::create_dir(tmp.path().join("subdir")).unwrap();
        fs::write(tmp.path().join("subdir/data.bin"), "0123456789").unwrap();
        tmp
    }

    #[test]
    fn test_run_to_string_basic() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "--no-color", tmp.path().to_str().unwrap()]);
        let result = run_to_string(&args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_run_to_string_human_readable() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-H", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Should contain human-readable sizes (ending with B, K, M, etc.)
        assert!(
            output.contains('B') || output.contains('K'),
            "Expected human-readable size in output: {output}"
        );
    }

    #[test]
    fn test_run_to_string_summarize() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-s", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Summarize should produce a single line
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn test_run_to_string_show_all() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from(["dusk", "-a", "--no-color", tmp.path().to_str().unwrap()]);
        let output = run_to_string(&args).unwrap();
        // Should show files too
        assert!(output.contains("hello.txt"));
        assert!(output.contains("data.bin"));
    }

    #[test]
    fn test_run_to_string_nonexistent_path() {
        let args = CliArgs::parse_from(["dusk", "/nonexistent/path/does/not/exist"]);
        let result = run_to_string(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_to_string_sorted_by_name() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from([
            "dusk",
            "-a",
            "--no-color",
            "--sort",
            "name",
            tmp.path().to_str().unwrap(),
        ]);
        let output = run_to_string(&args).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_run_to_string_max_depth() {
        let tmp = create_test_dir();
        let args = CliArgs::parse_from([
            "dusk",
            "-d",
            "0",
            "--no-color",
            tmp.path().to_str().unwrap(),
        ]);
        let output = run_to_string(&args).unwrap();
        // depth 0 = only root, so single line
        assert_eq!(output.lines().count(), 1);
    }

    use clap::Parser;
}
