#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;
use test_case::test_case;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test_case(
    "a\tb\tc\n1\tx\t10\n1\ty\t20\n2\tx\t30\n2\tz\t40\n",
    &["--header", "--field", "1"],
    "a\tb\tc\n1\tx\t10\n\ty\t20\n2\tx\t30\n\tz\t40"
    ; "basic_with_header"
)]
#[test_case(
    "a\tb\tc\n1\tx\t10\n1\ty\t20\n",
    &["--header", "--field", "1", "--line-buffered"],
    "a\tb\tc\n1\tx\t10\n\ty\t20"
    ; "line_buffered"
)]
#[test_case(
    "1\tx\t10\n1\ty\t20\n2\tx\t30\n2\tz\t40\n",
    &["--field", "1"],
    "1\tx\t10\n\ty\t20\n2\tx\t30\n\tz\t40"
    ; "no_header"
)]
fn blank_basic_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["blank"];
    all_args.extend_from_slice(args);
    let (result, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert_eq!(result.trim(), expected);
}

// ============================================================================
// File Input Tests
// ============================================================================

#[test]
fn blank_from_file() {
    let expected = "a\tb\tc\n1\tx\t10\n\ty\t20\n2\tx\t30\n\tz\t40";
    let (result, _) = TvaCmd::new()
        .args(&[
            "blank",
            "--header",
            "--field",
            "1",
            "tests/data/blank/input1.tsv",
        ])
        .run();
    assert_eq!(result.trim(), expected);
}

#[test]
fn blank_multi_file() {
    let expected =
        "a\tb\tc\n1\tx\t10\n\ty\t20\n2\tx\t30\n\tz\t40\n\t\t50\n3\t\t60\n\tw\t70";
    let (result, _) = TvaCmd::new()
        .args(&[
            "blank",
            "--header",
            "--field",
            "1",
            "--field",
            "2",
            "tests/data/blank/input1.tsv",
            "tests/data/blank/input2.tsv",
        ])
        .run();
    assert_eq!(result.trim(), expected);
}

// ============================================================================
// Replacement Tests
// ============================================================================

#[test_case(
    "a\tb\tc\n1\tx\t10\n1\ty\t20\n2\tx\t30\n2\tz\t40\n",
    &["--header", "--field", "1:---"],
    "a\tb\tc\n1\tx\t10\n---\ty\t20\n2\tx\t30\n---\tz\t40"
    ; "single_replacement"
)]
#[test_case(
    "c1\tc2\nA\t10\nA\t10\nB\t10\n",
    &["--header", "-f", "1:.", "-f", "2:-"],
    "c1\tc2\nA\t10\n.\t-\nB\t-"
    ; "mixed_replacements"
)]
fn blank_replacement_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["blank"];
    all_args.extend_from_slice(args);
    let (result, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert_eq!(result.trim(), expected);
}

// ============================================================================
// Multiple Columns Tests
// ============================================================================

#[test]
fn blank_multiple_columns() {
    let input = "g1\tg2\tval\nA\tX\t1\nA\tX\t2\nA\tY\t3\nB\tY\t4\nB\tY\t5\n";
    let expected = "g1\tg2\tval\nA\tX\t1\n\t\t2\n\tY\t3\nB\t\t4\n\t\t5";

    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1", "--field", "2"])
        .run();

    assert_eq!(result.trim(), expected);
}

// ============================================================================
// Case Insensitive Tests
// ============================================================================

#[test]
fn blank_ignore_case() {
    let input = "a\nA\na\nB\n";
    let expected = "a\nA\n\nB";

    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1", "-i"])
        .run();

    assert_eq!(result.trim(), expected);
}

// ============================================================================
// Multi-File Header Handling Tests
// ============================================================================

#[test]
fn blank_multi_file_header_handling() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["blank", "--header", "-f", "1", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t2\n3\t4\n");
}

#[test]
fn blank_empty_file_handling() {
    let file1 = NamedTempFile::new().unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n1\t2").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["blank", "--header", "-f", "1", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t2\n");
}
