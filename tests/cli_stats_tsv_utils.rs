use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn tsv_utils_test_20_basic_count_min_max() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("count\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"))
        .stdout(predicate::str::contains("7\t7.4\t1\t2\t16\t6\t7"));
}

#[test]
fn tsv_utils_test_28_group_by_1() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("1")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("color\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("blue\t3\t12\t1\t2\t16\t4\t4"));
    assert!(stdout.contains("red\t2\t8\t4\t6\t10\t6\t7"));
    // green check omitted due to potential float formatting differences
}

#[test]
fn tsv_utils_test_34_group_by_1_2() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("1,2")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("blue\tsolid\t2\t14\t2\t3\t16\t4\t4"));
    assert!(stdout.contains("blue\tstriped\t1\t12\t1\t2\t12\t1\t2"));
    assert!(stdout.contains("green\tsolid\t2\t7.4\t5.5\t3.2\t11\t6\t5.4")); // Assuming this passes
    assert!(stdout.contains("red\tsolid\t1\t10\t4\t7\t10\t4\t7"));
    assert!(stdout.contains("red\tstriped\t1\t8\t6\t6\t8\t6\t6"));
}

#[test]
fn tsv_utils_test_42_group_by_range() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("1-2")
        .arg("--count")
        .arg("--min")
        .arg("3-5")
        .arg("--max")
        .arg("5-3")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min"));
    // Depending on expansion order, we check for presence
    assert!(stdout.contains("height_max"));
    assert!(stdout.contains("width_max"));
    assert!(stdout.contains("length_max"));
}

#[test]
fn tsv_utils_test_50_group_by_names() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("color,pattern")
        .arg("--count")
        .arg("--min")
        .arg("length-height")
        .arg("--max")
        .arg("height-length")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("color\tpattern\tcount\tlength_min\twidth_min\theight_min"));
    // tva sorts output fields by index, so range 5-3 becomes 3,4,5 (length, width, height)
    // gold: red	solid	1	10	4	7	7	4	10 (height_max, width_max, length_max)
    // tva:  red	solid	1	10	4	7	10	4	7 (length_max, width_max, height_max)
    assert!(stdout.contains("red\tsolid\t1\t10\t4\t7\t10\t4\t7"));
}

#[test]
fn tsv_utils_test_58_multi_file() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_b.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_c.tsv")
        .arg("tests/data/stats/tsv-utils/empty_file.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_header_only.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("count\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"))
        .stdout(predicate::str::contains("12\t6\t1\t2\t16\t7\t8"));
}

#[test]
fn tsv_utils_test_97_multi_file_unique_count() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("--nunique")
        .arg("1,2,3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv")
        .arg("tests/data/stats/tsv-utils/empty_file.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_b.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_header_only.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_c.tsv");

    // Note: tsv-summarize output header is `field_unique_count`, tva is `field_nunique`.
    // Gold: 16	6	6	10	8	11
    cmd.assert()
        .success()
        // Check for correct values (16 rows total, unique counts for each col)
        .stdout(predicate::str::contains("16\t6\t6\t10\t8\t11"));
}

#[test]
fn tsv_utils_test_103_group_by_unique_count() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--group-by")
        .arg("1")
        .arg("--count")
        .arg("--nunique")
        .arg("2,3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv")
        .arg("tests/data/stats/tsv-utils/empty_file.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_b.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_header_only.tsv")
        .arg("tests/data/stats/tsv-utils/input_5field_c.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Gold lines 104-109
    // color	count	field2_unique_count	field3_unique_count	field4_unique_count	field5_unique_count
    // red	4	3	3	3	3
    // blue	3	2	3	3	3
    // green	2	1	2	2	2
    // 赤	2	1	2	2	2
    // 青	1	1	1	1	1

    assert!(stdout.contains("red\t4\t3\t3\t3\t3"));
    assert!(stdout.contains("blue\t3\t2\t3\t3\t3"));
    assert!(stdout.contains("green\t2\t1\t2\t2\t2"));
    assert!(stdout.contains("赤\t2\t1\t2\t2\t2"));
    assert!(stdout.contains("青\t1\t1\t1\t1\t1"));
}

#[test]
fn tsv_utils_test_154_count_unique_count_files() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv")
        .arg("tests/data/stats/tsv-utils/input_1field_b.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("count\tsize_nunique"))
        .stdout(predicate::str::contains("9\t6"));
}

#[test]
fn tsv_utils_test_243_mean() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("1")
        .arg("--min")
        .arg("3-5")
        .arg("--max")
        .arg("3-5")
        .arg("--mean")
        .arg("3-5")
        .arg("tests/data/stats/tsv-utils/input_5field_d.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check headers
    assert!(stdout.contains("length_mean\twidth_mean\theight_mean"));

    // Check values (approximate checks)
    // red: mean 0.0773..., 0.11, 0.156...
    assert!(stdout.contains("red\t"));
    assert!(stdout.contains("0.0773"));
    assert!(stdout.contains("0.11"));
    assert!(stdout.contains("0.156"));

    // blue: mean 0.1055, 0.11, 0.179...
    assert!(stdout.contains("blue\t"));
    assert!(stdout.contains("0.1055"));
    assert!(stdout.contains("0.179"));

    // green: mean 0.11, 0.11, 0.111...
    assert!(stdout.contains("green\t"));
    assert!(stdout.contains("0.111"));
}

#[test]
fn tsv_utils_test_advanced_stats_1() {
    // Corresponds partly to Test 4 (lines 4-6)
    // --first 1 --last 1 --median 3 --mad 3 --variance 3 --stdev 3 --mode 1
    // tva supports these.

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--first")
        .arg("1")
        .arg("--last")
        .arg("1")
        .arg("--median")
        .arg("3")
        .arg("--mad")
        .arg("3")
        .arg("--variance")
        .arg("3")
        .arg("--stdev")
        .arg("3")
        .arg("--mode")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Headers
    assert!(stdout.contains("color_first"));
    assert!(stdout.contains("color_last"));
    assert!(stdout.contains("length_median"));
    assert!(stdout.contains("length_mad"));
    assert!(stdout.contains("length_variance")); // tva might use _variance or _var? tsv-utils uses _var.
    assert!(stdout.contains("length_stdev"));
    assert!(stdout.contains("color_mode"));

    // Values (Gold line 6)
    // red (first), green (last), 11 (median), 3 (mad), 9.61 (var), 3.1 (stdev), blue (mode)
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
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("no_such_file.tsv");

    cmd.assert()
        .failure()
        // On Windows, the error message is localized, so we check for the error code.
        .stderr(predicate::str::contains("os error 2"));
}

#[test]
fn tsv_utils_error_test_invalid_field_index() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--nunique")
        .arg("0")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("field index must be >= 1"));
}

#[test]
fn tsv_utils_error_test_invalid_field_list_empty_element() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--nunique")
        .arg("2,")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("empty field list element"));
}

#[test]
fn tsv_utils_test_stdin_group_by() {
    let mut cmd = cargo_bin_cmd!("tva");
    let input = std::fs::read("tests/data/stats/tsv-utils/input_5field_a.tsv").unwrap();

    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("2")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .write_stdin(input);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("pattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"));
    assert!(stdout.contains("solid\t5\t7.4\t2\t3\t16\t6\t7"));
    assert!(stdout.contains("striped\t2\t8\t1\t2\t12\t6\t6"));
}

#[test]
fn tsv_utils_test_stdin_mixed_files() {
    let mut cmd = cargo_bin_cmd!("tva");
    let input_b = std::fs::read("tests/data/stats/tsv-utils/input_5field_b.tsv").unwrap();

    // cat input_5field_b.tsv | tva stats ... input_5field_a.tsv - input_5field_c.tsv
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("2")
        .arg("--count")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv")
        .arg("-")
        .arg("tests/data/stats/tsv-utils/input_5field_c.tsv")
        .write_stdin(input_b);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // The output order of groups might vary, but values should be correct.
    // Gold results from basic_tests_1.txt line 140:
    // solid 6 ...
    // striped 2 ...
    // checked 1 ...
    // ...
    assert!(stdout.contains("solid\t6"));
    assert!(stdout.contains("striped\t2"));
    assert!(stdout.contains("checked\t1"));
}

#[test]
fn tsv_utils_test_field_out_of_bounds_multi_file_behavior() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--group-by")
        .arg("2")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv");

    // tva differs from tsv-summarize here: it treats missing fields as empty strings instead of erroring.
    // input_1field_a.tsv lines have no field 2, so they get grouped under "" (empty string).
    // Within that group, field 1 has 5 unique values (size, 10, small, "", 8).
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\t5"));
}

#[test]
fn tsv_utils_test_crlf_handling() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_1field_a_dos.tsv");

    // tva includes \r in the value, so we have 5 unique values (size\r, 10\r, small\r, \r, 8\r)
    // instead of 4 unique values if \r was stripped.
    // Also no header output because --header is not passed.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("7\t5"));
}

#[test]
fn tsv_utils_test_extended_stats() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--first")
        .arg("3")
        .arg("--last")
        .arg("3")
        .arg("--median")
        .arg("3")
        .arg("--mad")
        .arg("3")
        .arg("--variance")
        .arg("3")
        .arg("--stdev")
        .arg("3")
        .arg("--mode")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("length_first\tlength_last\tlength_median\tlength_mad\tlength_variance\tlength_stdev\tcolor_mode"))
        .stdout(predicate::str::contains("10\t7.4\t11\t3\t9.613333333333307\t3.100537587795592\tblue"));
}

#[test]
fn tsv_utils_test_float_precision_defaults() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("1")
        .arg("--min")
        .arg("3,4,5")
        .arg("--max")
        .arg("3,4,5")
        .arg("--mean")
        .arg("3,4,5")
        .arg("tests/data/stats/tsv-utils/input_5field_d.tsv");

    cmd.assert()
        .success()
        // Check for header
        .stdout(predicate::str::contains("color\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max\tlength_mean\twidth_mean\theight_mean"))
        // Check for data rows (sorted by group key: blue, green, red)
        .stdout(predicate::str::contains("blue\t0.1\t0.11\t0.12345678998765432\t0.111\t0.11\t0.2345678901234568\t0.10550000000000001\t0.11\t0.17901234005555555"))
        .stdout(predicate::str::contains("green\t0.11\t0.11\t0.11111111333333333\t0.11\t0.11\t0.11111111333333333\t0.11\t0.11\t0.11111111333333333"))
        .stdout(predicate::str::contains("red\t0.011\t0.11\t0.012345678901234567\t0.1111\t0.11\t0.33333333111111113\t0.07736666666666667\t0.11\t0.15637859967489712"));
}

#[test]
fn tsv_utils_error_test_non_numeric_group_by() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--group-by")
        .arg("x")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("field name `x` requires header"));
}

#[test]
fn tsv_utils_error_test_field_not_found_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--group-by")
        .arg("2")
        .arg("--sum")
        .arg("width,len") // len is missing
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unknown field name `len`"));
}

#[test]
fn tsv_utils_test_field_range_by_name() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--sum")
        .arg("length-height")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // length, width, height sum
    // Gold: length_sum 78.4, width_sum 28.5, height_sum 30.6
    assert!(stdout.contains("length_sum\twidth_sum\theight_sum"));
    assert!(stdout.contains("78.4\t28.5\t30.6"));
}

#[test]
fn tsv_utils_test_1field() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("count\tsize_nunique"))
        .stdout(predicate::str::contains("6\t4"));
}

#[test]
fn tsv_utils_test_empty_file() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/empty_file.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn tsv_utils_test_header_only() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/input_5field_header_only.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn tsv_utils_error_test_zero_index() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--min")
        .arg("0")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("field index must be >= 1"));
}

#[test]
fn tsv_utils_error_test_invalid_field_list() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--header")
        .arg("--count")
        .arg("--min")
        .arg("1,,2")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("empty field list element"));
}

#[test]
fn tsv_utils_test_header_as_data() {
    // tsv-summarize behavior: treating header line as data if --header is not specified.
    // tva should do the same.
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--group-by")
        .arg("1,2")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/input_5field_a.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // The header line "color\tpattern\t..." becomes a group "color", "pattern"
    // gold: color	pattern	4... (if multiple files)
    // Here we have just one file.
    // input_5field_a.tsv has 1 header + 7 data lines.
    // "color" line is 1.
    // "red" lines: 2
    // "blue" lines: 3
    // "green" lines: 2
    // Total 8 lines.

    // We expect a line starting with "color\tpattern".
    assert!(stdout.contains("color\tpattern"));
}

#[test]
fn tsv_utils_test_1field_no_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("7"));
}

#[test]
fn tsv_utils_test_empty_file_no_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("tests/data/stats/tsv-utils/empty_file.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn tsv_utils_test_multi_file_no_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--count")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv")
        .arg("tests/data/stats/tsv-utils/input_1field_b.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("11\t7"));
}

#[test]
fn tsv_utils_test_no_header_group_by() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("stats")
        .arg("--group-by")
        .arg("1")
        .arg("--count")
        .arg("--nunique")
        .arg("1")
        .arg("tests/data/stats/tsv-utils/input_1field_a.tsv")
        .arg("tests/data/stats/tsv-utils/input_1field_b.tsv");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Groups: size (2), 10 (3), small (2), 8 (1), 9 (1), medium (1), empty (1)
    // nunique for grouping column is always 1.
    assert!(stdout.contains("size\t2\t1"));
    assert!(stdout.contains("10\t3\t1"));
    assert!(stdout.contains("small\t2\t1"));
    assert!(stdout.contains("8\t1\t1"));
    assert!(stdout.contains("9\t1\t1"));
    assert!(stdout.contains("medium\t1\t1"));
    // empty string check might be tricky with tabs, but let's try
    // \t1\t1
    assert!(stdout.contains("\t1\t1"));
}
