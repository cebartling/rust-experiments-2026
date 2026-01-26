#![allow(deprecated)] // cargo_bin deprecation - still functional

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag() {
    Command::cargo_bin("less-clone")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("terminal pager"));
}

#[test]
fn version_flag() {
    Command::cargo_bin("less-clone")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("less-clone"));
}

#[test]
fn nonexistent_file() {
    Command::cargo_bin("less-clone")
        .unwrap()
        .arg("/nonexistent/path/to/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("I/O error"));
}

#[test]
fn pipe_stdin_empty() {
    Command::cargo_bin("less-clone")
        .unwrap()
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No input"));
}
