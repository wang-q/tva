#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

// --- Helper Functions and Constants ---

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

const INPUT_BASIC: &str = "header1\theader2\tvalue
A\tX\t10
A\tX\t20
A\tY\t30
B\tX\t40
B\tY\t50
B\tY\t60
";

const INPUT_NEW: &str = "A\t10
A\t20
B\t30
B\t40
B\t50
";

const INPUT_5FIELD_A: &str = "color\tpattern\tlength\twidth\theight
red\tsolid\t10\t4\t7
red\tstriped\t8\t6\t6
blue\tsolid\t16\t2\t4
green\tsolid\t11\t5.5\t3.2
blue\tstriped\t12\t1\t2
blue\tsolid\t14\t4\t3
green\tsolid\t7.4\t6.0\t5.4
";

const INPUT_5FIELD_B: &str = "color\tpattern\tlength\twidth\theight
red\tsolid\t6\t2\t5
赤\t水玉模様\t8\t6\t6
青\t弁慶縞\t10\t5.5\t4.5
赤\t水玉模様\t9\t7\t8
";

const INPUT_5FIELD_C: &str = "color\tpattern\tlength\twidth\theight
red\tchecked\t10\t4\t7
";

const INPUT_5FIELD_D: &str = "color\tpattern\tlength\twidth\theight
red\tsolid\t0.11\t0.11\t0.12345678901234567
red\tplaid\t0.011\t0.11\t0.012345678901234567
blue\tplaid\t0.111\t0.11\t0.2345678901234567891
blue\tsolid\t0.1\t0.11\t0.1234567899876543211
green\tplaid\t0.11\t0.11\t0.1111111133333333333
red\tsolid\t0.1111\t0.11\t0.3333333311111111111
";

const INPUT_5FIELD_HEADER_ONLY: &str = "color\tpattern\tlength\twidth\theight\n";

const INPUT_1FIELD_A: &str = "size
10
small

small
8
10
";

// DOS line endings
const INPUT_1FIELD_A_DOS: &str = "size\r\n10\r\nsmall\r\n\r\nsmall\r\n8\r\n10\r\n";

const INPUT_1FIELD_B: &str = "size
9
medium
10
";

const INPUT_MISSING: &str = "A\t10
A\t
B\t20
B\t
";

const INPUT_ALL_MISSING: &str = "A\t
A\t
";

// --- Tests from cli_stats.rs ---

#[test]
fn stats_basic_help() {
    let (stdout, _) = TvaCmd::new().args(&["stats", "--help"]).run();
    assert!(stdout.contains("Calculates summary statistics"));
}

#[test]
fn stats_count() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(INPUT_BASIC)
        .run();

    assert_eq!(stdout, "count\n6\n");
}

#[test]
fn stats_sum() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "value"])
        .stdin(INPUT_BASIC)
        .run();

    assert_eq!(stdout, "value_sum\n210\n");
}

#[test]
fn stats_mean() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "value"])
        .stdin(INPUT_BASIC)
        .run();

    assert_eq!(stdout, "value_mean\n35\n");
}

#[test]
fn stats_min_max() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--min", "value", "--max", "value"])
        .stdin(INPUT_BASIC)
        .run();

    assert_eq!(stdout, "value_min\tvalue_max\n10\t60\n");
}

#[test]
fn stats_median() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--median", "value"])
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
        .stdin(INPUT_BASIC)
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
    assert!(unique_val == "A|B" || unique_val == "B|A");

    let collapse_val = parts[1];
    assert_eq!(collapse_val, "A|B|A");
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

    assert_eq!(stdout.trim(), "A|B");
}

#[test]
fn stats_unique_values_alias() {
    let input = "A\nA\nB";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique-values", "1"]) // Alias for --unique
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "A|B");
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

// --- Tests from cli_stats_new.rs ---

#[test]
fn stats_retain_alias() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--retain", "2"])
        .stdin(INPUT_NEW)
        .run();

    // retain should act like first
    assert_eq!(stdout.trim(), "10");
}

#[test]
fn stats_var_alias() {
    let input = "val\n2\n4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--var", "val"])
        .stdin(input)
        .run();

    // Variance of 2,4 is 2. (Mean=3, (1+1)/1 = 2)
    assert_eq!(stdout.trim(), "val_variance\n2");
}

#[test]
fn stats_custom_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "2:my_sum", "--write-header"])
        .stdin(INPUT_NEW)
        .run();

    assert_eq!(stdout.trim(), "my_sum\n150");
}

#[test]
fn stats_custom_header_multiple() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "2:S", "--write-header"])
        .stdin(INPUT_NEW)
        .run();

    assert_eq!(stdout.trim(), "S\n150");
}

#[test]
fn stats_custom_header_quantile() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--quantile", "2:0.5:Median", "--write-header"])
        .stdin(INPUT_NEW)
        .run();

    // Quantile of 10,20,30,40,50 is 30. Header should be Median.
    assert_eq!(stdout.trim(), "Median\n30");
}

#[test]
fn stats_replace_missing_input_side() {
    let input = "A\t10
A\t
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "5");
}

// --- Tests from cli_stats_more_features.rs ---

#[test]
fn stats_exclude_missing() {
    let input = "A\t10
A\t
A\t20
";
    // With -r 0: (10+0+20)/3 = 10.
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0"])
        .stdin(input)
        .run();
    assert_eq!(stdout.trim(), "10");
}

#[test]
fn stats_custom_delimiter() {
    let input = "A,10
A,20
B,30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--write-header", "--delimiter", ",", "--sum", "2"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "field2_sum\n60");
}

#[test]
fn stats_custom_delimiter_group() {
    let input = "A,10
A,20
B,30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "-d", ",", "-g", "1", "--sum", "2"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "A\t30\nB\t30");
}

// --- Tests from cli_stats_tsv_utils.rs ---

#[test]
fn tsv_utils_test_20_basic_count_min_max() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--count", "--min", "3,4,5", "--max", "3,4,5",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains(
        "count\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"
    ));
    assert!(stdout.contains("7\t7.4\t1\t2\t16\t6\t7"));
}

#[test]
fn tsv_utils_test_28_group_by_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--count",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("blue\t3\t12\t1\t2\t16\t4\t4"));
    assert!(stdout.contains("red\t2\t8\t4\t6\t10\t6\t7"));
}

#[test]
fn tsv_utils_test_34_group_by_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1,2",
            "--count",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("blue\tsolid\t2\t14\t2\t3\t16\t4\t4"));
    assert!(stdout.contains("blue\tstriped\t1\t12\t1\t2\t12\t1\t2"));
    assert!(stdout.contains("green\tsolid\t2\t7.4\t5.5\t3.2\t11\t6\t5.4"));
    assert!(stdout.contains("red\tsolid\t1\t10\t4\t7\t10\t4\t7"));
    assert!(stdout.contains("red\tstriped\t1\t8\t6\t6\t8\t6\t6"));
}

#[test]
fn tsv_utils_test_42_group_by_range() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1-2",
            "--count",
            "--min",
            "3-5",
            "--max",
            "5-3",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min"));
    assert!(stdout.contains("height_max"));
    assert!(stdout.contains("width_max"));
    assert!(stdout.contains("length_max"));
}

#[test]
fn tsv_utils_test_50_group_by_names() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "color,pattern",
            "--count",
            "--min",
            "length-height",
            "--max",
            "height-length",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min"));
    assert!(stdout.contains("red\tsolid\t1\t10\t4\t7\t10\t4\t7"));
}

#[test]
fn tsv_utils_test_58_multi_file() {
    let file_a = create_file(INPUT_5FIELD_A);
    let file_b = create_file(INPUT_5FIELD_B);
    let file_c = create_file(INPUT_5FIELD_C);
    let file_empty = create_file("");
    let file_header_only = create_file(INPUT_5FIELD_HEADER_ONLY);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--count",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
            file_a.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
            file_c.path().to_str().unwrap(),
            file_empty.path().to_str().unwrap(),
            file_header_only.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains(
        "count\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"
    ));
    assert!(stdout.contains("12\t6\t1\t2\t16\t7\t8"));
}

#[test]
fn tsv_utils_test_97_multi_file_unique_count() {
    let file_a = create_file(INPUT_5FIELD_A);
    let file_empty = create_file("");
    let file_b = create_file(INPUT_5FIELD_B);
    let file_header_only = create_file(INPUT_5FIELD_HEADER_ONLY);
    let file_c = create_file(INPUT_5FIELD_C);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--count",
            "--nunique",
            "1,2,3,4,5",
            file_a.path().to_str().unwrap(),
            file_empty.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
            file_header_only.path().to_str().unwrap(),
            file_c.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("16\t6\t6\t10\t8\t11"));
}

#[test]
fn tsv_utils_test_103_group_by_unique_count() {
    let file_a = create_file(INPUT_5FIELD_A);
    let file_empty = create_file("");
    let file_b = create_file(INPUT_5FIELD_B);
    let file_header_only = create_file(INPUT_5FIELD_HEADER_ONLY);
    let file_c = create_file(INPUT_5FIELD_C);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--group-by",
            "1",
            "--count",
            "--nunique",
            "2,3,4,5",
            file_a.path().to_str().unwrap(),
            file_empty.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
            file_header_only.path().to_str().unwrap(),
            file_c.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("red\t4\t3\t3\t3\t3"));
    assert!(stdout.contains("blue\t3\t2\t3\t3\t3"));
    assert!(stdout.contains("green\t2\t1\t2\t2\t2"));
    assert!(stdout.contains("赤\t2\t1\t2\t2\t2"));
    assert!(stdout.contains("青\t1\t1\t1\t1\t1"));
}

#[test]
fn tsv_utils_test_154_count_unique_count_files() {
    let file_a = create_file(INPUT_1FIELD_A);
    let file_b = create_file(INPUT_1FIELD_B);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--count",
            "--nunique",
            "1",
            file_a.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("count\tsize_nunique"));
    assert!(stdout.contains("9\t6"));
}

#[test]
fn tsv_utils_test_243_mean() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--min",
            "3-5",
            "--max",
            "3-5",
            "--mean",
            "3-5",
        ])
        .stdin(INPUT_5FIELD_D)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert!(lines[0].contains("length_mean\twidth_mean\theight_mean"));

    let red_line = lines
        .iter()
        .find(|l| l.starts_with("red"))
        .expect("Should have red line");
    let red_parts: Vec<&str> = red_line.split('\t').collect();
    // red length_mean ~ 0.0773666...
    common::assert_close(
        red_parts[red_parts.len() - 3].parse().unwrap(),
        0.077366666,
        1e-4,
    );

    let blue_line = lines
        .iter()
        .find(|l| l.starts_with("blue"))
        .expect("Should have blue line");
    let blue_parts: Vec<&str> = blue_line.split('\t').collect();
    // blue length_mean ~ 0.1055
    common::assert_close(
        blue_parts[blue_parts.len() - 3].parse().unwrap(),
        0.1055,
        1e-4,
    );
}

#[test]
fn tsv_utils_test_advanced_stats_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--first",
            "1",
            "--last",
            "1",
            "--median",
            "3",
            "--mad",
            "3",
            "--variance",
            "3",
            "--stdev",
            "3",
            "--mode",
            "1",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color_first"));
    assert!(stdout.contains("color_last"));
    assert!(stdout.contains("length_median"));
    assert!(stdout.contains("length_mad"));
    assert!(stdout.contains("length_variance"));
    assert!(stdout.contains("length_stdev"));
    assert!(stdout.contains("color_mode"));
    assert!(stdout.contains("red"));
    assert!(stdout.contains("green"));
    assert!(stdout.contains("11"));
    assert!(stdout.contains("3"));
    assert!(stdout.contains("9.61"));
    assert!(stdout.contains("3.1"));
    assert!(stdout.contains("blue"));
}

#[test]
fn tsv_utils_error_test_missing_file() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--count", "no_such_file.tsv"])
        .run_fail();

    assert!(stderr.contains("os error 2"));
}

#[test]
fn tsv_utils_error_test_invalid_field_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--nunique", "0"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn tsv_utils_error_test_invalid_field_list_empty_element() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--nunique", "2,"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("empty field list element"));
}

#[test]
fn tsv_utils_test_stdin_group_by() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "2",
            "--count",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("pattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("solid\t5\t7.4\t2\t3\t16\t6\t7"));
    assert!(stdout.contains("striped\t2\t8\t1\t2\t12\t6\t6"));
}

#[test]
fn tsv_utils_test_stdin_mixed_files() {
    let file_a = create_file(INPUT_5FIELD_A);
    let file_c = create_file(INPUT_5FIELD_C);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "2",
            "--count",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
            file_a.path().to_str().unwrap(),
            "-",
            file_c.path().to_str().unwrap(),
        ])
        .stdin(INPUT_5FIELD_B)
        .run();

    assert!(stdout.contains("solid\t6"));
    assert!(stdout.contains("striped\t2"));
    assert!(stdout.contains("checked\t1"));
}

#[test]
fn tsv_utils_test_field_out_of_bounds_multi_file_behavior() {
    let file_5field = create_file(INPUT_5FIELD_A);
    let file_1field = create_file(INPUT_1FIELD_A);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--group-by",
            "2",
            "--nunique",
            "1",
            file_5field.path().to_str().unwrap(),
            file_1field.path().to_str().unwrap(),
        ])
        .run();

    // The empty string group (from file_1field which lacks field 2) has 5 unique values
    assert!(stdout.contains("\t5"));
}

#[test]
fn tsv_utils_test_crlf_handling() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--count", "--nunique", "1"])
        .stdin(INPUT_1FIELD_A_DOS)
        .run();

    // tva includes \r in the value, so we have 5 unique values (size\r, 10\r, small\r, \r, 8\r)
    assert!(stdout.contains("7\t5"));
}

#[test]
fn tsv_utils_test_extended_stats() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--first",
            "3",
            "--last",
            "3",
            "--median",
            "3",
            "--mad",
            "3",
            "--variance",
            "3",
            "--stdev",
            "3",
            "--mode",
            "1",
        ])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("length_first\tlength_last\tlength_median\tlength_mad\tlength_variance\tlength_stdev\tcolor_mode"));
    // MAD of length:
    // Values: 10, 8, 16, 11, 12, 14, 7.4
    // Sorted: 7.4, 8, 10, 11, 12, 14, 16
    // Median: 11
    // Deviations: |7.4-11|=3.6, |8-11|=3, |10-11|=1, |11-11|=0, |12-11|=1, |14-11|=3, |16-11|=5
    // Sorted Deviations: 0, 1, 1, 3, 3, 3.6, 5
    // Median Deviation: 3
    // MAD = 3 * 1.4826 = 4.4478

    let output = stdout.trim();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[0], "10"); // first
    assert_eq!(parts[1], "7.4"); // last
    assert_eq!(parts[2], "11"); // median

    let mad: f64 = parts[3].parse().expect("MAD should be a number");
    assert!((mad - 4.4478).abs() < 1e-4);

    // Variance/Stdev checks
    // Variance ~ 9.6133
    // Stdev ~ 3.1005
    assert!(parts[4].starts_with("9.6133"));
    assert!(parts[5].starts_with("3.1005"));

    assert_eq!(parts[6], "blue"); // mode
}

#[test]
fn tsv_utils_test_float_precision_defaults() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--min",
            "3,4,5",
            "--max",
            "3,4,5",
            "--mean",
            "3,4,5",
        ])
        .stdin(INPUT_5FIELD_D)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert!(lines[0].contains("color\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max\tlength_mean\twidth_mean\theight_mean"));

    let blue_line = lines
        .iter()
        .find(|l| l.starts_with("blue"))
        .expect("Should have blue line");
    let blue_parts: Vec<&str> = blue_line.split('\t').collect();
    // length_min: 0.1
    common::assert_close(blue_parts[1].parse().unwrap(), 0.1, 1e-4);
    // height_min: 0.123456...
    common::assert_close(blue_parts[3].parse().unwrap(), 0.1235, 1e-4);
    // length_mean: 0.1055
    common::assert_close(blue_parts[7].parse().unwrap(), 0.1055, 1e-4);

    let red_line = lines
        .iter()
        .find(|l| l.starts_with("red"))
        .expect("Should have red line");
    let red_parts: Vec<&str> = red_line.split('\t').collect();
    // length_mean: 0.077366... -> 0.0774
    common::assert_close(red_parts[7].parse().unwrap(), 0.0774, 1e-4);
}

#[test]
fn tsv_utils_error_test_non_numeric_group_by() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--group-by", "x", "--count"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("field name `x` requires header"));
}

#[test]
fn tsv_utils_error_test_field_not_found_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--sum", "width,len"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("unknown field name `len`"));
}

#[test]
fn tsv_utils_test_field_range_by_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "length-height"])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("length_sum\twidth_sum\theight_sum"));
    assert!(stdout.contains("78.4\t28.5\t30.6"));
}

#[test]
fn tsv_utils_test_1field() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--nunique", "1"])
        .stdin(INPUT_1FIELD_A)
        .run();

    assert!(stdout.contains("count\tsize_nunique"));
    assert!(stdout.contains("6\t4"));
}

#[test]
fn tsv_utils_test_empty_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin("")
        .run();

    assert!(stdout.contains("0"));
}

#[test]
fn tsv_utils_test_header_only() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(INPUT_5FIELD_HEADER_ONLY)
        .run();

    assert!(stdout.contains("0"));
}

#[test]
fn tsv_utils_error_test_zero_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--min", "0"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn tsv_utils_error_test_invalid_field_list() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--min", "1,,2"])
        .stdin(INPUT_5FIELD_A)
        .run_fail();

    assert!(stderr.contains("empty field list element"));
}

#[test]
fn tsv_utils_test_header_as_data() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1,2", "--count"])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tpattern"));
}

#[test]
fn tsv_utils_test_1field_no_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--count"])
        .stdin(INPUT_1FIELD_A)
        .run();

    assert!(stdout.contains("7"));
}

#[test]
fn tsv_utils_test_empty_file_no_header() {
    let (stdout, _) = TvaCmd::new().args(&["stats", "--count"]).stdin("").run();

    assert!(stdout.contains("0"));
}

#[test]
fn tsv_utils_test_multi_file_no_header() {
    let file_a = create_file(INPUT_1FIELD_A);
    let file_b = create_file(INPUT_1FIELD_B);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--count",
            "--nunique",
            "1",
            file_a.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("11\t7"));
}

#[test]
fn tsv_utils_test_no_header_group_by() {
    let file_a = create_file(INPUT_1FIELD_A);
    let file_b = create_file(INPUT_1FIELD_B);

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--group-by",
            "1",
            "--count",
            "--nunique",
            "1",
            file_a.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("size\t2\t1"));
    assert!(stdout.contains("10\t3\t1"));
    assert!(stdout.contains("small\t2\t1"));
    assert!(stdout.contains("8\t1\t1"));
    assert!(stdout.contains("9\t1\t1"));
    assert!(stdout.contains("medium\t1\t1"));
    assert!(stdout.contains("\t1\t1"));
}

#[test]
fn stats_error_quantile_invalid_prob() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--quantile", "1:1.5"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("probability must be between 0.0 and 1.0"));
}

#[test]
fn stats_error_quantile_no_prob() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--quantile", "1"])
        .stdin(INPUT_BASIC)
        .run_fail();

    // tva uses clap, so it might be a parsing error or custom validation
    // The current implementation expects field:prob format
    assert!(stderr.to_lowercase().contains("invalid quantile syntax"));
}

#[test]
fn stats_error_custom_header_multiple_fields() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--sum", "1,2:header"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("custom header is not allowed with multiple fields"));
}

#[test]
fn stats_error_exclude_and_replace_missing() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "stats",
            "--exclude-missing",
            "--replace-missing",
            "0",
            "--mean",
            "1",
        ])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("argument '--exclude-missing' cannot be used with '--replace-missing <replace-missing>'"));
}

#[test]
fn stats_error_delimiter_and_values_delimiter_same() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "-d", ",", "-v", ",", "--values", "1"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("values delimiter cannot be the same as field delimiter"));
}
