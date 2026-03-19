#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use tempfile::TempDir;
use test_case::test_case;

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test_case(
    "10.5\n12.8\n25.0\n10.1\n18.5",
    &["--width", "10", "--field", "1"],
    "10\n10\n20\n10\n10\n"
    ; "basic_numeric"
)]
#[test_case(
    "Price\n10.5\n25.0",
    &["--header", "--width", "10", "--field", "Price"],
    "Price\n10\n20\n"
    ; "header_named"
)]
#[test_case(
    "12\n18\n23",
    &["--width", "10", "--min", "5", "--field", "1"],
    "5\n15\n15\n"
    ; "min_offset"
)]
#[test_case(
    "A\t12\nB\t25",
    &["--width", "10", "--field", "2"],
    "A\t10\nB\t20\n"
    ; "multi_column"
)]
fn bin_basic_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["bin"];
    all_args.extend_from_slice(args);
    let (stdout, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn bin_new_name() {
    let input = "Price\n10.5\n25.0";
    let expected = "Price\tPrice_bin\n10.5\t10\n25.0\t20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&[
            "bin",
            "--header",
            "--width",
            "10",
            "--field",
            "Price",
            "--new-name",
            "Price_bin",
        ])
        .run();

    assert_eq!(normalize_newlines(&stdout), expected);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(
    &["--width", "0", "--field", "1"],
    "Width must be positive"
    ; "width_zero"
)]
#[test_case(
    &["--width=-5", "--field", "1"],
    "Width must be positive"
    ; "width_negative"
)]
fn bin_error_width_tests(args: &[&str], expected_err: &str) {
    let mut all_args = vec!["bin"];
    all_args.extend_from_slice(args);
    let (_, stderr) = TvaCmd::new().args(&all_args).stdin("10\n").run_fail();
    assert!(stderr.contains(expected_err));
}

#[test]
fn bin_error_field_name_requires_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "Price"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr.contains("Field name 'Price' requires --header"));
}

#[test]
fn bin_error_field_not_found_in_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "Missing", "--header"])
        .stdin("Price\n10\n")
        .run_fail();
    assert!(stderr.contains("Field 'Missing' not found in header"));
}

#[test]
fn bin_field_index_zero_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "0"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr.contains("Field index must be >= 1"));
}

// ============================================================================
// Non-Numeric and Boundary Tests
// ============================================================================

#[test_case(
    "10\nNotANum\n30",
    &["--width", "10", "--field", "1"],
    "10\nNotANum\n30\n"
    ; "replace_mode_non_numeric_fallback"
)]
#[test_case(
    "10\tabc\t30\n",
    &["--width", "10", "--field", "2"],
    "10\tabc\t30\n"
    ; "non_numeric_field_passthrough"
)]
#[test_case(
    "10\tabc\t30\n",
    &["--width", "10", "--field", "2", "--new-name", "binned"],
    "10\tabc\t30\t\n"
    ; "new_name_append_mode_non_numeric"
)]
#[test_case(
    "10\t20\n",
    &["--width", "10", "--field", "10"],
    "10\t20\n"
    ; "field_index_too_large"
)]
fn bin_non_numeric_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["bin"];
    all_args.extend_from_slice(args);
    let (stdout, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn bin_new_name_field_index_out_of_bounds() {
    let input = "10\n20\n";
    let expected = "10\t\n20\t\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2", "--new-name", "Bin"])
        .run();

    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn bin_field_skip_logic() {
    let input = "10\t20\t30\n";
    let expected = "10\t20\t30\t30\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "3", "--new-name", "bin"])
        .run();

    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case(
    "# This is a comment\n# Another comment\nPrice\n10.5\n25.0\n",
    &["--header-hash1", "--width", "10", "--field", "Price"],
    "Price\n10\n20\n"
    ; "header_hash1"
)]
#[test_case(
    "",
    &["--width", "10", "--field", "1"],
    ""
    ; "empty_file"
)]
fn bin_header_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["bin"];
    all_args.extend_from_slice(args);
    let (stdout, _) = TvaCmd::new().stdin(input).args(&all_args).run();
    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn bin_field_not_found_with_hash1() {
    let input = "# Comment\nPrice\n10.5\n";

    let (_, stderr) = TvaCmd::new()
        .stdin(input)
        .args(&[
            "bin",
            "--header-hash1",
            "--width",
            "10",
            "--field",
            "NonExistentField",
        ])
        .run_fail();

    assert!(stderr.contains("Field 'NonExistentField' not found in header"));
}

// ============================================================================
// Multiple Files Tests
// ============================================================================

#[test]
fn bin_multiple_files_header() {
    let file1 = "H\n10\n";
    let file2 = "H\n20\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let p1 = temp_dir.path().join("f1.tsv");
    let p2 = temp_dir.path().join("f2.tsv");
    std::fs::write(&p1, file1).unwrap();
    std::fs::write(&p2, file2).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "bin",
            "--width",
            "10",
            "--field",
            "1",
            "--header",
            p1.to_str().unwrap(),
            p2.to_str().unwrap(),
        ])
        .run();

    assert_eq!(normalize_newlines(&stdout), "H\n10\n20\n");
}

#[test]
fn bin_multiple_files_mixed_headers() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");

    fs::write(&file1, "# Comment\nValue\n10.5\n15.2\n").unwrap();
    fs::write(&file2, "Value\n25.7\n30.1\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "bin",
            "--header-hash1",
            "--width",
            "10",
            "--field",
            "Value",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    let expected = "Value\n10\n10\n20\n30\n";
    assert_eq!(normalize_newlines(&stdout), expected);
}

// ============================================================================
// Special Cases
// ============================================================================

#[test]
fn bin_new_name_field_parsing_optimization() {
    let input = "A\t12\tC\nB\t25\tD";
    let expected = "A\t12\tC\t10\nB\t25\tD\t20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2", "--new-name", "Bin"])
        .run();

    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn bin_data_invalid_utf8_in_numeric_field() {
    use assert_cmd::cargo::cargo_bin_cmd;

    let input = b"10\t\xFF\t30\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let assert = cmd
        .args(&["bin", "--width", "10", "-f", "2"])
        .write_stdin(input.as_slice())
        .assert();

    let output = assert.get_output();
    assert_eq!(output.stdout, input);
}
