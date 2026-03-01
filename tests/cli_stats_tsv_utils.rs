#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

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

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

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

    assert!(stdout.contains("length_mean\twidth_mean\theight_mean"));
    assert!(stdout.contains("red\t"));
    assert!(stdout.contains("0.0773"));
    assert!(stdout.contains("0.11"));
    assert!(stdout.contains("0.156"));
    assert!(stdout.contains("blue\t"));
    assert!(stdout.contains("0.1055"));
    assert!(stdout.contains("0.179"));
    assert!(stdout.contains("green\t"));
    assert!(stdout.contains("0.111"));
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

    assert!(stdout.contains("color\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max\tlength_mean\twidth_mean\theight_mean"));
    assert!(stdout.contains("blue\t0.1\t0.11\t0.12345678998765432\t0.111\t0.11\t0.2345678901234568\t0.10550000000000001\t0.11\t0.17901234005555555"));
    assert!(stdout.contains("green\t0.11\t0.11\t0.11111111333333333\t0.11\t0.11\t0.11111111333333333\t0.11\t0.11\t0.11111111333333333"));
    assert!(stdout.contains("red\t0.011\t0.11\t0.012345678901234567\t0.1111\t0.11\t0.33333333111111113\t0.07736666666666667\t0.11\t0.15637859967489712"));
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
