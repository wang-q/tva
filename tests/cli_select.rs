#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Basic Field Selection Tests
// ============================================================================

#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "1,3"], "a\tc\n1\t3\n" ; "select_fields_by_index_without_header")]
#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "1"], "a\n1\n" ; "select_single_field")]
#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "2,1"], "b\ta\n2\t1\n" ; "select_reorder_fields")]
fn select_basic(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn select_fields_by_name_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "field3,field1",
            "tests/data/select/input_header1.tsv",
        ])
        .run();

    assert_eq!(stdout, "field3\tfield1\n13567\t11567\n23567\t21567\n");
}

// ============================================================================
// Header Name Pattern Tests
// ============================================================================

#[test_case(
    "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n",
    "*_time",
    "elapsed_time\tuser_time\tsystem_time\n57.5\t52.0\t5.5\n52.0\t49.0\t3.0\n"
    ; "wildcard_pattern"
)]
#[test_case(
    "run\telapsed_time\tuser_time\tsystem_time\tmax_memory\n1\t57.5\t52.0\t5.5\t1420\n2\t52.0\t49.0\t3.0\t1270\n",
    "run-user_time",
    "run\telapsed_time\tuser_time\n1\t57.5\t52.0\n2\t52.0\t49.0\n"
    ; "name_range"
)]
#[test_case(
    "test id\trun:id\ttime-stamp\t001\t100\nv1\tv2\tv3\tv4\tv5\n",
    r"test\ id,run\:id,time\-stamp,\001,\100",
    "test id\trun:id\ttime-stamp\t001\t100\nv1\tv2\tv3\tv4\tv5\n"
    ; "special_char_escapes"
)]
fn select_header_name_patterns(input: &str, fields: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-H", "-f", fields])
        .stdin(input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn select_handles_crlf_input_from_stdin() {
    let input = "f1\tf2\n1\t2\r\n3\t4\r\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1,2", "-"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "f1\tf2\n1\t2\n3\t4\n");
}

// ============================================================================
// Other File Input Tests
// ============================================================================

#[test_case(&["select", "-e", "2", "tests/data/select/input_3x3.tsv"], "f1\tf3\n3x3-r1\t31\n3x3-r2\t32\n3x3-r3\t33\n" ; "exclude_field_3x3")]
#[test_case(&["select", "-f", "2,1", "tests/data/select/input_2fields.tsv"], "f2\tf1\ndef\tabc\n456\t123\nDEF\tABC\n" ; "reorder_fields_2fields")]
fn select_other_file_tests(args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Input1 File Tests
// ============================================================================

#[test_case(&["select", "-f", "1", "tests/data/select/input1.tsv"], "f1\n1\n\n3\n4\n5\n6\n7\n8\n" ; "select_first_field")]
#[test_case(&["select", "-f", "2-3", "tests/data/select/input1.tsv"], "f2\tf3\nggg\tUUU\nf1-empty\tCCC\nßßß\tSSS\nsss\tf4-empty\nÀBC\t\n\t\n \t \n0.0\tZ\n" ; "select_field_range")]
#[test_case(&["select", "-e", "1", "tests/data/select/input1.tsv"], "f2\tf3\tf4\nggg\tUUU\t101\nf1-empty\tCCC\t5734\nßßß\tSSS\t 7\nsss\tf4-empty\nÀBC\t\t1367\n\t\tf23-empty\n \t \tf23-space\n0.0\tZ\t1931\n" ; "exclude_first_field")]
fn select_input1_tests(args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Large Index Tests
// ============================================================================

#[test_case(&["select", "-e", "1048576", "tests/data/select/input1.tsv"], "f1\tf2\tf3\tf4\n1\tggg\tUUU\t101\n\tf1-empty\tCCC\t5734\n3\tßßß\tSSS\t 7\n4\tsss\tf4-empty\n5\tÀBC\t\t1367\n6\t\t\tf23-empty\n7\t \t \tf23-space\n8\t0.0\tZ\t1931\n" ; "single_large_index")]
#[test_case(&["select", "-e", "5-1048576", "tests/data/select/input1.tsv"], "f1\tf2\tf3\tf4\n1\tggg\tUUU\t101\n\tf1-empty\tCCC\t5734\n3\tßßß\tSSS\t 7\n4\tsss\tf4-empty\n5\tÀBC\t\t1367\n6\t\t\tf23-empty\n7\t \t \tf23-space\n8\t0.0\tZ\t1931\n" ; "large_range")]
fn select_large_index_noop(args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Alternate Delimiter Tests
// ============================================================================

#[test_case(&["select", "-f", "1", "--delimiter", "^", "tests/data/select/input_2plus_hat_delim.tsv"], "f1\nabc\n\n\n123\n\n" ; "first_field")]
#[test_case(&["select", "-f", "2", "--delimiter", "^", "tests/data/select/input_2plus_hat_delim.tsv"], "f2\ndef\n\n\n456\nabc\n" ; "second_field")]
fn select_alternate_delimiter(args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Empty File Tests
// ============================================================================

#[test_case(&["select", "-f", "1", "tests/data/select/input_emptyfile.tsv"], "" ; "without_header")]
#[test_case(&["select", "-H", "-f", "1", "tests/data/select/input_emptyfile.tsv"], "\n" ; "with_header")]
fn select_empty_file(args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn select_from_multiple_files_without_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "2,1",
            "tests/data/select/input_3x2.tsv",
            "tests/data/select/input_emptyfile.tsv",
            "tests/data/select/input_3x1.tsv",
            "tests/data/select/input_3x0.tsv",
            "tests/data/select/input_3x3.tsv",
        ])
        .run();

    assert_eq!(
        stdout,
        "f2\tf1\n2001\t3x2-r1\n2002\t3x2-r2\nf2\tf1\n201\t3x1-r1\nf2\tf1\nf2\tf1\n21\t3x3-r1\n22\t3x3-r2\n23\t3x3-r3\n"
    );
}

#[test]
fn select_from_multiple_files_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-H",
            "-f",
            "2",
            "tests/data/select/input_header1.tsv",
            "tests/data/select/input_header2.tsv",
            "tests/data/select/input_header3.tsv",
            "tests/data/select/input_header4.tsv",
        ])
        .run();

    assert_eq!(stdout, "field2\n12567\n22567\n12987\n12888\n22888\n");
}

#[test]
fn select_fields_and_exclude_together() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "select",
            "-f",
            "1",
            "-e",
            "2",
            "tests/data/select/input1.tsv",
        ])
        .run();

    assert_eq!(
        stdout,
        "f1\tf3\tf4\n1\tUUU\t101\n\tCCC\t5734\n3\tSSS\t 7\n4\tf4-empty\n5\t\t1367\n6\t\tf23-empty\n7\t \tf23-space\n8\tZ\t1931\n"
    );
}

// ============================================================================
// Field List Error Tests
// ============================================================================

#[test_case(&["select", "-f", "1,", "tests/data/select/input1.tsv"], "empty field list element" ; "trailing_comma")]
#[test_case(&["select", "-f", "field1", "tests/data/select/input1.tsv"], "requires header" ; "name_without_header")]
fn select_field_list_errors(args: &[&str], expected_err: &str) {
    let (_, stderr) = TvaCmd::new().args(args).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}

// ============================================================================
// Unknown Field Name Tests
// ============================================================================

#[test_case(&["select", "-H", "-f", "no_such_field", "tests/data/select/input_header1.tsv"], "Field not found in file header: 'no_such_field'" ; "with_fields_flag")]
#[test_case(&["select", "-H", "-e", "no_such_field", "tests/data/select/input_header1.tsv"], "Field not found in file header: 'no_such_field'" ; "with_exclude_flag")]
fn select_unknown_field_name(args: &[&str], expected_err: &str) {
    let (_, stderr) = TvaCmd::new().args(args).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}

// ============================================================================
// Missing Required Arguments Tests
// ============================================================================

#[test_case(&["select", "tests/data/select/input1.tsv"], "", "one of --fields/-f or --exclude/-e is required" ; "no_args_with_file")]
#[test_case(&["select"], "a\tb\n", "one of --fields/-f or --exclude/-e is required" ; "no_args_with_stdin")]
fn select_missing_required_args(args: &[&str], stdin: &str, expected_err: &str) {
    let (_, stderr) = TvaCmd::new().args(args).stdin(stdin).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}

#[test]
fn select_invalid_delimiter() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "1", "--delimiter", "TAB"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stderr.contains("delimiter must be a single character"));
}

#[test]
fn select_empty_selection() {
    let input = "a\tb\n1\t2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "--exclude", "1,2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("\n\n")); // Two newlines for two rows
}

#[test]
fn select_invalid_field_spec() {
    let (_, stderr) = TvaCmd::new()
        .args(&["select", "-f", "0"])
        .stdin("a\n")
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case("h1\th2\th3\nv1\tv2\tv3\n", &["select", "--header", "--exclude", "2"], "h1\th3\nv1\tv3" ; "exclude_with_header")]
#[test_case("h1\th2\th3\nv1\tv2\tv3\n", &["select", "--header", "--exclude", "h2"], "h1\th3\nv1\tv3" ; "exclude_by_name_with_header")]
fn select_header_modes(input: &str, args: &[&str], expected_substr: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert!(
        stdout.contains(expected_substr),
        "Expected '{}' in stdout, got: {}",
        expected_substr,
        stdout
    );
}

// ============================================================================
// Rest Position Tests
// ============================================================================

#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "2", "--rest", "first"], "a\tc\tb\n1\t3\t2\n" ; "rest_first")]
#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "2", "--rest", "last"], "b\ta\tc\n2\t1\t3\n" ; "rest_last")]
#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "1", "--rest", "none"], "a\n1\n" ; "rest_none")]
fn select_rest_position(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Fields and Exclude Combination Tests
// ============================================================================

#[test_case("a\tb\tc\td\n1\t2\t3\t4\n", &["select", "-f", "2", "-e", "3"], "b\ta\td\n2\t1\t4\n" ; "fields_and_exclude_rest_last")]
#[test_case("a\tb\tc\td\n1\t2\t3\t4\n", &["select", "-f", "2", "-e", "3", "--rest", "first"], "a\td\tb\n1\t4\t2\n" ; "fields_and_exclude_rest_first")]
fn select_fields_and_exclude_combo(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn select_exclude_only_implies_rest() {
    let input = "a\tb\tc\n1\t2\t3\n";
    // -e 2. Implies output all except 2.
    // Output: 1, 3.
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-e", "2"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "a\tc\n1\t3\n");
}

#[test]
fn select_rest_with_header_parsing() {
    let input = "col1\tcol2\tcol3\n1\t2\t3\n";
    // -H -f col2 --rest last
    // Selected: col2 (2). Rest: 1, 3.
    // Order: 2, 1, 3.
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-H", "-f", "col2", "--rest", "last"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "col2\tcol1\tcol3\n2\t1\t3\n");
}

// ============================================================================
// Repeated Fields Tests
// ============================================================================

#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "1,2,1"], "a\tb\ta\n1\t2\t1\n" ; "repeated_fields")]
#[test_case("a\tb\tc\n1\t2\t3\n", &["select", "-f", "1-3,3-1"], "a\tb\tc\tc\tb\ta\n1\t2\t3\t3\t2\t1\n" ; "repeated_fields_with_range")]
fn select_repeated_fields(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(&["select", "-f", "1", "-e", "1"], "a\tb\n", "Field 1 is both selected and excluded" ; "overlap_error")]
#[test_case(&["select", "-f", "-2"], "a\tb\n", "unexpected argument" ; "incomplete_range_start")]
#[test_case(&["select", "-f", "2-"], "a\tb\n", "Incomplete ranges are not supported" ; "incomplete_range_end")]
#[test_case(&["select", "-f", "1,2"], "a\tb\n1\n", "Not enough fields" ; "not_enough_fields")]
#[test_case(&["select", "-e", "1048577"], "a\n", "Maximum allowed '--e|exclude' field number is 1048576" ; "exclude_max_field_limit")]
#[test_case(&["select", "-H", "-f", "h3"], "h1\th2\n1\t2\n", "Field not found in file header: 'h3'" ; "header_no_such_field")]
#[test_case(&["select", "-H", "-f", "h1-h3"], "h1\th2\n1\t2\n", "Second field in range not found" ; "header_range_second_missing")]
fn select_errors(args: &[&str], stdin: &str, expected_err: &str) {
    let (_, stderr) = TvaCmd::new().args(args).stdin(stdin).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}

#[test]
fn select_empty_file_with_header() {
    // Test empty file handling with --header flag (covers L198-199)
    let input = "";
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-H", "-f", "1"])
        .stdin(input)
        .run();
    // Empty file should produce no output
    assert_eq!(stdout, "");
}

#[test]
fn select_fields_and_exclude_no_conflict() {
    // Test fields and exclude without conflict (covers L179-180 check_conflicts)
    let input = "a\tb\tc\td\n1\t2\t3\t4\n";
    // -f 1,3 -e 2 (no overlap)
    // With both --fields and --exclude, the behavior is:
    // --fields specifies selected columns, --exclude removes from the rest
    let (stdout, _) = TvaCmd::new()
        .args(&["select", "-f", "1,3", "-e", "2"])
        .stdin(input)
        .run();
    // Output: 1, 3, 4 (1,3 selected; 2 excluded from rest; 4 is rest)
    assert_eq!(stdout, "a\tc\td\n1\t3\t4\n");
}

// ============================================================================
// Header Hash1 Mode Tests
// ============================================================================

#[test_case("# Comment 1\n# Comment 2\ncol1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n", &["select", "--header-hash1", "-f", "col1,col3"], "col1\tcol3\n1\t3\n4\t6\n" ; "hash1_select_by_name")]
#[test_case("# Metadata\nname\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA\n", &["select", "--header-hash1", "-e", "age"], "name\tcity\nAlice\tNYC\nBob\tLA\n" ; "hash1_exclude_by_name")]
#[test_case("# Comment\nA\tB\tC\n1\t2\t3\n", &["select", "--header-hash1", "-f", "3,1"], "C\tA\n3\t1\n" ; "hash1_select_by_index")]
#[test_case("# File: data.tsv\n# Author: test\n# Date: 2024\nx\ty\tz\n10\t20\t30\n", &["select", "--header-hash1", "-f", "y"], "y\n20\n" ; "hash1_multiple_comment_lines")]
#[test_case("# Comment only\n", &["select", "--header-hash1", "-f", "1"], "" ; "hash1_no_column_names")]
fn select_header_hash1(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}
