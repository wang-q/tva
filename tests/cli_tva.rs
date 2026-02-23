use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*; // Used for writing assertions

#[test]
fn command_invalid() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("foobar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("recognized"));

    Ok(())
}

#[test]
fn command_md() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("tests/genome/ctg.range.tsv")
        .arg("--num")
        .arg("-c")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(
        stdout.contains("| -----: | :--------: | --------------- |"),
        "separator"
    );
    assert!(stdout.contains("| 130218 |  ctg:I:2   | I:100001-230218 |"));

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("tests/genome/ctg.range.tsv")
        .arg("--fmt")
        .arg("--digits")
        .arg("2")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(
        stdout.contains("| ---------: | ---------- | --------------- |"),
        "separator"
    );
    assert!(stdout.contains("| 130,218.00 | ctg:I:2    | I:100001-230218 |"));

    Ok(())
}

#[test]
fn command_nl() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("nl").arg("tests/genome/ctg.tsv").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("1\tID\tchr_id"));
    assert!(lines[3].starts_with("4\tctg:I:2\tI"));

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header")
        .arg("tests/genome/ctg.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        lines[0],
        "line\tID\tchr_id\tchr_start\tchr_end\tchr_strand\tlength"
    );
    assert!(lines[1].starts_with("1\tctg:I:1\tI"));
    assert!(lines[3].starts_with("3\tctg:I:2\tI"));

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("nl")
        .arg("--header-string")
        .arg("linenum")
        .arg("tests/genome/ctg.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "linenum\tID\tchr_id\tchr_start\tchr_end\tchr_strand\tlength");

    Ok(())
}

#[test]
fn command_keep_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/genome/ctg.tsv")
        .arg("--")
        .arg("sort")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty());
    assert_eq!(
        lines[0],
        "ID\tchr_id\tchr_start\tchr_end\tchr_strand\tlength"
    );

    Ok(())
}
