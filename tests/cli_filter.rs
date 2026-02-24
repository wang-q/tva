use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn filter_numeric_gt_basic() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("value:10")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tvalue");
    assert_eq!(lines[1], "2\t15");
    assert_eq!(lines[2], "3\t20");

    Ok(())
}

#[test]
fn filter_str_eq_basic() -> anyhow::Result<()> {
    let input = "id\tcolor\n1\tred\n2\tblue\n3\tred\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--str-eq")
        .arg("color:red")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tcolor");
    assert_eq!(lines[1], "1\tred");
    assert_eq!(lines[2], "3\tred");

    Ok(())
}

#[test]
fn filter_regex_basic() -> anyhow::Result<()> {
    let input = "id\tname\n1\talice\n2\tbob\n3\talex\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--regex")
        .arg("name:^al")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\talice");
    assert_eq!(lines[2], "3\talex");

    Ok(())
}

#[test]
fn filter_count_and_invert() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n2\t15\n3\t20\n";
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("value:10")
        .arg("--invert")
        .arg("--count")
        .write_stdin(input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim().starts_with("1"));

    Ok(())
}

#[test]
fn filter_invalid_field_list_reports_error() -> anyhow::Result<()> {
    let input = "id\tvalue\n1\t5\n";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("filter")
        .arg("--header")
        .arg("--gt")
        .arg("0:10")
        .write_stdin(input);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("tva filter:"));

    Ok(())
}
