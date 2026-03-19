#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;
use test_case::test_case;

// --- Test Data Constants ---

const INPUT1: &str = "The first line - Is it a header?\nabc\tdef\tghi\nsome random text\nJapanese: 私はガラスを食べられます。それは私を傷つけません。\n\nPrevious line blank\n\t\nPrevious line a single tab\n";

const INPUT2: &str = "The first line\nThe second line\nThe third line\n";

const ONE_LINE: &str = "The one line\n";

const EMPTY: &str = "";

// --- Helper Functions ---

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

// ============================================================================
// Basic Tests
// ============================================================================

#[test]
fn nl_basic() {
    let file = create_file(INPUT1);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", file.path().to_str().unwrap()])
        .run();

    let expected = "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Start Number Tests
// ============================================================================

#[test_case("-5", "-5\t" ; "negative_start")]
#[test_case("0", "0\t" ; "zero_start")]
#[test_case("10", "10\t" ; "positive_start")]
fn nl_start_number(start: &str, expected_prefix: &str) {
    let file = create_file(INPUT1);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-n", start, file.path().to_str().unwrap()])
        .run();

    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.starts_with(expected_prefix));
}

#[test_case("10", "10\tThe first line - Is it a header?\n11\tabc\tdef\tghi\n12\tsome random text\n13\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n14\t\n15\tPrevious line blank\n16\t\t\n17\tPrevious line a single tab\n" ; "start_at_10")]
#[test_case("-10", "-10\tThe first line - Is it a header?\n-9\tabc\tdef\tghi\n-8\tsome random text\n-7\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n-6\t\n-5\tPrevious line blank\n-4\t\t\n-3\tPrevious line a single tab\n" ; "start_at_negative_10")]
fn nl_start_number_full(start: &str, expected: &str) {
    let file = create_file(INPUT1);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-n", start, file.path().to_str().unwrap()])
        .run();

    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Tests
// ============================================================================

#[test_case(&["--header"], "line\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n" ; "default_header")]
#[test_case(&["--header-string", "LINENUM"], "LINENUM\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n" ; "header_string_LINENUM")]
#[test_case(&["-s", "LineNum_àßß"], "LineNum_àßß\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n" ; "header_string_unicode")]
#[test_case(&["--header", "-s", "line_num"], "line_num\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n" ; "header_and_custom_string")]
fn nl_header_options(args: &[&str], expected: &str) {
    let file = create_file(INPUT1);
    let mut all_args = vec!["nl"];
    all_args.extend_from_slice(args);
    all_args.push(file.path().to_str().unwrap());

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_string_implies_header() {
    let file = create_file(INPUT1);
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header-string",
            "LINENUM",
            file.path().to_str().unwrap(),
        ])
        .run();

    let (expected_stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "--header-string",
            "LINENUM",
            file.path().to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, expected_stdout);
}

#[test]
fn nl_header_string_short_implies_header() {
    let file = create_file(INPUT1);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-s", "LINENUM", file.path().to_str().unwrap()])
        .run();

    let (expected_stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "-s",
            "LINENUM",
            file.path().to_str().unwrap(),
        ])
        .run();

    assert_eq!(stdout, expected_stdout);
}

// ============================================================================
// Delimiter Tests
// ============================================================================

#[test_case(":" ; "colon")]
#[test_case("|" ; "pipe")]
#[test_case("#" ; "hash")]
#[test_case("@" ; "at")]
#[test_case("~" ; "tilde")]
fn nl_delimiter_special_chars(delim: &str) {
    let file = create_file(ONE_LINE);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-d", delim, file.path().to_str().unwrap()])
        .run();

    assert!(
        stdout.contains(delim),
        "Delimiter {} not found in output",
        delim
    );
}

#[test_case(&["--delimiter", ":"], "1:The first line - Is it a header?\n2:abc\tdef\tghi\n3:some random text\n4:Japanese: 私はガラスを食べられます。それは私を傷つけません。\n5:\n6:Previous line blank\n7:\t\n8:Previous line a single tab\n" ; "colon_delimiter")]
#[test_case(&["-d", "_"], "1_The first line - Is it a header?\n2_abc\tdef\tghi\n3_some random text\n4_Japanese: 私はガラスを食べられます。それは私を傷つけません。\n5_\n6_Previous line blank\n7_\t\n8_Previous line a single tab\n" ; "underscore_delimiter")]
#[test_case(&["--header", "-d", "^"], "line^The first line - Is it a header?\n1^abc\tdef\tghi\n2^some random text\n3^Japanese: 私はガラスを食べられます。それは私を傷つけません。\n4^\n5^Previous line blank\n6^\t\n7^Previous line a single tab\n" ; "header_with_caret")]
fn nl_delimiter_full(args: &[&str], expected: &str) {
    let file = create_file(INPUT1);
    let mut all_args = vec!["nl"];
    all_args.extend_from_slice(args);
    all_args.push(file.path().to_str().unwrap());

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

// ============================================================================
// Empty File Tests
// ============================================================================

#[test_case(&[], "" ; "no_options")]
#[test_case(&["-H"], "" ; "with_header")]
fn nl_empty_file(args: &[&str], expected: &str) {
    let file = create_file(EMPTY);
    let mut all_args = vec!["nl"];
    all_args.extend_from_slice(args);
    all_args.push(file.path().to_str().unwrap());

    let (stdout, _) = TvaCmd::new().args(&all_args).run();
    assert_eq!(stdout, expected);
}

#[test]
fn nl_empty_file_no_line_number_consumed() {
    let file1 = create_file(INPUT1);
    let file_empty = create_file(EMPTY);
    let file_one = create_file(ONE_LINE);

    let (single_file_out, _) = TvaCmd::new()
        .args(&["nl", file1.path().to_str().unwrap()])
        .run();
    let single_file_lines = single_file_out.lines().count();

    let (multi_file_out, _) = TvaCmd::new()
        .args(&[
            "nl",
            file1.path().to_str().unwrap(),
            file_empty.path().to_str().unwrap(),
            file_one.path().to_str().unwrap(),
        ])
        .run();
    let multi_file_lines = multi_file_out.lines().count();

    assert_eq!(single_file_lines + 1, multi_file_lines);
}

// ============================================================================
// Multi-File Tests
// ============================================================================

fn run_multi_file_test(file_contents: &[&str], extra_args: &[&str], expected: &str) {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut file_paths = Vec::new();

    for (i, content) in file_contents.iter().enumerate() {
        let path = temp_dir.path().join(format!("file{}.txt", i + 1));
        write!(&mut std::fs::File::create(&path).unwrap(), "{}", content).unwrap();
        file_paths.push(path);
    }

    let mut args = vec!["nl"];
    args.extend_from_slice(extra_args);
    for path in &file_paths {
        args.push(path.to_str().unwrap());
    }

    let (stdout, _) = TvaCmd::new().args(&args).run();
    assert_eq!(stdout, expected);
}

#[test_case(
    &[INPUT1, INPUT2, EMPTY, ONE_LINE],
    &[],
    "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n9\tThe first line\n10\tThe second line\n11\tThe third line\n12\tThe one line\n"
    ; "basic_multi_file"
)]
#[test_case(
    &[INPUT1, ONE_LINE, INPUT2, EMPTY],
    &[],
    "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n9\tThe one line\n10\tThe first line\n11\tThe second line\n12\tThe third line\n"
    ; "reordered_files"
)]
#[test_case(
    &[EMPTY, INPUT1, ONE_LINE, INPUT2, INPUT1],
    &[],
    "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n9\tThe one line\n10\tThe first line\n11\tThe second line\n12\tThe third line\n13\tThe first line - Is it a header?\n14\tabc\tdef\tghi\n15\tsome random text\n16\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n17\t\n18\tPrevious line blank\n19\t\t\n20\tPrevious line a single tab\n"
    ; "leading_empty_file"
)]
#[test_case(
    &[INPUT2, INPUT2, INPUT2],
    &["-H"],
    "line\tThe first line\n1\tThe second line\n2\tThe third line\n3\tThe second line\n4\tThe third line\n5\tThe second line\n6\tThe third line\n"
    ; "header_second_file"
)]
#[test_case(
    &[INPUT1, INPUT2, EMPTY, ONE_LINE],
    &["--header"],
    "line\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n8\tThe second line\n9\tThe third line\n"
    ; "header_mixed_files"
)]
fn nl_multi_file(file_contents: &[&str], extra_args: &[&str], expected: &str) {
    run_multi_file_test(file_contents, extra_args, expected);
}

#[test]
fn nl_multi_file_continuous_numbering() {
    let file_one = create_file(ONE_LINE);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            file_one.path().to_str().unwrap(),
            file_one.path().to_str().unwrap(),
            file_one.path().to_str().unwrap(),
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("1\t"));
    assert!(lines[1].starts_with("2\t"));
    assert!(lines[2].starts_with("3\t"));
}

// ============================================================================
// Stdin Tests
// ============================================================================

#[test_case(INPUT1, &[], "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n" ; "stdin_basic")]
#[test_case(INPUT1, &["-"], "1\tThe first line - Is it a header?\n2\tabc\tdef\tghi\n3\tsome random text\n4\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n5\t\n6\tPrevious line blank\n7\t\t\n8\tPrevious line a single tab\n" ; "stdin_dash_alias")]
fn nl_stdin(input: &str, args: &[&str], expected: &str) {
    let mut all_args = vec!["nl"];
    all_args.extend_from_slice(args);

    let (stdout, _) = TvaCmd::new().args(&all_args).stdin(input).run();
    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_multi_file_header() {
    let input = format!("{}{}", INPUT1, INPUT2);
    let (stdout, _) = TvaCmd::new().args(&["nl", "--header"]).stdin(input).run();

    let expected = "line\tThe first line - Is it a header?\n1\tabc\tdef\tghi\n2\tsome random text\n3\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n4\t\n5\tPrevious line blank\n6\t\t\n7\tPrevious line a single tab\n8\tThe first line\n9\tThe second line\n10\tThe third line\n";
    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_with_args() {
    let file2 = create_file(INPUT2);
    let (stdout, _) = TvaCmd::new()
        .args(&["nl", file2.path().to_str().unwrap(), "-"])
        .stdin(INPUT1)
        .run();

    let expected = "1\tThe first line\n2\tThe second line\n3\tThe third line\n4\tThe first line - Is it a header?\n5\tabc\tdef\tghi\n6\tsome random text\n7\tJapanese: 私はガラスを食べられます。それは私を傷つけません。\n8\t\n9\tPrevious line blank\n10\t\t\n11\tPrevious line a single tab\n";
    assert_eq!(stdout, expected);
}

#[test]
fn nl_only_newlines() {
    let input = "\n\n\n";
    let (stdout, _) = TvaCmd::new().args(&["nl"]).stdin(input).run();

    let expected = "1\t\n2\t\n3\t\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Help and Version Tests
// ============================================================================

#[test]
fn nl_help_displays_usage() {
    let (stdout, _) = TvaCmd::new().args(&["nl", "--help"]).run();
    assert!(stdout.contains("Reads TSV data from files or standard input"));
}

#[test]
fn nl_version_matches_tva() {
    let (tva_version, _) = TvaCmd::new().args(&["--version"]).run();
    let tva_version_num = tva_version.split_whitespace().last().unwrap().to_string();

    let (nl_version, _) = TvaCmd::new().args(&["nl", "--version"]).run();
    let nl_version_num = nl_version.split_whitespace().last().unwrap().to_string();

    assert_eq!(nl_version_num, tva_version_num);
}

#[test]
fn nl_line_buffered_help_text() {
    let (stdout, _) = TvaCmd::new().args(&["nl", "--help"]).run();
    assert!(stdout.contains("Force line-buffered output mode"));
    assert!(stdout.contains("real-time viewing"));
}

#[test]
fn nl_line_buffered_matches_default() {
    let file = create_file(INPUT1);
    let (default_out, _) = TvaCmd::new()
        .args(&["nl", file.path().to_str().unwrap()])
        .run();

    let (buffered_out, _) = TvaCmd::new()
        .args(&["nl", "--line-buffered", file.path().to_str().unwrap()])
        .run();

    assert_eq!(default_out, buffered_out);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test_case(&["nl", "/nonexistent/file.txt"], "could not open" ; "nosuchfile")]
#[test_case(&["nl", "--nosuchparam", "/tmp/test.txt"], "--nosuchparam" ; "unknown_option")]
fn nl_errors(args: &[&str], expected_err: &str) {
    let (_, stderr) = TvaCmd::new().args(args).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected '{}' in stderr, got: {}",
        expected_err,
        stderr
    );
}
