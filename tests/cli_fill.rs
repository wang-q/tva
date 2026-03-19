#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;
use test_case::test_case;

// ============================================================================
// Basic Fill Tests
// ============================================================================

#[test_case(
    "a\tb\tc\n1\tx\t\n\ty\t20\n2\t\t30\n\tz\t\n",
    &["--header", "--field", "1,2,3"],
    "a\tb\tc\n1\tx\t\n1\ty\t20\n2\ty\t30\n2\tz\t30\n"
    ; "down_basic"
)]
#[test_case(
    "a\tb\n1\t\n\t2\n",
    &["--header", "--field", "1,2", "--value", "0"],
    "a\tb\n1\t0\n0\t2\n"
    ; "const_basic"
)]
#[test_case(
    "a\tb\n1\tNA\nNA\t2\n",
    &["--header", "--field", "1,2", "--value", "0", "--na", "NA"],
    "a\tb\n1\t0\n0\t2\n"
    ; "custom_na"
)]
#[test_case(
    "a\tb\n1\t10\n\t\n2\t\n",
    &["--header", "-f", "1", "-f", "2"],
    "a\tb\n1\t10\n1\t10\n2\t10\n"
    ; "multi_field"
)]
fn fill_basic_tests(input: &str, args: &[&str], expected: &str) {
    let (result, _) = TvaCmd::new().stdin(input).args(&["fill"]).args(args).run();
    assert_eq!(result.trim(), expected.trim());
}

// ============================================================================
// No Header Tests
// ============================================================================

#[test]
fn fill_no_header() {
    let input = "1\t\n\t2\n";
    let expected = "1\t\n1\t2\n";
    let (result, _) = TvaCmd::new().stdin(input).args(&["fill", "-f", "1"]).run();
    assert_eq!(result.trim(), expected.trim());
}

// ============================================================================
// Empty File Tests
// ============================================================================

#[test]
fn fill_empty_file_with_header() {
    let normal_file = "H\n10\n";
    let empty_file = "";

    let temp_dir = tempfile::tempdir().unwrap();
    let p_normal = temp_dir.path().join("normal.tsv");
    let p_empty = temp_dir.path().join("empty.tsv");
    std::fs::write(&p_normal, normal_file).unwrap();
    std::fs::write(&p_empty, empty_file).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "fill",
            "--header",
            "--field",
            "1",
            "--value",
            "0",
            p_empty.to_str().unwrap(),
            p_normal.to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), "H\n10\n");
}

// ============================================================================
// Line Buffered Tests
// ============================================================================

#[test]
fn fill_line_buffered_flush() {
    let input = "H\n10\n\n";
    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "--field", "1", "--line-buffered"])
        .run();
    assert_eq!(stdout.replace("\r\n", "\n"), "H\n10\n10\n");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn fill_invalid_field_spec() {
    let (_, stderr) = TvaCmd::new()
        .stdin("H\n10\n")
        .args(&["fill", "--header", "--field", "NonExistentField"])
        .run_fail();

    assert!(stderr.contains("Field not found in file header"));
}

// ============================================================================
// Multi-file Tests
// ============================================================================

#[test]
fn fill_multi_file_header_handling() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n2\t").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["fill", "--header", "-f", "2", "-v", "0", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t0\n2\t0\n");
}

// ============================================================================
// Header Hash1 Tests
// ============================================================================

#[test]
fn fill_header_hash1_with_comments() {
    let input = "# comment 1\n# comment 2\ncol1\tcol2\n1\t\n\t20\n";
    let expected = "# comment 1\n# comment 2\ncol1\tcol2\n1\t\n1\t20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header-hash1", "--field", "1,2"])
        .run();

    assert_eq!(stdout, expected);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn fill_first_row_is_na_no_prev() {
    let input = "col1\tcol2\n\t10\n20\t\n";
    let expected = "col1\tcol2\n\t10\n20\t10\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "--field", "1,2"])
        .run();

    assert_eq!(stdout, expected);
}
