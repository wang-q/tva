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
fn stats_median() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--median")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Values: 10, 20, 30, 40, 50, 60
    // Median: (30 + 40) / 2 = 35
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_median");
    assert_eq!(lines[1], "35");
}

#[test]
fn stats_variance_stdev() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--variance")
        .arg("value")
        .arg("--stdev")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Values: 10, 20, 30, 40, 50, 60
    // Mean: 35
    // Variance: ((10-35)^2 + (20-35)^2 + ... + (60-35)^2) / 5
    // = (625 + 225 + 25 + 25 + 225 + 625) / 5
    // = 1750 / 5 = 350
    // Stdev: sqrt(350) ≈ 18.708286933869708

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_variance\tvalue_stdev");

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "350");
    // Check stdev with some tolerance
    let stdev: f64 = parts[1].parse().unwrap();
    assert!((stdev - 18.708286933869708).abs() < 1e-6);
}

#[test]
fn stats_mad() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--mad")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Median = 35
    // Deviations: 25, 15, 5, 5, 15, 25
    // Sorted Deviations: 5, 5, 15, 15, 25, 25
    // Median of Deviations: 15

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_mad");
    assert_eq!(lines[1], "15");
}

#[test]
fn stats_first_last() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--first")
        .arg("value")
        .arg("--last")
        .arg("value")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_first\tvalue_last");
    assert_eq!(lines[1], "10\t60");
}

#[test]
fn stats_nunique_mode() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--nunique")
        .arg("header1")
        .arg("--mode")
        .arg("header1")
        .arg("tests/data/stats/input.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // header1 values: A, A, A, B, B, B
    // nunique: 2
    // mode: A (tie with B, but A comes first lexicographically)

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "header1_nunique\theader1_mode");
    assert_eq!(lines[1], "2\tA");
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

#[test]
fn stats_advanced_math() {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    writeln!(file, "val").unwrap();
    writeln!(file, "2").unwrap();
    writeln!(file, "8").unwrap();
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--geomean")
        .arg("val")
        .arg("--harmmean")
        .arg("val")
        .arg("--range")
        .arg("val")
        .arg("--cv")
        .arg("val")
        .arg(path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Data: 2, 8
    // Mean: 5
    // GeoMean: sqrt(16) = 4
    // HarmMean: 2 / (1/2 + 1/8) = 2 / 0.625 = 3.2
    // Range: 8 - 2 = 6
    // Variance: ((2-5)^2 + (8-5)^2) / 1 = 18
    // Stdev: sqrt(18) ≈ 4.24264
    // CV: 4.24264 / 5 ≈ 0.848528

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "val_geomean\tval_harmmean\tval_range\tval_cv");
    
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[0], "4");
    assert_eq!(parts[1], "3.2");
    assert_eq!(parts[2], "6");
    
    let cv: f64 = parts[3].parse().unwrap();
    assert!((cv - 0.848528).abs() < 1e-5);
}

#[test]
fn stats_quartiles() {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    writeln!(file, "val").unwrap();
    for i in 1..=5 {
        writeln!(file, "{}", i).unwrap();
    }
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--q1")
        .arg("val")
        .arg("--q3")
        .arg("val")
        .arg("--iqr")
        .arg("val")
        .arg(path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Data: 1, 2, 3, 4, 5
    // Q1 (25th): 2
    // Q3 (75th): 4
    // IQR: 2

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "val_q1\tval_q3\tval_iqr");
    assert_eq!(lines[1], "2\t4\t2");
}

#[test]
fn stats_string_ops() {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    writeln!(file, "txt").unwrap();
    writeln!(file, "A").unwrap();
    writeln!(file, "B").unwrap();
    writeln!(file, "A").unwrap();
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--unique")
        .arg("txt")
        .arg("--collapse")
        .arg("txt")
        .arg(path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "txt_unique\ttxt_collapse");
    
    // Unique: A,B (sorted)
    // Collapse: A,B,A (input order preserved usually, but implementation dependent? 
    // Actually implementation might just join. Let's check output.)
    // If unique sorts, it should be A,B.
    
    let unique_val = lines[1].split('\t').next().unwrap();
    // Unique likely comma separated
    assert!(unique_val == "A,B" || unique_val == "B,A");

    let collapse_val = lines[1].split('\t').nth(1).unwrap();
    assert_eq!(collapse_val, "A,B,A");
}

#[test]
fn stats_rand() {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    writeln!(file, "val").unwrap();
    writeln!(file, "100").unwrap();
    writeln!(file, "200").unwrap();
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("stats")
        .arg("--header")
        .arg("--rand")
        .arg("val")
        .arg(path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "val_rand");
    
    let val = lines[1];
    assert!(val == "100" || val == "200");
}
