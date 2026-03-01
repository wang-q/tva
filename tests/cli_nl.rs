#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;

fn expected_block(command: &str) -> String {
    let gold = fs::read_to_string("tests/data/nl/gold_basic_tests_1.txt").unwrap();

    let header = format!("====[number-lines {}]====", command);
    let mut lines = gold.lines();

    while let Some(line) = lines.next() {
        if line == header {
            let mut block = Vec::new();
            for line in lines.by_ref() {
                if line.is_empty() {
                    break;
                }
                block.push(line);
            }
            if block.is_empty() {
                return String::new();
            }
            return block.join("\n") + "\n";
        }
    }

    panic!("Expected block not found for command: {}", command);
}

fn expected_stdin_block(header: &str) -> String {
    let gold = fs::read_to_string("tests/data/nl/gold_basic_tests_1.txt").unwrap();

    let mut lines = gold.lines();
    while let Some(line) = lines.next() {
        if line == header {
            let mut block = Vec::new();
            for line in lines.by_ref() {
                if line.is_empty() {
                    break;
                }
                block.push(line);
            }
            return block.join("\n") + "\n";
        }
    }

    panic!("Expected stdin block not found for header: {}", header);
}

#[test]
fn nl_basic_from_gold() {
    let expected = expected_block("input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_start_number_from_gold() {
    let expected = expected_block("--start-number 10 input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "--start-number", "10", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_start_number_short_from_gold() {
    let expected = expected_block("-n 10 input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-n", "10", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_start_number_negative_from_gold() {
    let expected = expected_block("-n -10 input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "--start-number", "-10", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_empty_file_from_gold() {
    let expected = expected_block("empty-file.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "tests/data/nl/empty-file.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_empty_file_from_gold() {
    let expected = expected_block("-H empty-file.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-H", "tests/data/nl/empty-file.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_from_gold() {
    let expected = expected_block("--header input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "--header", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_string_unicode_from_gold() {
    let expected = expected_block("-s LineNum_àßß input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-s", "LineNum_àßß", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_and_header_string_from_gold() {
    let expected = expected_block("--header -s line_num input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "-s",
            "line_num",
            "tests/data/nl/input1.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_delimiter_colon_from_gold() {
    let expected = expected_block("--delimiter : input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "--delimiter", ":", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_delimiter_underscore_from_gold() {
    let expected = expected_block("-d _ input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "-d", "_", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_and_delimiter_from_gold() {
    let expected = expected_block("--header -d ^ input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "--header", "-d", "^", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_header_string_from_gold() {
    let expected = expected_block("--header-string LINENUM input1.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header-string",
            "LINENUM",
            "tests/data/nl/input1.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_from_gold() {
    let expected =
        expected_block("input1.txt input2.txt empty-file.txt one-line-file.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "tests/data/nl/input1.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/empty-file.txt",
            "tests/data/nl/one-line-file.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_reordered_from_gold() {
    let expected =
        expected_block("input1.txt one-line-file.txt input2.txt empty-file.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "tests/data/nl/input1.txt",
            "tests/data/nl/one-line-file.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/empty-file.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_with_leading_empty_from_gold() {
    let expected = expected_block(
        "empty-file.txt input1.txt one-line-file.txt input2.txt input1.txt",
    );

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "tests/data/nl/empty-file.txt",
            "tests/data/nl/input1.txt",
            "tests/data/nl/one-line-file.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/input1.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_header_second_from_gold() {
    let expected = expected_block("-H input2.txt input2.txt input2.txt");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "-H",
            "tests/data/nl/input2.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/input2.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_header_mixed_from_gold() {
    let expected = expected_block(
        "--header input1.txt input2.txt empty-file.txt one-line-file.txt",
    );

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "tests/data/nl/input1.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/empty-file.txt",
            "tests/data/nl/one-line-file.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_header_string_from_gold() {
    let expected = expected_block(
        "--header -s LINENUM empty-file.txt input1.txt one-line-file.txt input2.txt input1.txt",
    );

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "-s",
            "LINENUM",
            "tests/data/nl/empty-file.txt",
            "tests/data/nl/input1.txt",
            "tests/data/nl/one-line-file.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/input1.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_multi_file_header_start_number_from_gold() {
    let expected = expected_block(
        "--header -n 10 input1.txt one-line-file.txt input2.txt empty-file.txt",
    );

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "-n",
            "10",
            "tests/data/nl/input1.txt",
            "tests/data/nl/one-line-file.txt",
            "tests/data/nl/input2.txt",
            "tests/data/nl/empty-file.txt",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_from_gold() {
    let expected = expected_stdin_block("====[cat input1.txt | number-lines]====");

    let input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();

    let (stdout, _) = TvaCmd::new().args(&["nl"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_multi_file_header_from_gold() {
    let expected = expected_stdin_block(
        "====[cat input1.txt input2.txt | number-lines --header]====",
    );

    let input1 = fs::read_to_string("tests/data/nl/input1.txt").unwrap();
    let input2 = fs::read_to_string("tests/data/nl/input2.txt").unwrap();
    let input = format!("{input1}{input2}");

    let (stdout, _) = TvaCmd::new().args(&["nl", "--header"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_with_args_from_gold() {
    let expected =
        expected_stdin_block("====[cat input1.txt | number-lines -- input2.txt -]====");

    let stdin_input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["nl", "tests/data/nl/input2.txt", "stdin"])
        .stdin(stdin_input)
        .run();

    assert_eq!(stdout, expected);
}

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
fn nl_error_nosuchfile() {
    let (_, stderr) = TvaCmd::new()
        .args(&["nl", "tests/data/nl/nosuchfile.txt"])
        .run_fail();

    assert!(stderr.contains("could not open"));
}

#[test]
fn nl_error_unknown_option() {
    let (_, stderr) = TvaCmd::new()
        .args(&["nl", "--nosuchparam", "tests/data/nl/input1.txt"])
        .run_fail();

    assert!(stderr.contains("--nosuchparam"));
}

#[test]
fn nl_stdin_dash_alias() {
    let expected = expected_stdin_block("====[cat input1.txt | number-lines]====");

    let input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();

    let (stdout, _) = TvaCmd::new().args(&["nl", "-"]).stdin(input).run();

    assert_eq!(stdout, expected);
}

#[test]
fn nl_line_buffered_matches_default() {
    let (default_out, _) = TvaCmd::new()
        .args(&["nl", "tests/data/nl/input1.txt"])
        .run();

    let (buffered_out, _) = TvaCmd::new()
        .args(&["nl", "--line-buffered", "tests/data/nl/input1.txt"])
        .run();

    assert_eq!(default_out, buffered_out);
}

#[test]
fn nl_stdin_mixed_dash_complex() {
    // Equivalent to: cat input1.txt | nl -- input2.txt - one-line-file.txt
    // input2 has 3 lines.
    // input1 (stdin) has 8 lines.
    // one-line-file has 1 line.
    // Total lines: 3 + 8 + 1 = 12.
    // Ordering: input2 (1-3), stdin (4-11), one-line (12).

    let stdin_input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();
    let input2 = fs::read_to_string("tests/data/nl/input2.txt").unwrap();
    let one_line = fs::read_to_string("tests/data/nl/one-line-file.txt").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "tests/data/nl/input2.txt",
            "-",
            "tests/data/nl/one-line-file.txt",
        ])
        .stdin(stdin_input.clone())
        .run();

    // Construct expected output manually
    let mut expected = String::new();
    let mut line_num = 1;

    // Process input2
    for line in input2.lines() {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    // Process stdin (input1)
    for line in stdin_input.lines() {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    // Process one-line-file
    for line in one_line.lines() {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    assert_eq!(stdout, expected);
}

#[test]
fn nl_stdin_mixed_dash_complex_header() {
    // Equivalent to: cat input1.txt | nl --header -- input2.txt - one-line-file.txt
    // All files have headers or are treated as having headers.
    // input2: header + 2 lines.
    // input1 (stdin): header + 7 lines.
    // one-line-file: header (1 line only).

    let stdin_input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();
    let input2 = fs::read_to_string("tests/data/nl/input2.txt").unwrap();
    let one_line = fs::read_to_string("tests/data/nl/one-line-file.txt").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "nl",
            "--header",
            "tests/data/nl/input2.txt",
            "-",
            "tests/data/nl/one-line-file.txt",
        ])
        .stdin(stdin_input.clone())
        .run();

    // Construct expected output manually
    let mut expected = String::new();
    // Header comes from the first file (input2)
    let mut input2_lines = input2.lines();
    if let Some(header) = input2_lines.next() {
        expected.push_str(&format!("line\t{}\n", header));
    }

    let mut line_num = 1;

    // Rest of input2
    for line in input2_lines {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    // Process stdin (input1) - skip header
    let mut stdin_lines = stdin_input.lines();
    let _ = stdin_lines.next(); // Skip header
    for line in stdin_lines {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    // Process one-line-file - skip header (it only has one line which is header)
    let mut one_line_lines = one_line.lines();
    let _ = one_line_lines.next(); // Skip header
    for line in one_line_lines {
        expected.push_str(&format!("{}\t{}\n", line_num, line));
        line_num += 1;
    }

    assert_eq!(stdout, expected);
}
