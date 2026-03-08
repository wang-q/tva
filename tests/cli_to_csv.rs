#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use tempfile::Builder;

#[test]
fn to_csv_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&["to", "csv"])
        .stdin("a\tb\n1\t2\n")
        .run();

    assert_eq!(stdout, "a,b\n1,2\n");
}

#[test]
fn to_csv_custom_delimiter() {
    let (stdout, _) = TvaCmd::new()
        .args(&["to", "csv", "--delimiter", ";"])
        .stdin("a\tb\n1\t2\n")
        .run();

    assert_eq!(stdout, "a;b\n1;2\n");
}

#[test]
fn to_csv_with_quotes() {
    let (stdout, _) = TvaCmd::new()
        .args(&["to", "csv"])
        .stdin("a\tb\n1,2\t3\n")
        .run();

    assert_eq!(stdout, "a,b\n\"1,2\",3\n");
}

#[test]
fn to_csv_file() {
    let file = Builder::new().suffix(".tsv").tempfile().unwrap();
    let file_path = file.path().to_str().unwrap();
    fs::write(file_path, "a\tb\n1\t2\n").unwrap();

    let (stdout, _) = TvaCmd::new().args(&["to", "csv", file_path]).run();

    assert_eq!(stdout, "a,b\n1,2\n");
}

#[test]
fn to_csv_invalid_delimiter_length() {
    // Test multi-byte delimiter error (covers L48-49)
    let (_, stderr) = TvaCmd::new()
        .args(&["to", "csv", "--delimiter", ",,"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn to_csv_invalid_delimiter_word() {
    // Test word delimiter error (covers L48-49)
    let (_, stderr) = TvaCmd::new()
        .args(&["to", "csv", "--delimiter", "comma"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("delimiter must be a single byte"));
}
