#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::fs;

fn expected_block(command: &str) -> String {
    let gold = fs::read_to_string("tests/data/uniq/gold_basic_tests_1.txt").unwrap();

    let header = format!("====[tsv-uniq {}]====", command);
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

#[test]
fn uniq_basic_files() {
    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/genome/ctg.tsv", "tests/genome/ctg.tsv"])
        .run();

    assert_eq!(stdout.lines().count(), 4);

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/genome/ctg.tsv", "-f", "2"])
        .run();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("ctg:I:2\tI"));
}

#[test]
fn uniq_basic_stdin() {
    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();
    let input_dup = format!("{input}{input}");

    let (stdout, _) = TvaCmd::new().args(&["uniq"]).stdin(input_dup).run();

    assert_eq!(stdout.lines().count(), 4);

    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();

    let (stdout, _) = TvaCmd::new().args(&["uniq", "-f", "2"]).stdin(input).run();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("ctg:I:2\tI"));
}

#[test]
fn uniq_mixed_input() {
    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "stdin", "tests/genome/ctg.tsv"])
        .stdin(input)
        .run();

    assert_eq!(stdout.lines().count(), 4);
}

#[test]
fn uniq_header_merge() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "uniq",
            "--header",
            "tests/data/uniq/input1.tsv",
            "tests/data/uniq/input2.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty());
    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5");

    let header_count = lines
        .iter()
        .filter(|line| line.starts_with("f1\tf2\tf3\tf4\tf5"))
        .count();
    assert_eq!(header_count, 1);
}

#[test]
fn uniq_header_named_fields_single() {
    let input = "tests/data/uniq/input1.tsv";

    let (stdout_numeric, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "3,4", input])
        .run();

    let (stdout_named, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "f3,f4", input])
        .run();

    assert_eq!(stdout_numeric, stdout_named);
}

#[test]
fn uniq_gold_basic() {
    let expected = expected_block("input1.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/data/uniq/input1.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_header() {
    let expected = expected_block("--header input1.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--header", "tests/data/uniq/input1.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_field0() {
    let expected = expected_block("-f 0 input1.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "-f", "0", "tests/data/uniq/input1.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_header_field0() {
    let expected = expected_block("--header -f 0 input1.tsv");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "0", "tests/data/uniq/input1.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_fields1() {
    let expected = expected_block("input1.tsv --fields 1");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/data/uniq/input1.tsv", "--fields", "1"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_header_field2() {
    let expected = expected_block("input1.tsv --header -f 2");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/data/uniq/input1.tsv", "--header", "-f", "2"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_noheader_field2() {
    let expected = expected_block("input1_noheader.tsv -f 2");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/data/uniq/input1_noheader.tsv", "-f", "2"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_header_fields3_4() {
    let expected = expected_block("input1.tsv --header -f 3,4");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "uniq",
            "tests/data/uniq/input1.tsv",
            "--header",
            "-f",
            "3,4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_header_named_fields() {
    let expected = expected_block("input1.tsv --header -f f3,f4");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "uniq",
            "tests/data/uniq/input1.tsv",
            "--header",
            "-f",
            "f3,f4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_gold_noheader_fields3_4() {
    let expected = expected_block("input1_noheader.tsv -f 3,4");

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "tests/data/uniq/input1_noheader.tsv", "-f", "3,4"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn uniq_header_named_fields_multiple() {
    let input1 = "tests/data/uniq/input1.tsv";
    let input2 = "tests/data/uniq/input2.tsv";

    let (stdout_numeric, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "3,4", input1, input2])
        .run();

    let (stdout_named, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "f3,f4", input1, input2])
        .run();

    assert_eq!(stdout_numeric, stdout_named);
}

#[test]
fn uniq_ignore_case() {
    let input = "key\n\
                 a\n\
                 A\n\
                 a\n";

    let (stdout, _) = TvaCmd::new().args(&["uniq"]).stdin(input).run();
    assert_eq!(stdout.lines().count(), 3);

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--ignore-case"])
        .stdin(input)
        .run();
    assert_eq!(stdout.lines().count(), 2);
}

#[test]
fn uniq_repeated_min_max() {
    let input = "a\n\
                 a\n\
                 a\n\
                 b\n\
                 b\n\
                 c\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--repeated"])
        .stdin(input)
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a", "b"]);

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--at-least", "3"])
        .stdin(input)
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a"]);

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--at-least", "2", "--max", "3"])
        .stdin(input)
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a", "a", "b"]);
}

#[test]
fn uniq_equiv_and_number() {
    let input = "f1\tf2\n\
                 a\tX\n\
                 b\tY\n\
                 a\tZ\n\
                 a\tX\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "1", "--equiv", "--number"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "f1\tf2\tequiv_id\tequiv_line");
    assert_eq!(lines[1], "a\tX\t1\t1");
    assert_eq!(lines[2], "b\tY\t2\t1");
    assert_eq!(lines[3], "a\tZ\t1\t2");
    assert_eq!(lines[4], "a\tX\t1\t3");
}

#[test]
fn uniq_custom_delimiter() {
    let input = "k1_k2\n\
                 a_X\n\
                 a_Y\n\
                 b_Z\n\
                 a_X\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "uniq",
            "--delimiter",
            "_",
            "--header",
            "-f",
            "1",
            "--line-buffered",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "k1_k2");
    assert_eq!(lines[1], "a_X");
    assert_eq!(lines[2], "b_Z");
}

#[test]
fn uniq_error_equiv_start_requires_equiv() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "uniq",
            "-f",
            "2",
            "--equiv-start",
            "10",
            "tests/data/uniq/input1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("tva uniq: --equiv-start requires --equiv"));
}

#[test]
fn uniq_error_equiv_header_requires_equiv() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "uniq",
            "-f",
            "2",
            "--equiv-header",
            "id",
            "tests/data/uniq/input1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("tva uniq: --equiv-header requires --equiv"));
}

#[test]
fn uniq_error_number_header_requires_number() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "uniq",
            "-f",
            "2",
            "--number-header",
            "line",
            "tests/data/uniq/input1.tsv",
        ])
        .run_fail();

    assert!(stderr.contains("tva uniq: --number-header requires --number"));
}

#[test]
fn uniq_error_zero_in_field_range() {
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "-f", "0-2", "tests/data/uniq/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn uniq_error_field_name_requires_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "-f", "f1", "tests/data/uniq/input1.tsv"])
        .run_fail();

    assert!(stderr.contains("requires header"));
}

#[test]
fn uniq_missing_fields_strict() {
    let input = "col1\nA\nB\n";
    // Field 2 is missing.

    // We expect FAILURE because tsv-uniq is strict about field existence.
    // If tva uniq is not strict, this test will fail (because run_fail panics).
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "-f", "2"])
        .stdin(input)
        .run_fail();
    assert!(stderr.contains("Not enough fields"));
}

#[test]
fn uniq_error_delimiter_length() {
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "--delimiter", "TAB"])
        .run_fail();
    assert!(stderr.contains("delimiter must be a single character"));
}

#[test]
fn uniq_equiv_start_negative() {
    // L186: if equiv_start < 0 { 1 } else { equiv_start as u64 }
    let input = "a\nb\na\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--equiv", "--equiv-start=-5"])
        .stdin(input)
        .run();

    // Should start from 1
    // Output should be:
    // a\t1
    // b\t2
    // a\t1 (since key "a" maps to ID 1)

    assert!(stdout.contains("a\t1"));
    assert!(stdout.contains("b\t2"));

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].ends_with("\t1"));
    assert!(lines[1].ends_with("\t2"));
    assert!(lines[2].ends_with("\t1"));
}

#[test]
fn uniq_empty_input_with_header() {
    // L227: continue; // Empty file
    // Create an empty file
    let temp = tempfile::Builder::new().tempfile().unwrap();
    let path = temp.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new().args(&["uniq", "--header", path]).run();
    assert!(stdout.is_empty());
}

#[test]
fn uniq_header_field_spec_0() {
    // L232: if spec.trim() == "0"
    let input = "h1\th2\nv1\tv2\nv1\tv2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "0"])
        .stdin(input)
        .run();
    // Should output header + unique lines (whole line key)
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "h1\th2");
    assert_eq!(lines[1], "v1\tv2");
}

#[test]
fn uniq_noheader_field_spec_0() {
    // L276: if spec.trim() == "0"
    let input = "v1\tv2\nv1\tv2\n";
    let (stdout, _) = TvaCmd::new().args(&["uniq", "-f", "0"]).stdin(input).run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "v1\tv2");
}

#[test]
fn uniq_field_parse_error_in_header_block() {
    // L244: Err(e) => arg_error(&e)
    let input = "h1\th2\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "--header", "-f", "invalid"])
        .stdin(input)
        .run_fail();

    assert!(
        stderr.contains("Field")
            || stderr.contains("not found")
            || stderr.contains("Invalid")
            || stderr.contains("does not exist")
            || stderr.contains("unknown field")
    );
}

#[test]
fn uniq_field_parse_error_in_noheader_block() {
    // L288: Err(e) => arg_error(&e)
    let input = "v1\tv2\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["uniq", "-f", "invalid"])
        .stdin(input)
        .run_fail();
    // Without header, named fields fail with "requires header" or similar from parse_field_list_with_header(..., None)
    assert!(stderr.contains("requires header"));
}

#[test]
fn uniq_repeated_logic_at_least_1() {
    // L149: if repeated && at_least <= 1 { at_least = 2; }
    let input = "a\na\nb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--repeated", "--at-least", "1"])
        .stdin(input)
        .run();
    // repeated implies at_least=2 if not specified or <=1
    // So "b" (count 1) should be excluded. "a" (count 2) included.
    assert_eq!(stdout.trim(), "a");
}

#[test]
fn uniq_max_logic() {
    // L152: logic for max = at_least
    // Case: at_least=2, max=1 (max < at_least) -> max becomes at_least (2)
    let input = "a\na\na\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["uniq", "--at-least", "2", "--max", "1"])
        .stdin(input)
        .run();
    // max should be adjusted to 2.
    // 1st a: count=1. Not output (at_least=2).
    // 2nd a: count=2. Output (2 <= 2 && 2 >= 2).
    // 3rd a: count=3. Not output (3 > 2).
    // So output line count is 1.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn uniq_equiv_start_negative_warning() {
    // Test that negative equiv-start produces a warning and uses 1
    let input = "a\nb\na\n";
    let (stdout, stderr) = TvaCmd::new()
        .args(&["uniq", "--equiv", "--equiv-start=-5"])
        .stdin(input)
        .run();

    // Check warning message is printed to stderr
    assert!(
        stderr.contains("warning: --equiv-start value -5 is negative, using 1 instead"),
        "Expected warning for negative equiv-start, got stderr: {}",
        stderr
    );

    // Verify output starts from 1 (not -5)
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    // First occurrence of "a" gets ID 1
    assert!(
        lines[0].ends_with("\t1"),
        "First line should end with tab 1"
    );
    // "b" gets ID 2
    assert!(
        lines[1].ends_with("\t2"),
        "Second line should end with tab 2"
    );
    // Second occurrence of "a" reuses ID 1
    assert!(
        lines[2].ends_with("\t1"),
        "Third line should end with tab 1"
    );
}

#[test]
fn uniq_equiv_start_zero_ok() {
    // Test that equiv-start=0 is treated as 0 (not negative), but since 0 is not useful,
    // the code will use it as u64 value 0, which will be incremented to 1 for the first ID
    let input = "a\nb\n";
    let (stdout, stderr) = TvaCmd::new()
        .args(&["uniq", "--equiv", "--equiv-start=0"])
        .stdin(input)
        .run();

    // 0 is not negative, so no warning should be printed
    assert!(
        !stderr.contains("warning"),
        "Should not have warning for zero equiv-start (0 is not negative), got stderr: {}",
        stderr
    );

    // equiv_start=0 means next_equiv_id starts at 0, but the first entry will use 0 and increment
    // Actually looking at the code: equiv_start as u64 = 0, so next_equiv_id = 0
    // But then entry.equiv_id = next_equiv_id (0), then next_equiv_id += 1
    // So first ID should be 0
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].ends_with("\t0"),
        "First line should end with tab 0"
    );
    assert!(
        lines[1].ends_with("\t1"),
        "Second line should end with tab 1"
    );
}

#[test]
fn uniq_equiv_start_positive_ok() {
    // Test that positive equiv-start works without warning
    let input = "a\nb\na\n";
    let (stdout, stderr) = TvaCmd::new()
        .args(&["uniq", "--equiv", "--equiv-start=10"])
        .stdin(input)
        .run();

    // No warning should be printed
    assert!(
        !stderr.contains("warning"),
        "Should not have warning for positive equiv-start, got stderr: {}",
        stderr
    );

    // Verify output starts from 10
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(
        lines[0].ends_with("\t10"),
        "First line should end with tab 10"
    );
    assert!(
        lines[1].ends_with("\t11"),
        "Second line should end with tab 11"
    );
    assert!(
        lines[2].ends_with("\t10"),
        "Third line should end with tab 10 (reused ID)"
    );
}
