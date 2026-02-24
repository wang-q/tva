use assert_cmd::cargo::cargo_bin_cmd;
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
fn command_uniq() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/genome/ctg.tsv")
        .arg("tests/genome/ctg.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/genome/ctg.tsv")
        .arg("-f")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("ctg:I:2\tI"));

    Ok(())
}

#[test]
fn command_uniq_stdin() -> anyhow::Result<()> {
    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();
    let input_dup = format!("{input}{input}");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("uniq").write_stdin(input_dup).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("2")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("ctg:I:2\tI"));

    Ok(())
}

#[test]
fn command_uniq_stdin_and_file() -> anyhow::Result<()> {
    let input = fs::read_to_string("tests/genome/ctg.tsv").unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("stdin")
        .arg("tests/genome/ctg.tsv")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    Ok(())
}

#[test]
fn command_uniq_header_single_header_across_files() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--header")
        .arg("tests/data/uniq/input1.tsv")
        .arg("tests/data/uniq/input2.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty());
    assert_eq!(lines[0], "f1\tf2\tf3\tf4\tf5");

    let header_count = lines
        .iter()
        .filter(|line| line.starts_with("f1\tf2\tf3\tf4\tf5"))
        .count();
    assert_eq!(header_count, 1);

    Ok(())
}

#[test]
fn command_uniq_header_named_fields_equivalent_to_numeric_single_file(
) -> anyhow::Result<()> {
    let input = "tests/data/uniq/input1.tsv";

    let mut cmd = cargo_bin_cmd!("tva");
    let output_numeric = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("3,4")
        .arg(input)
        .output()
        .unwrap();
    assert!(output_numeric.status.success());
    let stdout_numeric = String::from_utf8(output_numeric.stdout).unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output_named = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("f3,f4")
        .arg(input)
        .output()
        .unwrap();
    assert!(output_named.status.success());
    let stdout_named = String::from_utf8(output_named.stdout).unwrap();

    assert_eq!(stdout_numeric, stdout_named);

    Ok(())
}

#[test]
fn uniq_basic_input1_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_basic_header_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header input1.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--header")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_field0_whole_line_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-f 0 input1.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("0")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_header_field0_whole_line_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header -f 0 input1.tsv");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("0")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_fields1_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.tsv --fields 1");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1.tsv")
        .arg("--fields")
        .arg("1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_header_field2_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.tsv --header -f 2");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1.tsv")
        .arg("--header")
        .arg("-f")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_noheader_field2_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1_noheader.tsv -f 2");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1_noheader.tsv")
        .arg("-f")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_header_fields3_4_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.tsv --header -f 3,4");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1.tsv")
        .arg("--header")
        .arg("-f")
        .arg("3,4")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_header_named_fields3_4_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.tsv --header -f f3,f4");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1.tsv")
        .arg("--header")
        .arg("-f")
        .arg("f3,f4")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn uniq_noheader_fields3_4_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1_noheader.tsv -f 3,4");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("tests/data/uniq/input1_noheader.tsv")
        .arg("-f")
        .arg("3,4")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn command_uniq_header_named_fields_equivalent_to_numeric_multiple_files(
) -> anyhow::Result<()> {
    let input1 = "tests/data/uniq/input1.tsv";
    let input2 = "tests/data/uniq/input2.tsv";

    let mut cmd = cargo_bin_cmd!("tva");
    let output_numeric = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("3,4")
        .arg(input1)
        .arg(input2)
        .output()
        .unwrap();
    assert!(output_numeric.status.success());
    let stdout_numeric = String::from_utf8(output_numeric.stdout).unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output_named = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("f3,f4")
        .arg(input1)
        .arg(input2)
        .output()
        .unwrap();
    assert!(output_named.status.success());
    let stdout_named = String::from_utf8(output_named.stdout).unwrap();

    assert_eq!(stdout_numeric, stdout_named);

    Ok(())
}

#[test]
fn command_uniq_ignore_case() -> anyhow::Result<()> {
    let input = "key\n\
                 a\n\
                 A\n\
                 a\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("uniq").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 3);

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--ignore-case")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2);

    Ok(())
}

#[test]
fn command_uniq_repeated_and_at_least_max() -> anyhow::Result<()> {
    let input = "a\n\
                 a\n\
                 a\n\
                 b\n\
                 b\n\
                 c\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--repeated")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a", "b"]);

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--at-least")
        .arg("3")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a"]);

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--at-least")
        .arg("2")
        .arg("--max")
        .arg("3")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a", "a", "b"]);

    Ok(())
}

#[test]
fn command_uniq_equiv_and_number() -> anyhow::Result<()> {
    let input = "f1\tf2\n\
                 a\tX\n\
                 b\tY\n\
                 a\tZ\n\
                 a\tX\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--header")
        .arg("-f")
        .arg("1")
        .arg("--equiv")
        .arg("--number")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "f1\tf2\tequiv_id\tequiv_line");
    assert_eq!(lines[1], "a\tX\t1\t1");
    assert_eq!(lines[2], "b\tY\t2\t1");
    assert_eq!(lines[3], "a\tZ\t1\t2");
    assert_eq!(lines[4], "a\tX\t1\t3");

    Ok(())
}

#[test]
fn command_uniq_custom_delimiter_and_line_buffered() -> anyhow::Result<()> {
    let input = "k1_k2\n\
                 a_X\n\
                 a_Y\n\
                 b_Z\n\
                 a_X\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("--delimiter")
        .arg("_")
        .arg("--header")
        .arg("-f")
        .arg("1")
        .arg("--line-buffered")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines[0], "k1_k2");
    assert_eq!(lines[1], "a_X");
    assert_eq!(lines[2], "b_Z");

    Ok(())
}

#[test]
fn uniq_error_equiv_start_requires_equiv() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("2")
        .arg("--equiv-start")
        .arg("10")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva uniq: --equiv-start requires --equiv"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn uniq_error_equiv_header_requires_equiv() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("2")
        .arg("--equiv-header")
        .arg("id")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva uniq: --equiv-header requires --equiv"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn uniq_error_number_header_requires_number() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("2")
        .arg("--number-header")
        .arg("line")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("tva uniq: --number-header requires --number"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn uniq_error_zero_in_field_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("0-2")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("field index must be >= 1"),
        "stderr was: {}",
        stderr
    );

    Ok(())
}

#[test]
fn uniq_error_field_name_requires_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("uniq")
        .arg("-f")
        .arg("f1")
        .arg("tests/data/uniq/input1.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("requires header"), "stderr was: {}", stderr);

    Ok(())
}
