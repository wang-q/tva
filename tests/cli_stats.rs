#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

const INPUT: &str = "header1\theader2\tvalue
A\tX\t10
A\tX\t20
A\tY\t30
B\tX\t40
B\tY\t50
B\tY\t60
";

#[test]
fn stats_basic_help() {
    let (stdout, _) = TvaCmd::new().args(&["stats", "--help"]).run();
    assert!(stdout.contains("Calculates summary statistics"));
}

#[test]
fn stats_count() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "count\n6\n");
}

#[test]
fn stats_sum() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_sum\n210\n");
}

#[test]
fn stats_mean() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_mean\n35\n");
}

#[test]
fn stats_min_max() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--min", "value", "--max", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_min\tvalue_max\n10\t60\n");
}

#[test]
fn stats_median() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--median", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_median\n35\n");
}

#[test]
fn stats_variance_stdev() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--variance",
            "value",
            "--stdev",
            "value",
            "-p",
            "12",
        ])
        .stdin(INPUT)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "value_variance\tvalue_stdev");

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "350");

    let stdev: f64 = parts[1].parse().unwrap();
    common::assert_close(stdev, 18.708286933869708, 1e-6);
}

#[test]
fn stats_mad() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mad", "value"])
        .stdin(INPUT)
        .run();

    // Data: 10, 20, 30, 40, 50, 60
    // Median: 35
    // Deviations: |10-35|=25, |20-35|=15, |30-35|=5, |40-35|=5, |50-35|=15, |60-35|=25
    // Sorted Deviations: 5, 5, 15, 15, 25, 25
    // Median Deviation: (15 + 15) / 2 = 15
    // MAD = 15 * 1.4826 = 22.239
    let output = stdout.trim();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2); // Header + Value
    assert_eq!(lines[0], "value_mad");
    let mad: f64 = lines[1].parse().expect("MAD should be a number");
    common::assert_close(mad, 22.239, 1e-3);
}

#[test]
fn stats_first_last() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--first", "value", "--last", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_first\tvalue_last\n10\t60\n");
}

#[test]
fn stats_nunique_mode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--nunique",
            "header1",
            "--mode",
            "header1",
        ])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "header1_nunique\theader1_mode\n2\tA\n");
}

#[test]
fn stats_group_by() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "header1",
            "--count",
            "--sum",
            "value",
        ])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "header1\tcount\tvalue_sum\nA\t3\t60\nB\t3\t150\n");
}

#[test]
fn stats_group_by_multiple() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "header1,header2",
            "--sum",
            "value",
        ])
        .stdin(INPUT)
        .run();

    assert_eq!(
        stdout,
        "header1\theader2\tvalue_sum
A\tX\t30
A\tY\t30
B\tX\t40
B\tY\t110
"
    );
}

#[test]
fn stats_advanced_math() {
    let input = "val\n2\n8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--geomean",
            "val",
            "--harmmean",
            "val",
            "--range",
            "val",
            "--cv",
            "val",
            "-p",
            "12",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "val_geomean\tval_harmmean\tval_range\tval_cv");

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[0], "4");
    assert_eq!(parts[1], "3.2");
    assert_eq!(parts[2], "6");

    let cv: f64 = parts[3].parse().unwrap();
    common::assert_close(cv, 0.848528, 1e-5);
}

#[test]
fn stats_quartiles() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--q1", "val", "--q3", "val", "--iqr", "val",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, "val_q1\tval_q3\tval_iqr\n2\t4\t2\n");
}

#[test]
fn stats_string_ops() {
    let input = "txt\nA\nB\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--unique", "txt", "--collapse", "txt"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "txt_unique\ttxt_collapse");

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts.len(), 2);

    let unique_val = parts[0];
    assert!(unique_val == "A,B" || unique_val == "B,A");

    let collapse_val = parts[1];
    assert_eq!(collapse_val, "A,B,A");
}

#[test]
fn stats_rand() {
    let input = "val\n100\n200\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--rand", "val"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "val_rand");

    let val = lines[1];
    assert!(val == "100" || val == "200");
}

const INPUT_MISSING: &str = "A\t10
A\t
B\t20
B\t
";

const INPUT_ALL_MISSING: &str = "A\t
A\t
";

#[test]
fn stats_replace_missing() {
    // Mean of all missing values is nan.
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0.0"])
        .stdin(INPUT_ALL_MISSING)
        .run();

    assert_eq!(stdout.trim(), "0");

    // Default behavior (nan)
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2"])
        .stdin(INPUT_ALL_MISSING)
        .run();

    assert_eq!(stdout.trim(), "nan");
}

#[test]
fn stats_missing_count_ops() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--missing-count",
            "2",
            "--not-missing-count",
            "2",
            "--count",
        ])
        .stdin(INPUT_MISSING)
        .run();

    // Total rows: 4.
    // Col 2: "10", "", "20", "".
    // Missing: 2. Not Missing: 2. Count: 4.
    // Note: 'count' (OpKind::Count) is currently forced to be the first column by stats.rs logic (arg_index: 0).
    assert_eq!(stdout.trim(), "4\t2\t2");
}

#[test]
fn stats_mode_count() {
    let input = "A\nA\nB";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mode-count", "1", "--mode", "1"])
        .stdin(input)
        .run();

    // Mode: A. Count: 2.
    assert_eq!(stdout.trim(), "2\tA");
}

#[test]
fn stats_unique_count_alias() {
    let input = "A\nA\nB";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique-count", "1"]) // Should work same as --nunique
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "2");
}

#[test]
fn stats_quantile() {
    let input = "A\t10
A\t20
B\t30
B\t40
B\t50
";
    // Quantile 0.5 (Median), 0.25, 0.75
    let (stdout, stderr) = TvaCmd::new()
        .args(&["stats", "--quantile", "2:0.5,0.25,0.75"])
        .stdin(input)
        .run();

    if !stderr.is_empty() {
        println!("STDERR: {}", stderr);
    }

    // Data: 10, 20, 30, 40, 50. Sorted: 10, 20, 30, 40, 50.
    // 0.5 -> 30
    // 0.25 -> 20
    // 0.75 -> 40
    assert_eq!(stdout.trim(), "30\t20\t40");
}

#[test]
fn stats_values_alias() {
    let input = "A\nB";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--values", "1"]) // Alias for --collapse
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "A,B");
}

#[test]
fn stats_unique_values_alias() {
    let input = "A\nA\nB";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique-values", "1"]) // Alias for --unique
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "A,B");
}

#[test]
fn stats_count_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--count-header", "my_count"])
        .stdin("col1\nA\n")
        .run();

    assert_eq!(stdout, "my_count\n1\n");
}

#[test]
fn stats_count_header_implicit() {
    // --count-header should enable count even without --count
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count-header", "my_count"])
        .stdin("col1\nA\n")
        .run();

    assert_eq!(stdout, "my_count\n1\n");
}

#[test]
fn stats_write_header() {
    let input = "A\t10
A\t20
B\t30
B\t40
B\t50
";
    // No input header (-H not set), but force output header (-w)
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--write-header", "--sum", "2"])
        .stdin(input)
        .run();

    assert_eq!(stdout, "field2_sum\n150\n");
}
