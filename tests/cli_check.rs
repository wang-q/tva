#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use tempfile::TempDir;
use test_case::test_case;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test_case("", "0 lines, 0 fields" ; "empty_input")]
#[test_case("A\t1\t!\nB\t2\t@\nC\t3\t#\nD\t4\t$\nE\t5\t%\n", "5 lines, 3 fields" ; "simple_matrix")]
fn check_basic_tests(input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().stdin(input).args(&["check"]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

#[test]
fn check_valid_ctg() {
    let (stdout, _) = TvaCmd::new().args(&["check", "tests/genome/ctg.tsv"]).run();
    assert!(stdout.contains("4 lines, 6 fields"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(
    "a\tb\tc\n1\t2\n",
    "line 2 (2 fields):",
    "tva check: structure check failed: line 2 has 2 fields (expected 3)"
    ; "invalid_structure"
)]
#[test_case(
    "x\ty\n\nu\tv\n",
    "line 2 (0 fields):",
    "tva check: structure check failed: line 2 has 0 fields (expected 2)"
    ; "empty_line_zero_fields"
)]
fn check_error_tests(input: &str, expected_err1: &str, expected_err2: &str) {
    let (_, stderr) = TvaCmd::new().stdin(input).args(&["check"]).run_fail();
    assert!(
        stderr.contains(expected_err1),
        "Expected '{}' in stderr, got: {}",
        expected_err1,
        stderr
    );
    assert!(
        stderr.contains(expected_err2),
        "Expected '{}' in stderr, got: {}",
        expected_err2,
        stderr
    );
}

#[test]
fn check_multiple_files_fail_second() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n").unwrap();
    fs::write(&file2, "a\tb\n1\t2\t3\n").unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["check", file1.to_str().unwrap(), file2.to_str().unwrap()])
        .run_fail();

    assert!(stderr.contains("structure check failed"));
}

#[test]
fn check_file_open_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["check", "non_existent_file_check.tsv"])
        .run_fail();

    assert!(stderr.contains("could not open"));
}

#[test]
fn check_lossy_utf8_error_display() {
    // Tests L41: eprintln!("  {}", String::from_utf8_lossy(record));
    let input = b"col1\tcol2\tcol3\n\xFF\tval2\n";

    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("invalid_utf8.tsv");
    fs::write(&file_path, input).unwrap();

    let (_, stderr) = TvaCmd::new()
        .args(&["check", file_path.to_str().unwrap()])
        .run_fail();

    assert!(stderr.contains("\u{FFFD}"));
    assert!(stderr.contains("line 2 (2 fields)"));
    assert!(stderr.contains("expected 3"));
}

// ============================================================================
// Multiple Files Tests
// ============================================================================

#[test]
fn check_multiple_files_consistent() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "a\tb\n1\t2\n").unwrap();
    fs::write(&file2, "3\t4\n5\t6\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["check", file1.to_str().unwrap(), file2.to_str().unwrap()])
        .run();

    assert!(stdout.contains("4 lines, 2 fields"));
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case(
    "col1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n",
    &["--header"],
    "3 lines total, 2 data lines, 3 fields"
    ; "header_firstline"
)]
#[test_case(
    "# comment\ncol1\tcol2\n1\t2\n3\t4\n",
    &["--header-lines", "2"],
    "4 lines total, 2 data lines"
    ; "header_lines_n"
)]
#[test_case(
    "# comment 1\n# comment 2\n1\t2\t3\n4\t5\t6\n",
    &["--header-hash"],
    "4 lines total, 2 data lines"
    ; "header_hash"
)]
#[test_case(
    "# comment\ncol1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n",
    &["--header-hash1"],
    "4 lines total, 2 data lines, 3 fields"
    ; "header_hash1"
)]
fn check_header_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["check"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

#[test]
fn check_empty_file_with_header() {
    let (stdout, _) = TvaCmd::new().stdin("").args(&["check", "--header"]).run();
    assert!(stdout.contains("0 lines, 0 fields"));
}

#[test]
fn check_multiple_files_with_header() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");
    fs::write(&file1, "col1\tcol2\n1\t2\n").unwrap();
    fs::write(&file2, "col1\tcol2\n3\t4\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "check",
            "--header",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("4 lines total, 2 data lines, 2 fields"));
}

// ============================================================================
// Header Error Tests
// ============================================================================

#[test_case(
    "col1\tcol2\tcol3\n1\t2\n",
    "line 2 (2 fields):",
    "expected 3"
    ; "header_field_mismatch"
)]
#[test_case(
    "\n1\t2\n",
    "line 2 (2 fields):",
    "expected 0"
    ; "header_empty_column_names"
)]
fn check_header_error_tests(input: &str, expected_err1: &str, expected_err2: &str) {
    let (_, stderr) = TvaCmd::new()
        .stdin(input)
        .args(&["check", "--header"])
        .run_fail();

    assert!(
        stderr.contains(expected_err1),
        "Expected '{}' in stderr, got: {}",
        expected_err1,
        stderr
    );
    assert!(
        stderr.contains(expected_err2),
        "Expected '{}' in stderr, got: {}",
        expected_err2,
        stderr
    );
}
