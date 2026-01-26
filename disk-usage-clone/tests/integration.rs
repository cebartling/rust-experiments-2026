use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[allow(deprecated)]
fn cmd() -> Command {
    Command::cargo_bin("disk-usage-clone").unwrap()
}

fn create_test_tree() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

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
fn test_default_run_current_dir() {
    cmd()
        .arg("--no-color")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_run_on_temp_dir() {
    let tmp = create_test_tree();
    cmd()
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_human_readable_flag() {
    let tmp = create_test_tree();
    cmd()
        .arg("-H")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("B").or(predicate::str::contains("K")));
}

#[test]
fn test_summarize_flag() {
    let tmp = create_test_tree();
    let output = cmd()
        .arg("-s")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Summarize produces exactly one line (plus trailing newline from println)
    let non_empty_lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(non_empty_lines.len(), 1);
}

#[test]
fn test_show_all_flag() {
    let tmp = create_test_tree();
    cmd()
        .arg("-a")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("file_a.txt"))
        .stdout(predicate::str::contains("file_b.txt"))
        .stdout(predicate::str::contains("file_c.txt"));
}

#[test]
fn test_max_depth_flag() {
    let tmp = create_test_tree();
    let output = cmd()
        .arg("-d")
        .arg("0")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let non_empty_lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(non_empty_lines.len(), 1);
}

#[test]
fn test_sort_flag() {
    let tmp = create_test_tree();
    cmd()
        .arg("--sort")
        .arg("size")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_sort_name_flag() {
    let tmp = create_test_tree();
    cmd()
        .arg("--sort")
        .arg("name")
        .arg("-a")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_threads_flag() {
    let tmp = create_test_tree();
    cmd()
        .arg("-j")
        .arg("2")
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_nonexistent_path_fails() {
    cmd()
        .arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("path not found"));
}

#[test]
fn test_help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("disk usage analyzer"));
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("dusk"));
}

#[test]
fn test_combined_flags() {
    let tmp = create_test_tree();
    cmd()
        .args(["-H", "-a", "-s", "--no-color"])
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_no_color_produces_clean_output() {
    let tmp = create_test_tree();
    let output = cmd()
        .arg("--no-color")
        .arg(tmp.path().to_str().unwrap())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // No ANSI escape sequences should be present
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI escape codes"
    );
}

#[test]
fn test_multiple_paths() {
    let tmp1 = create_test_tree();
    let tmp2 = create_test_tree();
    cmd()
        .arg("--no-color")
        .arg(tmp1.path().to_str().unwrap())
        .arg(tmp2.path().to_str().unwrap())
        .assert()
        .success();
}
