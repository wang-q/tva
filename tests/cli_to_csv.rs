use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::Builder;

#[test]
fn to_csv_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("csv")
        .write_stdin("a\tb\n1\t2\n")
        .assert()
        .success()
        .stdout("a,b\n1,2\n");
}

#[test]
fn to_csv_custom_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("csv")
        .arg("--delimiter")
        .arg(";")
        .write_stdin("a\tb\n1\t2\n")
        .assert()
        .success()
        .stdout("a;b\n1;2\n");
}

#[test]
fn to_csv_with_quotes() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("csv")
        .write_stdin("a\tb\n1,2\t3\n")
        .assert()
        .success()
        .stdout("a,b\n\"1,2\",3\n");
}

#[test]
fn to_csv_file() {
    let file = Builder::new().suffix(".tsv").tempfile().unwrap();
    let file_path = file.path().to_str().unwrap();
    fs::write(file_path, "a\tb\n1\t2\n").unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("csv")
        .arg(file_path)
        .assert()
        .success()
        .stdout("a,b\n1,2\n");
}
