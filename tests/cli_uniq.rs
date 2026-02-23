use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

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
    let output = cmd
        .arg("uniq")
        .write_stdin(input_dup)
        .output()
        .unwrap();
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
        .arg("tests/data/dedup/input1.tsv")
        .arg("tests/data/dedup/input2.tsv")
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
fn command_uniq_header_named_fields_equivalent_to_numeric_single_file() -> anyhow::Result<()> {
    let input = "tests/data/dedup/input1.tsv";

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
fn command_uniq_header_named_fields_equivalent_to_numeric_multiple_files() -> anyhow::Result<()> {
    let input1 = "tests/data/dedup/input1.tsv";
    let input2 = "tests/data/dedup/input2.tsv";

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

