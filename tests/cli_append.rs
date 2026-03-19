#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use test_case::test_case;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "field1\tfield2\tfield3\nabc\tdef\tghi\nfield1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n"
    ; "two_files_3col"
)]
#[test_case(
    &["tests/data/append/input1x3.tsv", "tests/data/append/input1x4.tsv"],
    "field1\nrow 1\nrow 2\nfield1\nnext-empty\n\nlast-line\n"
    ; "two_files_1col"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input1x3.tsv", "tests/data/append/input3x5.tsv", "tests/data/append/input1x4.tsv"],
    "field1\tfield2\tfield3\nabc\tdef\tghi\nfield1\nrow 1\nrow 2\nfield1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\nfield1\nnext-empty\n\nlast-line\n"
    ; "four_files"
)]
#[test_case(
    &["tests/data/append/input3x5.tsv"],
    "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n"
    ; "single_file"
)]
fn append_basic_tests(files: &[&str], expected: &str) {
    let mut args = vec!["append"];
    args.extend_from_slice(files);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Mode Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "field1\tfield2\tfield3\nabc\tdef\tghi\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n"
    ; "two_files_3col"
)]
#[test_case(
    &["tests/data/append/input1x3.tsv", "tests/data/append/input1x4.tsv"],
    "field1\nrow 1\nrow 2\nnext-empty\n\nlast-line\n"
    ; "two_files_1col"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input1x3.tsv", "tests/data/append/input3x5.tsv", "tests/data/append/input1x4.tsv"],
    "field1\tfield2\tfield3\nabc\tdef\tghi\nrow 1\nrow 2\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\nnext-empty\n\nlast-line\n"
    ; "four_files"
)]
#[test_case(
    &["tests/data/append/input3x5.tsv"],
    "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n"
    ; "single_file"
)]
fn append_header_tests(files: &[&str], expected: &str) {
    let mut args = vec!["append", "--header"];
    args.extend_from_slice(files);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_header_hash1() {
    let expected = "field1\tfield2\tfield3\nabc\tdef\tghi\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header-hash1",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_multiple_files_mixed_headers() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("f1.tsv");
    let file2 = temp.path().join("f2.tsv");

    fs::write(&file1, "# Comment\nfield1\tfield2\tfield3\nabc\tdef\tghi\n").unwrap();
    fs::write(&file2, "field1\tfield2\tfield3\njkl\tmno\tpqr\n").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header-hash1",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ])
        .run();

    assert_eq!(
        stdout,
        "field1\tfield2\tfield3\nabc\tdef\tghi\njkl\tmno\tpqr\n"
    );
}

// ============================================================================
// Track Source Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "input3x2\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput3x5\tfield1\tfield2\tfield3\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "two_files_3col"
)]
#[test_case(
    &["tests/data/append/input1x3.tsv", "tests/data/append/input1x4.tsv"],
    "input1x3\tfield1\ninput1x3\trow 1\ninput1x3\trow 2\ninput1x4\tfield1\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "two_files_1col"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input1x3.tsv", "tests/data/append/input3x5.tsv", "tests/data/append/input1x4.tsv"],
    "input3x2\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput1x3\tfield1\ninput1x3\trow 1\ninput1x3\trow 2\ninput3x5\tfield1\tfield2\tfield3\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\ninput1x4\tfield1\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "four_files"
)]
#[test_case(
    &["tests/data/append/input3x5.tsv"],
    "input3x5\tfield1\tfield2\tfield3\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "single_file"
)]
fn append_track_source_tests(files: &[&str], expected: &str) {
    let mut args = vec!["append", "--track-source"];
    args.extend_from_slice(files);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source", "tests/data/append/input3x2.tsv"])
        .run();
    assert!(stdout.contains("input3x2\tfield1\tfield2\tfield3"));
}

// ============================================================================
// Header + Track Source Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "file\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "two_files_3col"
)]
#[test_case(
    &["tests/data/append/input1x3.tsv", "tests/data/append/input1x4.tsv"],
    "file\tfield1\ninput1x3\trow 1\ninput1x3\trow 2\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "two_files_1col"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input1x3.tsv", "tests/data/append/input3x5.tsv", "tests/data/append/input1x4.tsv"],
    "file\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput1x3\trow 1\ninput1x3\trow 2\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "four_files"
)]
#[test_case(
    &["tests/data/append/input3x5.tsv"],
    "file\tfield1\tfield2\tfield3\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "single_file"
)]
fn append_header_and_track_source_tests(files: &[&str], expected: &str) {
    let mut args = vec!["append", "--header", "--track-source"];
    args.extend_from_slice(files);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Source Header Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "source\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "two_files_3col"
)]
#[test_case(
    &["tests/data/append/input1x3.tsv", "tests/data/append/input1x4.tsv"],
    "source\tfield1\ninput1x3\trow 1\ninput1x3\trow 2\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "two_files_1col"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "tests/data/append/input1x3.tsv", "tests/data/append/input3x5.tsv", "tests/data/append/input1x4.tsv"],
    "source\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput1x3\trow 1\ninput1x3\trow 2\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\ninput1x4\tnext-empty\ninput1x4\t\ninput1x4\tlast-line\n"
    ; "four_files"
)]
#[test_case(
    &["tests/data/append/input3x5.tsv"],
    "source\tfield1\tfield2\tfield3\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "single_file"
)]
fn append_source_header_tests(files: &[&str], expected: &str) {
    let mut args = vec!["append", "--source-header", "source"];
    args.extend_from_slice(files);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "filename",
            "tests/data/append/input3x2.tsv",
        ])
        .run();
    assert!(stdout.starts_with("filename\tfield1\tfield2\tfield3\n"));
}

// ============================================================================
// Stdin Tests
// ============================================================================

#[test_case(
    "field1\tfield2\nval1\tval2\n",
    &[],
    "field1\tfield2\nval1\tval2\n"
    ; "basic_stdin"
)]
#[test_case(
    "a\tb\n1\t2\n",
    &[],
    "a\tb\n1\t2\n"
    ; "basic_short"
)]
#[test_case(
    "a\tb\n1\t2\n",
    &["--track-source"],
    "stdin\ta\tb\nstdin\t1\t2\n"
    ; "track_source"
)]
#[test_case(
    "a\tb\n1\t2\n",
    &["--source-header", "SRC"],
    "SRC\ta\tb\nstdin\t1\t2\n"
    ; "source_header"
)]
fn append_stdin_tests(stdin: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["append"];
    all_args.extend_from_slice(args);
    let (stdout, _) = TvaCmd::new().stdin(stdin).args(&all_args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_pipe() {
    let input = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    let expected = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    let (stdout, _) = TvaCmd::new().args(&["append"]).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_dash_arg_middle() {
    let stdin_input = "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let expected = "field1\tfield2\tfield3\nabc\tdef\tghi\nfield1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--", "tests/data/append/input3x2.tsv", "-"])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_dash_arg_middle_with_header() {
    let stdin_input = "field1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let expected = "field1\tfield2\tfield3\nabc\tdef\tghi\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "--", "tests/data/append/input3x2.tsv", "-"])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_stdin_explicit_file_mapping() {
    let stdin_input = "field1\tfield2\tfield3\nabc\tdef\tghi\n";
    let expected = "file\tfield1\tfield2\tfield3\nstandard-input\tabc\tdef\tghi\n3x5\tjkl\tmno\tpqr\n3x5\t123\t456\t789\n3x5\txy1\txy2\txy3\n3x5\tpqx\tpqy\tpqz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-f",
            "standard-input=-",
            "-f",
            "3x5=tests/data/append/input3x5.tsv",
        ])
        .stdin(stdin_input)
        .run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_file_label() {
    let (stdout, _) = TvaCmd::new()
        .stdin("a\tb\n1\t2\n")
        .args(&["append", "--file", "mysource=-"])
        .run();
    assert_eq!(stdout, "mysource\ta\tb\nmysource\t1\t2\n");
}

// ============================================================================
// File Mapping Tests
// ============================================================================

#[test]
fn append_file_mapping() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--file",
            "custom_label=tests/data/append/input3x2.tsv",
        ])
        .run();
    assert!(stdout.contains("custom_label\tfield1\tfield2\tfield3"));
}

#[test]
fn append_subdir_filename_label() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source", "tests/data/append/input3x2.tsv"])
        .run();
    assert!(stdout.contains("input3x2\tfield1"));
}

#[test]
fn append_custom_delimiter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "--delimiter",
            ":",
            "tests/data/append/input3x2.tsv",
        ])
        .run();
    assert!(stdout.contains("input3x2:field1\tfield2\tfield3"));
}

// ============================================================================
// Mixed Order Tests
// ============================================================================

#[test_case(
    &["tests/data/append/input1x3.tsv", "--file", "L=tests/data/append/input1x4.tsv"],
    "file\tfield1\ninput1x3\trow 1\ninput1x3\trow 2\nL\tnext-empty\nL\t\nL\tlast-line\n"
    ; "pos_flag"
)]
#[test_case(
    &["--file", "L=tests/data/append/input1x4.tsv", "tests/data/append/input1x3.tsv"],
    "file\tfield1\nL\tnext-empty\nL\t\nL\tlast-line\ninput1x3\trow 1\ninput1x3\trow 2\n"
    ; "flag_pos"
)]
#[test_case(
    &["tests/data/append/input3x2.tsv", "--file", "L=tests/data/append/input1x4.tsv", "tests/data/append/input3x5.tsv"],
    "file\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\nL\tnext-empty\nL\t\nL\tlast-line\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n"
    ; "pos_flag_pos"
)]
fn append_mixed_order_tests(args: &[&str], expected: &str) {
    let mut all_args = vec!["append", "-H"];
    all_args.extend_from_slice(args);
    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Unicode Tests
// ============================================================================

#[test]
fn append_unicode_header_and_source_labels() {
    let expected = "πηγή\tfield1\nκόκκινος\trow 1\nκόκκινος\trow 2\nάσπρο\tnext-empty\nάσπρο\t\nάσπρο\tlast-line\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "-s",
            "πηγή",
            "-f",
            "κόκκινος=tests/data/append/input1x3.tsv",
            "-f",
            "άσπρο=tests/data/append/input1x4.tsv",
        ])
        .run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Empty File Tests
// ============================================================================

#[test_case(&[] ; "no_header")]
#[test_case(&["-H"] ; "with_header")]
fn append_empty_file_tests(header_args: &[&str]) {
    let mut args = vec!["append"];
    args.extend_from_slice(header_args);
    args.push("tests/data/append/empty-file.txt");
    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert!(stdout.is_empty());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(
    &["--file", "invalid_format"],
    "invalid --file value `invalid_format`; expected LABEL=FILE"
    ; "invalid_file_mapping"
)]
#[test_case(
    &["--file", "nolabel"],
    "invalid --file value"
    ; "file_format_no_equals"
)]
#[test_case(
    &["--delimiter", "TooLong"],
    "delimiter must be a single byte"
    ; "delimiter_too_long"
)]
#[test_case(
    &["--delimiter", "tab"],
    "delimiter must be a single byte"
    ; "delimiter_word"
)]
fn append_error_tests(args: &[&str], expected_err: &str) {
    let mut all_args = vec!["append"];
    all_args.extend_from_slice(args);
    let (_, stderr) = TvaCmd::new().args(&all_args).run_fail();
    assert!(stderr.contains(expected_err));
}

// ============================================================================
// Tempfile Tests
// ============================================================================

#[test]
fn append_header_handling_tempfiles() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--header", path1, path2])
        .run();

    assert_eq!(stdout, "h1\th2\n1\t2\n3\t4\n");
}

#[test]
fn append_header_handling_with_source_tempfiles() {
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "h1\th2\n1\t2").unwrap();
    let path1 = file1.path().to_str().unwrap();
    let name1 = std::path::Path::new(path1)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "h1\th2\n3\t4").unwrap();
    let path2 = file2.path().to_str().unwrap();
    let name2 = std::path::Path::new(path2)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--source-header", "SRC", path1, path2])
        .run();

    let expected = format!("SRC\th1\th2\n{}\t1\t2\n{}\t3\t4\n", name1, name2);
    assert_eq!(stdout, expected);
}

// ============================================================================
// Line Buffered Tests
// ============================================================================

#[test_case(
    "a\tb\n1\t2\n",
    &[],
    "a\tb\n1\t2\n"
    ; "basic"
)]
#[test_case(
    "",
    &["tests/data/append/input3x2.tsv", "tests/data/append/input3x5.tsv"],
    "field1\tfield2\tfield3\nabc\tdef\tghi\nfield1\tfield2\tfield3\njkl\tmno\tpqr\n123\t456\t789\nxy1\txy2\txy3\npqx\tpqy\tpqz\n"
    ; "multiple_files"
)]
fn append_line_buffered_tests(stdin: &str, file_args: &[&str], expected: &str) {
    let mut args = vec!["append", "--line-buffered"];
    args.extend_from_slice(file_args);
    let (stdout, _) = TvaCmd::new().stdin(stdin).args(&args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn append_line_buffered_with_source() {
    let expected = "source\tfield1\tfield2\tfield3\ninput3x2\tabc\tdef\tghi\ninput3x5\tjkl\tmno\tpqr\ninput3x5\t123\t456\t789\ninput3x5\txy1\txy2\txy3\ninput3x5\tpqx\tpqy\tpqz\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--line-buffered",
            "--source-header",
            "source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();
    assert_eq!(stdout, expected);
}
