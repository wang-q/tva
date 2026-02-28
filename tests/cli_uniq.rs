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
