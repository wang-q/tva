#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use test_case::test_case;

// ============================================================================
// Basic Reverse Tests
// ============================================================================

#[test_case("1\n2\n3\n", "3\n2\n1\n" ; "basic")]
#[test_case("", "" ; "empty")]
#[test_case("1", "1" ; "single_line_no_newline")]
#[test_case("1\n", "1\n" ; "single_line_with_newline")]
#[test_case("1\n2", "21\n" ; "two_lines_no_newline_at_end")]
fn reverse_basic(input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["reverse"]).stdin(input).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case("H\n1\n2\n3\n", "H\n3\n2\n1\n" ; "header_basic")]
#[test_case("", "" ; "header_empty")]
#[test_case("Header\n", "Header\n" ; "header_single_line")]
#[test_case("Header\nData\n", "Header\nData\n" ; "header_two_lines")]
#[test_case("Header\n1\n2\n", "Header\n2\n1\n" ; "header_three_lines")]
fn reverse_header(input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Multi-File Tests
// ============================================================================

fn run_multi_file_test(file_contents: &[&str], extra_args: &[&str], expected: &str) {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    fs::write(&file1_path, file_contents[0]).unwrap();
    fs::write(&file2_path, file_contents[1]).unwrap();

    let mut args = vec!["reverse"];
    args.extend_from_slice(extra_args);
    args.push(file1_path.to_str().unwrap());
    args.push(file2_path.to_str().unwrap());

    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

#[test_case(&["1\n2\n", "3\n4\n"], &[], "2\n1\n4\n3\n" ; "basic")]
#[test_case(&["Header\n1\n", "2\n3\n"], &["--header"], "Header\n1\n3\n2\n" ; "with_header")]
#[test_case(&["", "Header\n1\n"], &["--header"], "Header\n1\n" ; "empty_first_file")]
fn reverse_multi_file(file_contents: &[&str], extra_args: &[&str], expected: &str) {
    run_multi_file_test(file_contents, extra_args, expected);
}

#[test]
fn reverse_mmap_fallback() {
    // Tests lines 138-152 in reverse.rs: fallback when mmap fails
    // We force this using the hidden --no-mmap flag
    let input = "1\n2\n3\n";
    let expected = "3\n2\n1\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("fallback.tsv");
    fs::write(&file_path, input).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--no-mmap", file_path.to_str().unwrap()])
        .run();

    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Hash1 Mode Tests
// ============================================================================

#[test_case("# Comment 1\n# Comment 2\nCol1\tCol2\n1\t2\n3\t4\n", "# Comment 1\n# Comment 2\nCol1\tCol2\n3\t4\n1\t2\n" ; "hash1_basic")]
#[test_case("Col1\tCol2\n1\t2\n3\t4\n", "Col1\tCol2\n3\t4\n1\t2\n" ; "hash1_no_hash_lines")]
#[test_case("# Comment\nCol1\tCol2\n1\t2\n", "# Comment\nCol1\tCol2\n1\t2\n" ; "hash1_single_hash")]
#[test_case("# Comment\nCol1\tCol2\n", "# Comment\nCol1\tCol2\n" ; "hash1_only_header")]
#[test_case("# Comment", "# Comment" ; "hash1_hash_no_newline")]
#[test_case("# Comment\nCol1\tCol2", "# Comment\nCol1\tCol2" ; "hash1_column_names_no_newline")]
#[test_case("# Comment 1\n# Comment 2\n", "# Comment 2\n# Comment 1\n" ; "hash1_only_hash_lines")]
fn reverse_header_hash1(input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header-hash1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header_hash1_multi_file() {
    // Multi-file with --header-hash1: only first file's header is used
    let file1_content = "# File1 Comment\nCol1\tCol2\n1\t2\n";
    let file2_content = "3\t4\n5\t6\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    fs::write(&file1_path, file1_content).unwrap();
    fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "reverse",
            "--header-hash1",
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // file1: header printed, "1\t2\n" reversed -> "1\t2\n"
    // file2: header_printed=true, "3\t4\n5\t6\n" reversed -> "5\t6\n3\t4\n"
    let expected = "# File1 Comment\nCol1\tCol2\n1\t2\n5\t6\n3\t4\n";

    assert_eq!(stdout, expected);
}
