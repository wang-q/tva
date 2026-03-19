#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test_case(
    "ID\tQ1\tQ2\tQ3\nA\t1\t2\t3\nB\t4\t5\t6\nC\t7\t8\t9\n",
    &["--cols", "2-4"],
    "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n"
    ; "basic_longer"
)]
#[test_case(
    "ID\tQ1\tQ2\tQ3\nA\t1\t2\t3\nB\t4\t5\t6\nC\t7\t8\t9\n",
    &["--cols", "2-4", "--names-prefix", "Q"],
    "ID\tname\tvalue\nA\t1\t1\nA\t2\t2\nA\t3\t3\nB\t1\t4\nB\t2\t5\nB\t3\t6\nC\t1\t7\nC\t2\t8\nC\t3\t9\n"
    ; "names_prefix"
)]
#[test_case(
    "ID\tQ1\tExtra\tQ2\nA\t1\tx\t2\nB\t3\ty\t4\n",
    &["--cols", "2,4"],
    "ID\tExtra\tname\tvalue\nA\tx\tQ1\t1\nA\tx\tQ2\t2\nB\ty\tQ1\t3\nB\ty\tQ2\t4\n"
    ; "interleaved_cols"
)]
#[test_case(
    "ID\tnum\ttext\nA\t1\tfoo\nB\t2\tbar\n",
    &["--cols", "2-3"],
    "ID\tname\tvalue\nA\tnum\t1\nA\ttext\tfoo\nB\tnum\t2\nB\ttext\tbar\n"
    ; "mixed_types"
)]
#[test_case(
    "ID\tQ1\tQ2\tQ3\nA\t1\t2\t3\n",
    &["--cols", "2-3"],
    "ID\tQ3\tname\tvalue\nA\t3\tQ1\t1\nA\t3\tQ2\t2\n"
    ; "output_order_row_major"
)]
fn longer_basic_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Column Selection Tests
// ============================================================================

#[test_case(
    &["tests/data/longer/input_mixed.tsv", "--cols", "num-text"],
    "ID\tname\tvalue\nA\tnum\t1\nA\ttext\tfoo\nB\tnum\t2\nB\ttext\tbar\n"
    ; "by_name_range"
)]
#[test_case(
    &["tests/data/longer/input1.tsv", "--cols", "Q*"],
    "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n"
    ; "by_wildcard"
)]
#[test_case(
    &["tests/data/longer/input_dup_cols.tsv", "--cols", "2-3"],
    "ID\textra\tname\tvalue\nA\tx\tval\t1\nA\tx\tval\t2\nB\ty\tval\t3\nB\ty\tval\t4\n"
    ; "duplicate_col_names"
)]
fn longer_col_selection(args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Names Pattern and Separator Tests
// ============================================================================

#[test_case(
    "ID\tnew_sp_m014\tnew_sp_f014\nA\t1\t2\nB\t3\t4\n",
    &["--cols", "2-3", "--names-pattern", "new_?(.*)_(.*)", "--names-to", "diagnosis", "gender_age"],
    "ID\tdiagnosis\tgender_age\tvalue\nA\tsp\tm014\t1\nA\tsp\tf014\t2\nB\tsp\tm014\t3\nB\tsp\tf014\t4\n"
    ; "names_pattern_two_groups"
)]
#[test_case(
    "ID\tnew_sp_m014\tnew_sp_f014\nA\t1\t2\n",
    &["--cols", "2-3", "--names-pattern", "new_(.*)", "--names-to", "diagnosis", "gender_age"],
    "ID\tdiagnosis\tgender_age\tvalue\nA\tsp_m014\t\t1\nA\tsp_f014\t\t2\n"
    ; "pattern_partial_capture"
)]
#[test_case(
    "ID\twk_1\twk_2\nA\t1\t2\nB\t3\t4\n",
    &["--cols", "2-3", "--names-sep", "_", "--names-to", "unit", "num"],
    "ID\tunit\tnum\tvalue\nA\twk\t1\t1\nA\twk\t2\t2\nB\twk\t1\t3\nB\twk\t2\t4\n"
    ; "names_separator"
)]
#[test_case(
    "ID\tcol_A\tcol_B\n1\t2\t3\n",
    &["--cols", "2-3", "--names-pattern", "new_?(.*)_(.*)", "--names-to", "diagnosis", "gender_age"],
    "ID\tdiagnosis\tgender_age\tvalue\n1\tcol_A\t\t2\n1\tcol_B\t\t3\n"
    ; "pattern_no_match_fallback"
)]
fn longer_names_pattern_tests(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Custom Names Tests
// ============================================================================

#[test_case(
    &["--cols", "2-4", "--names-to", "Question", "--values-to", "Answer"],
    "ID\tQuestion\tAnswer\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\n"
    ; "custom_names"
)]
fn longer_custom_names(args: &[&str], expected: &str) {
    let mut all_args = vec!["longer", "tests/data/longer/input1.tsv"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// NA Handling Tests
// ============================================================================

#[test_case(
    &["tests/data/longer/input_na.tsv", "--cols", "2-3"],
    "ID\tname\tvalue\nF\tQ1\t16\nF\tQ2\t\nG\tQ1\t\nG\tQ2\t17\n"
    ; "keep_na"
)]
#[test_case(
    &["tests/data/longer/input_na.tsv", "--cols", "2-3", "--values-drop-na"],
    "ID\tname\tvalue\nF\tQ1\t16\nG\tQ2\t17\n"
    ; "drop_na"
)]
fn longer_na_handling(args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

#[test_case(
    "ID\tQ1\tQ2\nA\t  \t17\nB\t18\t  \n",
    &["--cols", "2-3", "--values-drop-na"],
    "ID\tname\tvalue\nA\tQ1\t  \nA\tQ2\t17\nB\tQ1\t18\nB\tQ2\t  \n"
    ; "whitespace_not_dropped"
)]
fn longer_whitespace_na(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Multi-ID Tests
// ============================================================================

#[test_case(
    &["tests/data/longer/input_multi_id.tsv", "--cols", "3-4"],
    "ID\tCategory\tname\tvalue\nA\tX\tQ1\t1\nA\tX\tQ2\t2\nB\tY\tQ1\t3\nB\tY\tQ2\t4\n"
    ; "multi_id_cols"
)]
fn longer_multi_id(args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test_case(
    &["tests/data/longer/input_only_header.tsv", "--cols", "2-3"],
    "ID\tQ3\tname\tvalue\n"
    ; "only_header_no_data"
)]
#[test_case(
    &["tests/data/longer/input_empty.tsv", "--cols", "2-3"],
    "ID\tname\tvalue\n"
    ; "empty_input"
)]
#[test_case(
    &["tests/data/longer/input_quotes.tsv", "--cols", "2-3"],
    "ID\tname\tvalue\nA\t\"col 1\"\t1\nA\tcol 2\t2\n"
    ; "quotes_in_header"
)]
fn longer_edge_cases(args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Multiple Files Tests
// ============================================================================

#[test]
fn longer_multiple_files() {
    let expected = "ID\tname\tvalue\nA\tQ1\t1\nA\tQ2\t2\nA\tQ3\t3\nB\tQ1\t4\nB\tQ2\t5\nB\tQ3\t6\nC\tQ1\t7\nC\tQ2\t8\nC\tQ3\t9\nD\tQ1\t10\nD\tQ2\t11\nD\tQ3\t12\nE\tQ1\t13\nE\tQ2\t14\nE\tQ3\t15\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input1.tsv",
            "tests/data/longer/input2.tsv",
            "--cols",
            "2-4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_empty_file_skip() {
    let input1 = "ID\tA\tB\n1\ta\tb\n";
    let input2 = "";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.tsv");
    let file2 = temp_dir.path().join("file2.tsv");
    std::fs::write(&file1, input1).unwrap();
    std::fs::write(&file2, input2).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
            "--cols",
            "2-3",
        ])
        .run();

    assert!(stdout.contains("1\tA\ta"));
}

// ============================================================================
// Data Quality Tests
// ============================================================================

#[test_case(
    "ID\tA\tB\n1\ta\tb\n\n2\tc\td\n",
    &["--cols", "2-3"],
    "ID\tname\tvalue\n1\tA\ta\n1\tB\tb\n2\tA\tc\n2\tB\td\n"
    ; "empty_lines_in_data"
)]
#[test_case(
    "ID\tA\tB\n1\ta\n2\ta\tb\tc\n",
    &["--cols", "2-3"],
    "ID\tname\tvalue\n1\tA\ta\n1\tB\t\n2\tA\ta\n2\tB\tb\n"
    ; "ragged_rows"
)]
fn longer_data_quality(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(
    &["tests/data/longer/input_zero_index.tsv", "--cols", "0"],
    "field index must be >= 1"
    ; "invalid_col_index_zero"
)]
#[test_case(
    &["tests/data/longer/input1.tsv", "--cols", "10"],
    "Invalid column index"
    ; "col_index_exceeds_actual"
)]
#[test_case(
    &["tests/data/longer/input1.tsv", "--cols", "99"],
    "Invalid column index"
    ; "invalid_col"
)]
fn longer_errors(args: &[&str], expected_err: &str) {
    let mut all_args = vec!["longer"];
    all_args.extend_from_slice(args);

    let (_, stderr) = TvaCmd::new().args(&all_args).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}

#[test]
fn longer_multi_names_to_without_sep_or_pattern() {
    let (_, stderr) = TvaCmd::new()
        .args(&["longer", "--cols", "2-3", "--names-to", "col1", "col2"])
        .stdin("ID\tA\tB\n1\t2\t3\n")
        .run_fail();

    assert!(stderr.contains("names-sep") || stderr.contains("names-pattern"));
}
