#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn sort_invalid_delimiter() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--delimiter", "TAB"])
        .stdin("a\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn sort_invalid_key() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--key", "0"])
        .stdin("a\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn sort_empty_key_part() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "-k", "1,,2"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains("empty key list element"));
}

#[test]
fn sort_header_modes_mutually_exclusive() {
    let (stdout, stderr) = TvaCmd::new()
        .args(&["sort", "--header", "--header-lines", "2", "-k", "1"])
        .stdin("a\tb\n")
        .run_fail();

    assert!(stdout.is_empty());
    assert!(stderr.contains(
        "only one of --header, --header-lines, --header-hash, --header-hash1"
    ));
}

// ============================================================================
// Basic Sorting Tests
// ============================================================================

#[test]
fn sort_empty_input() {
    let (stdout, _) = TvaCmd::new().args(&["sort"]).stdin("").run();

    assert!(stdout.is_empty());
}

#[test_case("a\t2\nc\t1\nb\t3\n", &["sort", "-k", "1"], "a\t2\nb\t3\nc\t1\n" ; "lexicographic_single_key")]
#[test_case("a\t2\nc\t1\nb\t3\n", &["sort", "-k", "1", "-r"], "c\t1\nb\t3\na\t2\n" ; "reverse_lexicographic")]
#[test_case("b\t2\nb\t1\na\t3\n", &["sort"], "a\t3\nb\t1\nb\t2\n" ; "default_all_columns")]
#[test_case("a,2\nc,1\nb,3\n", &["sort", "-t", ",", "-k", "1"], "a,2\nb,3\nc,1\n" ; "custom_delimiter")]
#[test_case("b\t2\n\na\t1\n", &["sort", "-k", "1"], "\na\t1\nb\t2\n" ; "with_empty_lines")]
#[test_case("b\t2\n", &["sort", "-k", "1"], "b\t2\n" ; "single_row")]
#[test_case(" b\t2\n a\t1\n", &["sort", "-k", "1"], " a\t1\n b\t2\n" ; "with_whitespace")]
fn sort_basic(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Numeric Sorting Tests
// ============================================================================

#[test_case("a\t2\nc\t10\nb\t3\n", "a\t2\nb\t3\nc\t10\n" ; "basic_numeric")]
#[test_case("a\t3.14\nb\t2.71\nc\t1.41\n", "c\t1.41\nb\t2.71\na\t3.14\n" ; "floats")]
#[test_case("a\t-5\nb\t-10\nc\t0\n", "b\t-10\na\t-5\nc\t0\n" ; "negative_numbers")]
#[test_case("a\t999999999\nb\t1000000000\nc\t1\n", "c\t1\na\t999999999\nb\t1000000000\n" ; "large_values")]
#[test_case("a\t1e5\nb\t1e2\nc\t1e3\n", "b\t1e2\nc\t1e3\na\t1e5\n" ; "scientific_notation")]
fn sort_numeric(input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n"])
        .stdin(input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn sort_numeric_with_non_numeric_values() {
    let input = "x\n10\nLETTER\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1", "-n"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "x\nLETTER\n1\n2\n10\n");
}

#[test]
fn sort_numeric_reverse_single_key() {
    let input = "a\t2\nc\t10\nb\t3\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n", "-r"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "c\t10\nb\t3\na\t2\n");
}

#[test]
fn sort_reverse_numeric() {
    let input = "a\t1\nb\t3\nc\t2\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "2", "-n", "-r"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "b\t3\nc\t2\na\t1\n");
}

// ============================================================================
// Multiple Keys Tests
// ============================================================================

#[test_case("a\t2\nc\t1\nb\t1\n", &["sort", "-k", "2,1"], "b\t1\nc\t1\na\t2\n" ; "multiple_keys")]
#[test_case("b\t2\na\t1\na\t2\n", &["sort", "-k", "1-2"], "a\t1\na\t2\nb\t2\n" ; "key_range")]
#[test_case("c\t3\na\t1\nb\t2\n", &["sort", "-k", "1,1"], "a\t1\nb\t2\nc\t3\n" ; "duplicate_indices")]
fn sort_keys(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case(
    "name\tval\nc\t1\na\t2\nb\t3\n",
    &["sort", "--header", "-k", "1"],
    "name\tval\na\t2\nb\t3\nc\t1\n" ; "header_basic"
)]
#[test_case(
    "# Comment 1\n# Comment 2\nc\t1\na\t2\nb\t3\n",
    &["sort", "--header-lines", "2", "-k", "1"],
    "# Comment 1\n# Comment 2\na\t2\nb\t3\nc\t1\n" ; "header_lines_n"
)]
#[test_case(
    "# Comment 1\n# Comment 2\nc\t1\na\t2\nb\t3\n",
    &["sort", "--header-hash", "-k", "1"],
    "# Comment 1\n# Comment 2\na\t2\nb\t3\nc\t1\n" ; "header_hash"
)]
#[test_case(
    "# Comment 1\n# Comment 2\nname\tval\nc\t1\na\t2\nb\t3\n",
    &["sort", "--header-hash1", "-k", "1"],
    "# Comment 1\n# Comment 2\nname\tval\na\t2\nb\t3\nc\t1\n" ; "header_hash1"
)]
#[test_case(
    "name\tval\nc\t1\na\t2\nb\t3\n",
    &["sort", "--header-hash1", "-k", "1"],
    "name\tval\na\t2\nb\t3\nc\t1\n" ; "header_hash1_no_hash_lines"
)]
#[test_case(
    "name\tvalue\n",
    &["sort", "--header", "-k", "1"],
    "name\tvalue\n" ; "header_only_no_data"
)]
#[test_case(
    "# Comment 1\n# Comment 2\n",
    &["sort", "--header-lines", "2", "-k", "1"],
    "# Comment 1\n# Comment 2\n" ; "header_lines_n_no_data"
)]
fn sort_header_modes(input: &str, args: &[&str], expected: &str) {
    let (stdout, _) = TvaCmd::new().args(args).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// File Output Tests
// ============================================================================

#[test]
fn sort_outfile() {
    let input = "c\t3\na\t1\nb\t2\n";
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("sorted.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["sort", "-k", "1", "-o", outfile.to_str().unwrap()])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let contents = std::fs::read_to_string(&outfile).unwrap();
    assert_eq!(contents, "a\t1\nb\t2\nc\t3\n");
}

#[test]
fn sort_outfile_with_header() {
    let input = "name\tvalue\nc\t3\na\t1\n";
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("sorted.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header",
            "-k",
            "1",
            "-o",
            outfile.to_str().unwrap(),
        ])
        .stdin(input)
        .run();

    assert!(stdout.is_empty());

    let contents = std::fs::read_to_string(&outfile).unwrap();
    assert_eq!(contents, "name\tvalue\na\t1\nc\t3\n");
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn sort_lexicographic_file_names() {
    let input = "file2.txt\na\nfile10.txt\nfile1.txt\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "a\nfile1.txt\nfile10.txt\nfile2.txt\n");
}

#[test]
fn sort_unicode_values() {
    let input = "中\t2\nあ\t1\n🎉\t3\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert!(stdout.contains("あ"));
    assert!(stdout.contains("中"));
    assert!(stdout.contains("🎉"));
}

#[test]
fn sort_missing_fields_in_some_rows() {
    let input = "a\t2\nb\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "1"]).stdin(input).run();

    assert_eq!(stdout, "a\t2\nb\n");
}

#[test]
fn sort_identical_values_stable() {
    let input = "b\t2\na\t1\na\t1\n";

    let (stdout, _) = TvaCmd::new().args(&["sort", "-k", "2"]).stdin(input).run();

    assert_eq!(stdout, "a\t1\na\t1\nb\t2\n");
}

// ============================================================================
// Multi-File Tests
// ============================================================================

#[test]
fn sort_multiple_files_with_different_headers() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("file1.tsv");
    let file2 = dir.path().join("file2.tsv");

    std::fs::write(&file1, "# File1 header\nc\t1\na\t2\n").unwrap();
    std::fs::write(&file2, "# File2 header\nb\t3\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header-lines",
            "1",
            "-k",
            "1",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, "# File1 header\na\t2\nb\t3\nc\t1\n");
}

#[test]
fn sort_with_header_lines_n_multi_file() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("file1.tsv");
    let file2 = dir.path().join("file2.tsv");

    std::fs::write(&file1, "# Comment 1\nc\t1\na\t2\n").unwrap();
    std::fs::write(&file2, "# Comment 1\nb\t3\nd\t4\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "sort",
            "--header-lines",
            "1",
            "-k",
            "1",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, "# Comment 1\na\t2\nb\t3\nc\t1\nd\t4\n");
}
