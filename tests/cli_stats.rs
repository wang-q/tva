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
    assert!((stdev - 18.708286933869708).abs() < 1e-6);
}

#[test]
fn stats_mad() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mad", "value"])
        .stdin(INPUT)
        .run();

    assert_eq!(stdout, "value_mad\n15\n");
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
    assert!((cv - 0.848528).abs() < 1e-5);
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
