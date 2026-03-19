#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Basic Slice Tests
// ============================================================================

#[test_case("h1\nr1\nr2\nr3\nr4\nr5\n", &["slice", "-r", "3-5"], "r2\nr3\nr4\n" ; "keep_single_range")]
#[test_case("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n", &["slice", "-r", "1-3", "-r", "8-10"], "1\n2\n3\n8\n9\n10\n" ; "keep_multiple_ranges")]
#[test_case("1\n2\n3\n4\n5\n", &["slice", "-r", "3", "--invert"], "1\n2\n4\n5\n" ; "drop_single_row")]
#[test_case("1\n2\n3\n4\n5\n", &["slice", "-r", "4-"], "4\n5\n" ; "open_end_range")]
#[test_case("1\n2\n3\n4\n5\n", &["slice", "-r", "-2"], "1\n2\n" ; "open_start_range")]
fn slice_basic(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case(
    "Header\nData1\nData2\nData3\nData4\n",
    &["slice", "-r", "1-3", "--invert", "--header"],
    "Header\nData3\nData4\n" ; "keep_header_drop_range"
)]
#[test_case(
    "Header\nData1\nData2\nData3\nData4\n",
    &["slice", "-r", "4-5", "--header"],
    "Header\nData3\nData4\n" ; "keep_header_keep_range"
)]
fn slice_header_modes(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test_case(
    "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n",
    &["slice", "--header-lines", "3", "-r", "5"],
    "# Comment 1\n# Comment 2\nCol1\tCol2\nData3\tData4\n" ; "header_lines_n"
)]
#[test_case(
    "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n",
    &["slice", "--header-hash", "-r", "4"],
    "# Comment 1\n# Comment 2\nData1\tData2\n" ; "header_hash"
)]
#[test_case(
    "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\nData3\tData4\n",
    &["slice", "--header-hash1", "-r", "4"],
    "# Comment 1\n# Comment 2\nCol1\tCol2\nData1\tData2\n" ; "header_hash1"
)]
fn slice_advanced_header_modes(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_lines_n_with_invert() {
    let input = "Header1\nHeader2\nData1\nData2\nData3\n";

    let expected = "Header1\nHeader2\nData3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-lines", "2", "-r", "3-4", "--invert"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn slice_header_hash_with_multiple_ranges() {
    let input = "# Comment\nCol1\tCol2\nRow1\nRow2\nRow3\nRow4\n";

    let expected = "# Comment\nRow1\nRow3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["slice", "--header-hash", "-r", "3", "-r", "5"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(&["slice", "-r", "0"], "Row index must be >= 1" ; "invalid_zero_index_single")]
#[test_case(&["slice", "-r", "0-5"], "Row index must be >= 1" ; "invalid_zero_index_range")]
#[test_case(&["slice", "-r", "5-2"], "Invalid range: end < start" ; "invalid_range_order")]
fn slice_error_cases(args: &[&str], expected_error: &str) {
    let (_, stderr) = TvaCmd::new().args(args).stdin("header\n").run_fail();
    assert!(stderr.contains(expected_error));
}

// ============================================================================
// Empty Ranges Behavior Tests
// ============================================================================

#[test_case(&["slice"], "", "keep_nothing" ; "keep_mode_no_ranges")]
#[test_case(&["slice", "--invert"], "1\n2\n3\n", "keep_all" ; "drop_mode_no_ranges")]
#[test_case(&["slice", "--header"], "1\n", "keep_header_only" ; "keep_mode_header_only")]
fn slice_empty_ranges(args: &[&str], expected: &str, _test_name: &str) {
    let input = "1\n2\n3\n";
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Multi-File Input Tests
// ============================================================================

#[test]
fn slice_multi_file_header_duplication() {
    let file1_content = "Header\n1\n2\n";
    let file2_content = "Header\n3\n4\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-H",
            "-r",
            "1-2",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    assert_eq!(
        stdout.matches("Header").count(),
        1,
        "Header should appear exactly once"
    );

    assert!(
        stdout.contains("1\n3\n"),
        "Should contain first data row of each file"
    );
    assert!(
        !stdout.contains("2\n"),
        "Should not contain second data row (L3)"
    );
    assert!(
        !stdout.contains("4\n"),
        "Should not contain second data row (L3)"
    );
}

#[test]
fn slice_multi_file_input() {
    let file1_content = "1\n2\n3\n";
    let file2_content = "4\n5\n6\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-r",
            "1-2",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, "1\n2\n4\n5\n");
}

#[test]
fn slice_multi_file_with_header() {
    let file1_content = "Header\nData1\nData2\n";
    let file2_content = "Header\nData3\nData4\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-H",
            "-r",
            "2",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, "Header\nData1\nData3\n");
    assert_eq!(
        stdout.matches("Header").count(),
        1,
        "Header should appear exactly once"
    );
}

// ============================================================================
// Output File Tests
// ============================================================================

#[test]
fn slice_outfile_option() {
    let input = "1\n2\n3\n4\n5\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-r",
            "2-3",
            "--outfile",
            output_path.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, "");

    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(output_content, "2\n3\n");
}
