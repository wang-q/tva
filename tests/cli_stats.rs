use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn stats_basic_help() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Calculates summary statistics"));
}

#[test]
fn stats_count() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // With header, output should be "count\n6"
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "count");
    assert_eq!(lines[1], "6");
}

#[test]
fn stats_sum() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--sum")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Sum of 10+20+30+40+50+60 = 210
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_sum");
    assert_eq!(lines[1], "210");
}

#[test]
fn stats_mean() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--mean")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Mean of 210 / 6 = 35
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_mean");
    assert_eq!(lines[1], "35");
}

#[test]
fn stats_min_max() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--min")
        .arg("value")
        .arg("--max")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Min 10, Max 60
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    // Output order: min, max (based on arg order usually, but let's check header)
    assert_eq!(lines[0], "value_min\tvalue_max");
    assert_eq!(lines[1], "10\t60");
}

#[test]
fn stats_group_by() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("header1")
        .arg("--count")
        .arg("--sum")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Groups: A (3 rows, sum 60), B (3 rows, sum 150)
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "header1\tcount\tvalue_sum");
    
    // Output should be sorted by key
    assert_eq!(lines[1], "A\t3\t60");
    assert_eq!(lines[2], "B\t3\t150");
}

#[test]
fn stats_group_by_multiple() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("header1,header2")
        .arg("--sum")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Groups: A,X (30); A,Y (30); B,X (40); B,Y (110)
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0], "header1\theader2\tvalue_sum");
    assert_eq!(lines[1], "A\tX\t30");
    assert_eq!(lines[2], "A\tY\t30");
    assert_eq!(lines[3], "B\tX\t40");
    assert_eq!(lines[4], "B\tY\t110");
}
