use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
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
fn nl_basic_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_start_number_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--start-number 10 input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--start-number")
        .arg("10")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_start_number_short_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-n 10 input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("-n")
        .arg("10")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_start_number_negative_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-n -10 input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--start-number")
        .arg("-10")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_empty_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("empty-file.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/empty-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_empty_file_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-H empty-file.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("-H")
        .arg("tests/data/nl/empty-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_string_unicode_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-s LineNum_àßß input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("-s")
        .arg("LineNum_àßß")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_and_header_string_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header -s line_num input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("-s")
        .arg("line_num")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_delimiter_colon_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--delimiter : input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--delimiter")
        .arg(":")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_delimiter_underscore_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-d _ input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("-d")
        .arg("_")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_and_delimiter_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header -d ^ input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("-d")
        .arg("^")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_header_string_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("--header-string LINENUM input1.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header-string")
        .arg("LINENUM")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("input1.txt input2.txt empty-file.txt one-line-file.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/empty-file.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_reordered_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_block("input1.txt one-line-file.txt input2.txt empty-file.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/empty-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_with_leading_empty_from_gold() -> anyhow::Result<()> {
    let expected = expected_block(
        "empty-file.txt input1.txt one-line-file.txt input2.txt input1.txt",
    );

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/empty-file.txt")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_header_second_from_gold() -> anyhow::Result<()> {
    let expected = expected_block("-H input2.txt input2.txt input2.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("-H")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/input2.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_header_mixed_from_gold() -> anyhow::Result<()> {
    let expected = expected_block(
        "--header input1.txt input2.txt empty-file.txt one-line-file.txt",
    );

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/empty-file.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_header_string_from_gold() -> anyhow::Result<()> {
    let expected = expected_block(
        "--header -s LINENUM empty-file.txt input1.txt one-line-file.txt input2.txt input1.txt",
    );

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("-s")
        .arg("LINENUM")
        .arg("tests/data/nl/empty-file.txt")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/input1.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_multi_file_header_start_number_from_gold() -> anyhow::Result<()> {
    let expected = expected_block(
        "--header -n 10 input1.txt one-line-file.txt input2.txt empty-file.txt",
    );

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("-n")
        .arg("10")
        .arg("tests/data/nl/input1.txt")
        .arg("tests/data/nl/one-line-file.txt")
        .arg("tests/data/nl/input2.txt")
        .arg("tests/data/nl/empty-file.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_stdin_from_gold() -> anyhow::Result<()> {
    let expected = expected_stdin_block("====[cat input1.txt | number-lines]====");

    let input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("nl").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_stdin_multi_file_header_from_gold() -> anyhow::Result<()> {
    let expected = expected_stdin_block(
        "====[cat input1.txt input2.txt | number-lines --header]====",
    );

    let input1 = fs::read_to_string("tests/data/nl/input1.txt").unwrap();
    let input2 = fs::read_to_string("tests/data/nl/input2.txt").unwrap();
    let input = format!("{input1}{input2}");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_stdin_with_args_from_gold() -> anyhow::Result<()> {
    let expected =
        expected_stdin_block("====[cat input1.txt | number-lines -- input2.txt -]====");

    let stdin_input = fs::read_to_string("tests/data/nl/input1.txt").unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("tests/data/nl/input2.txt")
        .arg("stdin")
        .write_stdin(stdin_input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn nl_help_displays_usage() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("nl").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Reads TSV data from files or standard input",
    ));

    Ok(())
}

#[test]
fn nl_version_matches_tva() -> anyhow::Result<()> {
    let mut cmd_tva = cargo_bin_cmd!("tva");
    let tva_version = cmd_tva.arg("--version").output().unwrap();
    let tva_version_str = String::from_utf8(tva_version.stdout).unwrap();
    let tva_version_num = tva_version_str
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();

    let mut cmd_nl = cargo_bin_cmd!("tva");
    let nl_version = cmd_nl.arg("nl").arg("--version").output().unwrap();
    let nl_version_str = String::from_utf8(nl_version.stdout).unwrap();
    let nl_version_num = nl_version_str
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();

    assert_eq!(nl_version_num, tva_version_num);

    Ok(())
}

#[test]
fn nl_error_nosuchfile() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("nl").arg("tests/data/nl/nosuchfile.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not open"));

    Ok(())
}

#[test]
fn nl_error_unknown_option() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("nl")
        .arg("--nosuchparam")
        .arg("tests/data/nl/input1.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--nosuchparam"));

    Ok(())
}
