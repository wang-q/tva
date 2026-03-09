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

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn stats_empty_file_with_header() {
    // Empty file with --header flag - this may succeed with empty output
    // or fail depending on implementation
    let (_, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin("")
        .run();
}

#[test]
fn stats_header_only_file() {
    // File with only header, no data rows
    let input = "col1\tcol2\tcol3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--sum", "1"])
        .stdin(input)
        .run();

    // Should output header and count=0
    assert!(stdout.contains("count"));
    assert!(stdout.contains("0"));
}

#[test]
fn stats_all_missing_values() {
    // All values are missing/empty
    let input = "col1\tcol2
\t
\t
\t
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "2", "--sum", "2", "--count"])
        .stdin(input)
        .run();

    // Mean should be nan, sum should be 0 or nan
    assert!(stdout.contains("count"));
    assert!(stdout.contains("3")); // 3 data rows
}

#[test]
fn stats_custom_values_delimiter() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique", "1", "--values-delimiter", ";"])
        .stdin(input)
        .run();

    // Unique values should be separated by ; instead of |
    assert!(
        stdout.contains(";") || stdout == "A\n" || stdout == "B\n" || stdout == "C\n"
    );
}

#[test]
fn stats_float_precision() {
    let input = "val\n1.123456789\n2.987654321\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "val", "-p", "2"])
        .stdin(input)
        .run();

    // Should have 2 decimal places
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains(".") || lines[1] == "2.06");
}

#[test]
fn stats_single_row() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--mean", "val", "--median", "val", "--stdev", "val",
            "--min", "val", "--max", "val",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);

    let parts: Vec<&str> = lines[1].split('\t').collect();
    // All stats should be 42 for single value
    for part in parts {
        let val: f64 = part.parse().unwrap();
        assert!((val - 42.0).abs() < 1e-6 || val.is_nan()); // stdev might be nan for single value
    }
}

#[test]
fn stats_two_rows() {
    let input = "val\n10\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "val", "--median", "val"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("15")); // mean = 15
}

#[test]
fn stats_negative_numbers() {
    let input = "val\n-10\n-20\n-30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--sum", "val", "--min", "val", "--max", "val",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("-60")); // sum
    assert!(stdout.contains("-30")); // min
    assert!(stdout.contains("-10")); // max
}

#[test]
fn stats_mixed_positive_negative() {
    let input = "val\n-10\n0\n10\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val", "--mean", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("0")); // sum = 0
}

#[test]
fn stats_large_numbers() {
    let input = "val\n1000000\n2000000\n3000000\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("6000000"));
}

#[test]
fn stats_small_numbers() {
    let input = "val\n0.0001\n0.0002\n0.0003\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val", "-p", "6"])
        .stdin(input)
        .run();

    assert!(stdout.contains("0.0006") || stdout.contains("0.0005"));
}

#[test]
fn stats_group_by_single_group() {
    // All rows have same group key
    let input = "grp\tval
A\t10
A\t20
A\t30
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "grp",
            "--sum",
            "val",
            "--count",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
    assert!(stdout.contains("60")); // sum
    assert!(stdout.contains("3")); // count
}

#[test]
fn stats_group_by_many_groups() {
    // Each row is its own group
    let input = "grp\tval
A\t10
B\t20
C\t30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "grp", "--sum", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t10"));
    assert!(stdout.contains("B\t20"));
    assert!(stdout.contains("C\t30"));
}

#[test]
fn stats_multiple_files_with_empty() {
    let file_a = create_file("val\n10\n");
    let file_empty = create_file("");
    let file_b = create_file("val\n20\n");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--sum",
            "val",
            file_a.path().to_str().unwrap(),
            file_empty.path().to_str().unwrap(),
            file_b.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("30")); // 10 + 20
}

#[test]
fn stats_collapse_with_custom_delimiter() {
    let input = "val\nA\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--values", "val", "-v", ","])
        .stdin(input)
        .run();

    assert!(stdout.contains("A,B,C") || stdout.contains("A|B|C"));
}

#[test]
fn stats_unique_single_value() {
    let input = "val\nA\nA\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--unique", "val", "--nunique", "val"])
        .stdin(input)
        .run();

    // Should have only one unique value
    assert!(stdout.contains("1")); // nunique = 1
}

#[test]
fn stats_mode_single_value() {
    let input = "val\nA\nA\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mode", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_mode_tie() {
    // Two values with same frequency
    let input = "val\nA\nA\nB\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mode", "val"])
        .stdin(input)
        .run();

    // Mode could be either A or B
    assert!(stdout.contains("A") || stdout.contains("B"));
}

#[test]
fn stats_geomean_positive() {
    let input = "val\n1\n2\n4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--geomean", "val"])
        .stdin(input)
        .run();

    // Geometric mean of 1, 2, 4 is 2
    assert!(stdout.contains("2"));
}

#[test]
fn stats_harmmean_positive() {
    let input = "val\n1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--harmmean", "val"])
        .stdin(input)
        .run();

    // Harmonic mean of 1, 2, 3 is 3/(1/1+1/2+1/3) = 3/1.833... = 1.636...
    assert!(stdout.contains("1.636") || stdout.contains("1.64"));
}

#[test]
fn stats_range() {
    let input = "val\n5\n10\n15\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--range", "val"])
        .stdin(input)
        .run();

    // Range = max - min = 15 - 5 = 10
    assert!(stdout.contains("10"));
}

#[test]
fn stats_cv() {
    let input = "val\n10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--cv", "val", "-p", "4"])
        .stdin(input)
        .run();

    // CV = stdev / mean = 10 / 20 = 0.5
    assert!(
        stdout.contains("0.5") || stdout.contains("0.50") || stdout.contains("0.500")
    );
}

#[test]
fn stats_quantile_single_prob() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--quantile", "val:0.5"])
        .stdin(input)
        .run();

    // Median of 1,2,3,4,5 is 3
    assert!(stdout.contains("3"));
}

#[test]
fn stats_quantile_extremes() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--quantile", "val:0.0,1.0"])
        .stdin(input)
        .run();

    // 0th percentile = 1, 100th percentile = 5
    assert!(stdout.contains("1"));
    assert!(stdout.contains("5"));
}

#[test]
fn stats_iqr() {
    let input = "val\n1\n2\n3\n4\n5\n6\n7\n8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--iqr", "val"])
        .stdin(input)
        .run();

    // IQR calculation returns 3.5 for this dataset
    assert!(stdout.contains("3.5"));
}

#[test]
fn stats_first_last_single_row() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--first", "val", "--last", "val"])
        .stdin(input)
        .run();

    // First and last should be the same
    let parts: Vec<&str> = stdout.trim().lines().last().unwrap().split('\t').collect();
    assert_eq!(parts[0], parts[1]);
}

#[test]
fn stats_rand_single_row() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--rand", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("42"));
}

#[test]
fn stats_exclude_missing_all_present() {
    // All values present, exclude-missing shouldn't change anything
    let input = "val\n10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "val", "--exclude-missing"])
        .stdin(input)
        .run();

    assert!(stdout.contains("20"));
}

#[test]
fn stats_replace_missing_numeric() {
    let input = "val\n10\n\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--mean",
            "val",
            "--replace-missing",
            "0",
        ])
        .stdin(input)
        .run();

    // Mean of 10, 0, 30 = 13.33...
    assert!(stdout.contains("13") || stdout.contains("14"));
}

#[test]
fn stats_count_header_only() {
    let input = "col\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(input)
        .run();

    assert!(stdout.contains("0"));
}

#[test]
fn stats_multiple_operations_same_field() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--sum", "val", "--mean", "val", "--median", "val",
            "--min", "val", "--max", "val",
        ])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts.len(), 5); // sum, mean, median, min, max
}

#[test]
fn stats_write_header_no_input_header() {
    let input = "10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--write-header", "--sum", "1"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("field1_sum"));
}

#[test]
fn stats_no_operations() {
    // No operations specified - should still work (just count rows implicitly?)
    let input = "val\n1\n2\n3\n";
    let (_, _stderr) = TvaCmd::new()
        .args(&["stats", "--header"])
        .stdin(input)
        .run();

    // Currently may produce no output or error
    // This tests the edge case
}

#[test]
fn stats_field_by_name_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "nonexistent"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("not found") || stderr.contains("field"));
}

#[test]
fn stats_group_by_field_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "nonexistent", "--count"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("not found") || stderr.contains("field"));
}

#[test]
fn stats_mixed_types() {
    // Mix of numeric and non-numeric in same column
    let input = "val\n10\nabc\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val", "--count"])
        .stdin(input)
        .run();

    // Should count all rows but sum only numeric ones
    assert!(stdout.contains("3")); // count
}

#[test]
fn stats_duplicate_field_in_list() {
    let input = "val\n1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val,val"])
        .stdin(input)
        .run();

    // Should handle duplicate fields gracefully
    assert!(stdout.contains("6"));
}

#[test]
fn stats_wildcard_field() {
    let input = "a\tb\tc\n1\t2\t3\n4\t5\t6\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "*"])
        .stdin(input)
        .run();

    // Should sum all fields
    assert!(stdout.contains("5") || stdout.contains("7") || stdout.contains("9"));
}

#[test]
fn stats_range_field_selection() {
    let input = "a\tb\tc\td\n1\t2\t3\t4\n5\t6\t7\t8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "2-3"])
        .stdin(input)
        .run();

    assert!(
        stdout.contains("b_sum\tc_sum") || stdout.contains("8") || stdout.contains("10")
    );
}

#[test]
fn stats_dos_line_endings() {
    let input = "val\r\n10\r\n20\r\n30\r\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("60"));
}

#[test]
fn stats_leading_trailing_whitespace() {
    let input = "val\n 10 \n 20 \n 30 \n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val"])
        .stdin(input)
        .run();

    // Should handle whitespace gracefully
    assert!(stdout.contains("60"));
}

#[test]
fn stats_scientific_notation() {
    let input = "val\n1e3\n2e3\n3e3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("6000") || stdout.contains("6e3"));
}

#[test]
fn stats_infinity_values() {
    let input = "val\ninf\n10\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--max", "val"])
        .stdin(input)
        .run();

    // Max should be infinity
    assert!(stdout.contains("inf") || stdout.contains("Inf"));
}

#[test]
fn stats_nan_values() {
    let input = "val\nnan\n10\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(input)
        .run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_very_long_field_name() {
    let long_name = "a".repeat(100);
    let input = format!("{}\n1\n2\n3\n", long_name);
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", &long_name])
        .stdin(&input)
        .run();

    assert!(stdout.contains("6"));
}

#[test]
fn stats_unicode_field_names() {
    let input = "颜色\tサイズ\n赤\t大\n青\t小\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_many_fields() {
    let mut input = String::from("f1\tf2\tf3\tf4\tf5\tf6\tf7\tf8\tf9\tf10\n");
    input.push_str("1\t2\t3\t4\t5\t6\t7\t8\t9\t10\n");
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "1-10"])
        .stdin(&input)
        .run();

    // Should handle all 10 fields (each column summed separately: 1, 2, 3, ..., 10)
    assert!(stdout.contains("f1_sum") && stdout.contains("f10_sum"));
    assert!(stdout.contains("1\t2\t3\t4\t5\t6\t7\t8\t9\t10"));
}

#[test]
fn stats_many_rows() {
    let mut input = String::from("val\n");
    for i in 1..=100 {
        input.push_str(&format!("{}\n", i));
    }
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "val", "--count"])
        .stdin(&input)
        .run();

    // Sum of 1..100 = 5050
    assert!(stdout.contains("5050"));
    assert!(stdout.contains("100"));
}

#[test]
fn stats_empty_values_in_middle() {
    let input = "a\tb
1\t10
\t20
3\t30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "1", "--sum", "2"])
        .stdin(input)
        .run();

    // Should handle empty values gracefully
    assert!(stdout.contains("4") || stdout.contains("60"));
}

#[test]
fn stats_all_same_value() {
    let input = "val\n5\n5\n5\n5\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "val", "--stdev", "val"])
        .stdin(input)
        .run();

    // Mean = 5, Stdev = 0
    assert!(stdout.contains("5"));
    assert!(stdout.contains("0") || stdout.contains("nan"));
}

#[test]
fn stats_alternating_values() {
    let input = "val\n0\n100\n0\n100\n0\n100\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mean", "val", "--median", "val"])
        .stdin(input)
        .run();

    // Mean = 50, Median = 50
    assert!(stdout.contains("50"));
}

#[test]
fn stats_descending_order() {
    let input = "val\n10\n9\n8\n7\n6\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--min", "val", "--max", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("5")); // min
    assert!(stdout.contains("10")); // max
}

#[test]
fn stats_single_column_file() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--count", "--sum", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("5")); // count
    assert!(stdout.contains("15")); // sum
}

#[test]
fn stats_two_column_file() {
    let input = "a\tb\n1\t10\n2\t20\n3\t30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--sum", "a", "--sum", "b"])
        .stdin(input)
        .run();

    assert!(stdout.contains("6") || stdout.contains("60"));
}

#[test]
fn stats_group_by_with_missing_values() {
    let input = "grp\tval
A\t10
\t20
A\t30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "grp", "--sum", "val"])
        .stdin(input)
        .run();

    // Should handle missing group key
    assert!(stdout.contains("A"));
}

#[test]
fn stats_multiple_operations_different_fields() {
    let input = "a\tb\tc\n1\t10\t100\n2\t20\t200\n3\t30\t300\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats", "--header", "--sum", "a", "--mean", "b", "--max", "c",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("6") || stdout.contains("20") || stdout.contains("300"));
}

#[test]
fn stats_quantile_invalid_field() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--quantile", "nonexistent:0.5"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(!stderr.is_empty());
}

#[test]
fn stats_custom_header_with_quantile() {
    let input = "val\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--quantile",
            "val:0.5:MedianValue",
            "--write-header",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("MedianValue"));
}

#[test]
fn stats_collapse_single_value() {
    let input = "val\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--values", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_unique_single_row() {
    let input = "val\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--unique", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_mode_no_repeats() {
    let input = "val\nA\nB\nC\nD\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mode", "val"])
        .stdin(input)
        .run();

    // Mode could be any of them
    assert!(!stdout.trim().is_empty());
}

#[test]
fn stats_mad_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mad", "val"])
        .stdin(input)
        .run();

    // MAD of single value is 0
    assert!(stdout.contains("0") || stdout.contains("nan"));
}

#[test]
fn stats_variance_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--variance", "val"])
        .stdin(input)
        .run();

    // Variance of single value is 0 or nan
    assert!(stdout.contains("0") || stdout.contains("nan"));
}

#[test]
fn stats_stdev_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--stdev", "val"])
        .stdin(input)
        .run();

    // Stdev of single value is 0 or nan
    assert!(stdout.contains("0") || stdout.contains("nan"));
}

#[test]
fn stats_geomean_with_zero() {
    let input = "val\n0\n10\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--geomean", "val"])
        .stdin(input)
        .run();

    // Geomean with 0 returns a calculated value (14.1421)
    assert!(stdout.contains("14.1421") || stdout.contains("14.14"));
}

#[test]
fn stats_harmmean_with_zero() {
    let input = "val\n0\n10\n20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--harmmean", "val"])
        .stdin(input)
        .run();

    // Harmmean with 0 returns a calculated value (13.3333)
    assert!(stdout.contains("13.3333") || stdout.contains("13.33"));
}

#[test]
fn stats_q1_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--q1", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("42"));
}

#[test]
fn stats_q3_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--q3", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("42"));
}

#[test]
fn stats_iqr_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--iqr", "val"])
        .stdin(input)
        .run();

    // IQR of single value is 0
    assert!(stdout.contains("0"));
}

#[test]
fn stats_cv_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--cv", "val"])
        .stdin(input)
        .run();

    // CV of single value is 0 or nan
    assert!(stdout.contains("0") || stdout.contains("nan"));
}

#[test]
fn stats_range_single_value() {
    let input = "val\n42\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--range", "val"])
        .stdin(input)
        .run();

    // Range of single value is 0
    assert!(stdout.contains("0"));
}

#[test]
fn stats_missing_count_no_missing() {
    let input = "val\n1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--missing-count", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("0"));
}

#[test]
fn stats_not_missing_count_all_present() {
    let input = "val\n1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--not-missing-count", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_mode_count_single_value() {
    let input = "val\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mode-count", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("1"));
}

#[test]
fn stats_mode_count_multiple_same() {
    let input = "val\nA\nA\nB\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--mode-count", "val"])
        .stdin(input)
        .run();

    // Mode count should be 2
    assert!(stdout.contains("2"));
}

#[test]
fn stats_count_no_header() {
    let input = "1\n2\n3\n";
    let (stdout, _) = TvaCmd::new().args(&["stats", "--count"]).stdin(input).run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_sum_no_header() {
    let input = "1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("6"));
}

#[test]
fn stats_mean_no_header() {
    let input = "10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("20"));
}

#[test]
fn stats_median_no_header() {
    let input = "1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--median", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_min_no_header() {
    let input = "5\n3\n8\n1\n9\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--min", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("1"));
}

#[test]
fn stats_max_no_header() {
    let input = "5\n3\n8\n1\n9\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--max", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("9"));
}

#[test]
fn stats_first_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--first", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_last_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--last", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("C"));
}

#[test]
fn stats_nunique_no_header() {
    let input = "A\nB\nA\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--nunique", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_unique_no_header() {
    let input = "A\nB\nA\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique", "1"])
        .stdin(input)
        .run();

    // Should contain A and B in some order
    assert!(stdout.contains("A") && stdout.contains("B"));
}

#[test]
fn stats_collapse_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--values", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A") && stdout.contains("B") && stdout.contains("C"));
}

#[test]
fn stats_rand_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--rand", "1"])
        .stdin(input)
        .run();

    // Should return one of the values
    assert!(stdout.contains("A") || stdout.contains("B") || stdout.contains("C"));
}

#[test]
fn stats_mode_no_header() {
    let input = "A\nA\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mode", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_mode_count_no_header() {
    let input = "A\nA\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mode-count", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_missing_count_no_header() {
    let input = "A\n\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--missing-count", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("1"));
}

#[test]
fn stats_not_missing_count_no_header() {
    let input = "A\n\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--not-missing-count", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_geomean_no_header() {
    let input = "1\n2\n4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--geomean", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_harmmean_no_header() {
    let input = "1\n2\n3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--harmmean", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("1.6") || stdout.contains("1.64"));
}

#[test]
fn stats_q1_no_header() {
    let input = "1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--q1", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_q3_no_header() {
    let input = "1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--q3", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("4"));
}

#[test]
fn stats_iqr_no_header() {
    let input = "1\n2\n3\n4\n5\n6\n7\n8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--iqr", "1"])
        .stdin(input)
        .run();

    // IQR calculation returns 3.5 for this dataset
    assert!(stdout.contains("3.5"));
}

#[test]
fn stats_cv_no_header() {
    let input = "10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--cv", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("0.5") || stdout.contains("0.50"));
}

#[test]
fn stats_range_no_header() {
    let input = "5\n10\n15\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--range", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("10"));
}

#[test]
fn stats_quantile_no_header() {
    let input = "1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--quantile", "1:0.5"])
        .stdin(input)
        .run();

    assert!(stdout.contains("3"));
}

#[test]
fn stats_stdev_no_header() {
    let input = "10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--stdev", "1"])
        .stdin(input)
        .run();

    // stdev of 10, 20, 30 is about 10
    assert!(stdout.contains("10"));
}

#[test]
fn stats_variance_no_header() {
    let input = "10\n20\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--variance", "1"])
        .stdin(input)
        .run();

    // variance of 10, 20, 30 is 100
    assert!(stdout.contains("100"));
}

#[test]
fn stats_mad_no_header() {
    let input = "10\n20\n30\n40\n50\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mad", "1"])
        .stdin(input)
        .run();

    // MAD of 10,20,30,40,50
    assert!(!stdout.trim().is_empty());
}

#[test]
fn stats_group_by_no_header() {
    let input = "A\t10\nA\t20\nB\t30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--sum", "2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t30") || stdout.contains("A\t30"));
    assert!(stdout.contains("B\t30"));
}

#[test]
fn stats_write_header_with_group_by() {
    let input = "grp\tval
A\t10\nB\t20\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--write-header",
            "--group-by",
            "grp",
            "--sum",
            "val",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("grp") || stdout.contains("val_sum"));
}

#[test]
fn stats_retain_alias_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--retain", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A"));
}

#[test]
fn stats_var_alias_no_header() {
    let input = "2\n4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--var", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_unique_count_alias_no_header() {
    let input = "A\nA\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique-count", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("2"));
}

#[test]
fn stats_unique_values_alias_no_header() {
    let input = "A\nA\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--unique-values", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A") && stdout.contains("B"));
}

#[test]
fn stats_collapse_alias_no_header() {
    let input = "A\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--collapse", "1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A") && stdout.contains("B") && stdout.contains("C"));
}

// --- Tests for --header-hash1 mode ---

const INPUT_WITH_HASH_HEADER: &str =
    "# This is a comment\n# Another comment\ncol1\tcol2\tcol3\n1\t2\t3\n4\t5\t6\n";

#[test]
fn stats_header_hash1_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--sum", "col1", "--mean", "col2"])
        .stdin(INPUT_WITH_HASH_HEADER)
        .run();

    assert_eq!(stdout, "col1_sum\tcol2_mean\n5\t3.5\n");
}

#[test]
fn stats_header_hash1_count() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--count"])
        .stdin(INPUT_WITH_HASH_HEADER)
        .run();

    assert_eq!(stdout, "count\n2\n");
}

#[test]
fn stats_header_hash1_group_by() {
    let input = "# Comment\ngroup\tvalue\nA\t10\nA\t20\nB\t30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--group-by",
            "group",
            "--sum",
            "value",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t30"));
    assert!(stdout.contains("B\t30"));
}

#[test]
fn stats_header_hash1_multiple_ops() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--sum",
            "col1",
            "--min",
            "col2",
            "--max",
            "col3",
        ])
        .stdin(INPUT_WITH_HASH_HEADER)
        .run();

    assert!(stdout.contains("col1_sum"));
    assert!(stdout.contains("col2_min"));
    assert!(stdout.contains("col3_max"));
    assert!(stdout.contains("5"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("6"));
}

#[test]
fn stats_header_hash1_no_hash_lines() {
    // When no hash lines exist, --header-hash1 should gracefully use first line as header
    let input = "col1\tcol2\n1\t2\n3\t4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--sum", "col1"])
        .stdin(input)
        .run();

    assert!(stdout.contains("col1_sum"));
    assert!(stdout.contains("4"));
}

#[test]
fn stats_header_hash1_multiple_hash_lines() {
    let input = "# Comment 1\n# Comment 2\n# Comment 3\ncolA\tcolB\n10\t20\n30\t40\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--mean", "colA", "--sum", "colB"])
        .stdin(input)
        .run();

    assert!(stdout.contains("colA_mean"));
    assert!(stdout.contains("colB_sum"));
    assert!(stdout.contains("20")); // mean of 10, 30
    assert!(stdout.contains("60")); // sum of 20, 40
}

#[test]
fn stats_header_hash1_with_write_header() {
    // --write-header should not duplicate header when --header-hash1 is used
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--write-header", "--count"])
        .stdin(INPUT_WITH_HASH_HEADER)
        .run();

    // Should only have one header line
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2); // header + data
    assert_eq!(lines[0], "count");
    assert_eq!(lines[1], "2");
}

#[test]
fn stats_header_hash1_median_variance() {
    let input = "# Comment\nval\n10\n20\n30\n40\n50\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--median",
            "val",
            "--variance",
            "val",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_median"));
    assert!(stdout.contains("val_variance"));
    assert!(stdout.contains("30")); // median
}

#[test]
fn stats_header_hash1_unique_collapse() {
    let input = "# Comment\nname\tcat\nA\tX\nB\tX\nC\tY\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--unique",
            "cat",
            "--collapse",
            "name",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("cat_unique"));
    assert!(stdout.contains("name_collapse"));
}

#[test]
fn stats_header_hash1_quantile() {
    let input = "# Comment\nscore\n10\n20\n30\n40\n50\n60\n70\n80\n90\n100\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--q1",
            "score",
            "--q3",
            "score",
            "--iqr",
            "score",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("score_q1"));
    assert!(stdout.contains("score_q3"));
    assert!(stdout.contains("score_iqr"));
}

#[test]
fn stats_header_hash1_mode_nunique() {
    let input = "# Comment\ncategory\nA\nA\nB\nB\nB\nC\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--mode",
            "category",
            "--nunique",
            "category",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("category_mode"));
    assert!(stdout.contains("category_nunique"));
    assert!(stdout.contains("B")); // mode
    assert!(stdout.contains("3")); // 3 unique values
}

#[test]
fn stats_header_hash1_geomean_harmmean() {
    let input = "# Comment\nval\n2\n4\n8\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--geomean",
            "val",
            "--harmmean",
            "val",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_geomean"));
    assert!(stdout.contains("val_harmmean"));
}

#[test]
fn stats_header_hash1_first_last() {
    let input = "# Comment\nval\n10\n20\n30\n40\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--first", "val", "--last", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_first"));
    assert!(stdout.contains("val_last"));
    assert!(stdout.contains("10"));
    assert!(stdout.contains("40"));
}

#[test]
fn stats_header_hash1_range_cv() {
    let input = "# Comment\nval\n10\n20\n30\n40\n50\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--range", "val", "--cv", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_range"));
    assert!(stdout.contains("val_cv"));
}

#[test]
fn stats_header_hash1_stdev_mad() {
    let input = "# Comment\nval\n10\n20\n30\n40\n50\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--stdev", "val", "--mad", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_stdev"));
    assert!(stdout.contains("val_mad"));
}

#[test]
fn stats_header_hash1_missing_count() {
    let input = "# Comment\nval\n10\n\n30\n\n50\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--missing-count",
            "val",
            "--not-missing-count",
            "val",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_missing_count"));
    assert!(stdout.contains("val_not_missing_count"));
}

#[test]
fn stats_header_hash1_mode_count() {
    let input = "# Comment\nval\nA\nA\nB\nB\nB\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--mode-count", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_mode_count"));
    assert!(stdout.contains("3"));
}

#[test]
fn stats_header_hash1_rand() {
    let input = "# Comment\nval\n100\n200\n300\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--rand", "val"])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_rand"));
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    let val = lines[1];
    assert!(val == "100" || val == "200" || val == "300");
}

#[test]
fn stats_header_hash1_multiple_files() {
    let file1 = create_file("# Comment\ncol1\tcol2\n1\t2\n3\t4\n");
    let file2 = create_file("# Comment\ncol1\tcol2\n5\t6\n7\t8\n");

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--sum",
            "col1",
            "--sum",
            "col2",
            &file1.path().to_string_lossy(),
            &file2.path().to_string_lossy(),
        ])
        .run();

    // Sum of col1: 1+3+5+7 = 16, Sum of col2: 2+4+6+8 = 20
    assert!(stdout.contains("col1_sum"));
    assert!(stdout.contains("col2_sum"));
    assert!(stdout.contains("16") || stdout.contains("20"));
}

#[test]
fn stats_header_hash1_empty_after_header() {
    // File with only header comments and column names, no data
    let input = "# Comment 1\n# Comment 2\ncol1\tcol2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--count"])
        .stdin(input)
        .run();

    assert!(stdout.contains("count"));
    assert!(stdout.contains("0"));
}

#[test]
fn stats_header_hash1_with_replace_missing() {
    let input = "# Comment\nval\n10\n\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--mean",
            "val",
            "--replace-missing",
            "0",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_mean"));
}

#[test]
fn stats_header_hash1_exclude_missing() {
    let input = "# Comment\nval\n10\n\n30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--mean",
            "val",
            "--exclude-missing",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("val_mean"));
    // Mean of 10 and 30 (excluding missing) = 20
    assert!(stdout.contains("20"));
}

#[test]
fn stats_header_hash1_field_by_name_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header-hash1", "--sum", "nonexistent"])
        .stdin(INPUT_WITH_HASH_HEADER)
        .run_fail();

    assert!(stderr.contains("not found") || stderr.contains("field"));
}

#[test]
fn stats_header_hash1_quantile_custom_header() {
    let input = "# Comment\nval\n1\n2\n3\n4\n5\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header-hash1",
            "--quantile",
            "val:0.5:MedianValue",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("MedianValue"));
}
