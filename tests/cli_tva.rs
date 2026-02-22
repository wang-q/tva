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
fn command_dedup() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("dedup")
        .arg("tests/genome/ctg.tsv")
        .arg("tests/genome/ctg.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("dedup")
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
