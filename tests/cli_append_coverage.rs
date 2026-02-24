use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_append_track_source() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_source_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--source-header")
        .arg("filename")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("filename\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_file_mapping() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--file")
        .arg("custom_label=tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom_label\tfield1\tfield2\tfield3"));
}

#[test]
fn test_append_invalid_file_mapping() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--file")
        .arg("invalid_format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid --file value `invalid_format`; expected LABEL=FILE"));
}

#[test]
fn test_append_invalid_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--delimiter")
        .arg("TooLong")
        .assert()
        .failure()
        .stderr(predicate::str::contains("delimiter must be a single byte"));
}

#[test]
fn test_append_stdin_default() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .write_stdin("field1\tfield2\nval1\tval2\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdin\tfield1\tfield2"));
}

#[test]
fn test_append_custom_delimiter() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("--delimiter")
        .arg(":")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2:field1\tfield2\tfield3"));
}

#[test]
fn test_append_subdir_filename_label() {
    // Tests that path/to/file.tsv becomes label "file"
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("append")
        .arg("--track-source")
        .arg("tests/data/append/input3x2.tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("input3x2\tfield1"));
}
