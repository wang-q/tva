#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;
use test_case::test_case;

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

#[test_case(
    "ID\tname\tvalue\nA\tcost\t10\nA\tsize\t5\nB\tcost\t20\nB\tsize\t8",
    "--names-from name --values-from value",
    "ID\tcost\tsize\nA\t10\t5\nB\t20\t8";
    "basic"
)]
#[test_case(
    "A\tB\tkey\tval\n1\tx\tk1\t10\n1\tx\tk2\t20\n2\ty\tk1\t30",
    "--names-from key --values-from val",
    "A\tB\tk1\tk2\n1\tx\t10\t20\n2\ty\t30\t";
    "implicit_id_multi_col"
)]
#[test_case(
    "ID\tkey\tval\n1\tb\t2\n1\ta\t1\n1\tc\t3",
    "--names-from key --values-from val --names-sort",
    "ID\ta\tb\tc\n1\t1\t2\t3";
    "names_sort"
)]
#[test_case(
    "ID\tkey\tval\n1\ta\t1\n2\tb\t2",
    "--names-from key --values-from val --values-fill missing --names-sort",
    "ID\ta\tb\n1\t1\tmissing\n2\tmissing\t2";
    "custom_fill_string"
)]
fn test_wider_basic(input: &str, args: &str, expected: &str) {
    let args: Vec<&str> = std::iter::once("wider")
        .chain(args.split_whitespace())
        .collect();
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert_eq!(stdout.trim(), expected.trim());
}

#[test_case(
    "ID\tname\tvalue\nA\tcost\t10\nB\tsize\t8",
    "--names-from name --values-from value --values-fill 0 --names-sort",
    "ID\tcost\tsize\nA\t10\t0\nB\t0\t8";
    "missing_values"
)]
#[test_case(
    "ID\tname\tvalue\nA\tcost\t10\nA\tcost\t12",
    "--names-from name --values-from value --id-cols ID",
    "ID\tcost\nA\t12";
    "explicit_id"
)]
fn test_wider_missing_and_id(input: &str, args: &str, expected: &str) {
    let args: Vec<&str> = std::iter::once("wider")
        .chain(args.split_whitespace())
        .collect();
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_doc_example_us_rent_income() {
    let expected = "GEOID\tNAME\tincome\trent\n01\tAlabama\t24476\t747\n02\tAlaska\t32940\t1200\n04\tArizona\t27517\t972\n05\tArkansas\t23789\t709\n06\tCalifornia\t29454\t1358";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "docs/data/us_rent_income.tsv",
            "--names-from",
            "variable",
            "--values-from",
            "estimate",
            "--id-cols",
            "GEOID,NAME",
        ])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_multi_file_error() {
    let file1 = create_file("ID\tname\tvalue\nA\tcost\t10\n");
    let file2 = create_file("ID\tvalue\nB\t20\n");

    let (_, stderr) = TvaCmd::new()
        .args(&[
            "wider",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            "--names-from",
            "name",
            "--values-from",
            "value",
        ])
        .run_fail();

    assert!(stderr.contains("All files must have the same column structure"));
}

#[test]
fn wider_preserve_space() {
    let input = "ID\tname\tvalue\nA\tcost\t ";
    let expected = "ID\tcost\nA\t ";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_datamash_scenarios() {
    // Scenario 1
    let input1 = "ID\tKey\tVal\na\tx\t1\na\ty\t2\na\tx\t3";
    let expected1 = "ID\tx\ty\na\t3\t2";
    let (stdout1, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input1)
        .run();

    assert_eq!(stdout1.trim(), expected1);

    // Scenario 2
    let input2 = "ID\tKey\tVal\na\tx\t1\na\ty\t2\nb\tx\t3";
    let expected2 = "ID\tx\ty\na\t1\t2\nb\t3\tXX";
    let (stdout2, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--values-fill",
            "XX",
        ])
        .stdin(input2)
        .run();

    assert_eq!(stdout2.trim(), expected2);
}

#[test_case(
    "sum",
    "ID\tX\tY\tZ\nA\t30\t\t\nB\t\t20\t\nC\t\t\t100";
    "aggregation_sum"
)]
#[test_case(
    "mean",
    "ID\tX\tY\tZ\nA\t15\t\t\nB\t\t10\t\nC\t\t\t100";
    "aggregation_mean"
)]
fn test_wider_aggregation_ops(op: &str, expected: &str) {
    let input = "ID\tname\tval\nA\tX\t10\nA\tX\t20\nB\tY\t5\nB\tY\t15\nC\tZ\t100";
    let args: Vec<&str> = if op == "count" {
        vec![
            "wider",
            "--names-from",
            "name",
            "--id-cols",
            "ID",
            "--op",
            op,
        ]
    } else {
        vec![
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "val",
            "--id-cols",
            "ID",
            "--op",
            op,
        ]
    };
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert_eq!(stdout.trim(), expected.trim());
}

#[test_case(
    "ID\tname\tval\nA\tX\t10\nA\tX\t20\nB\tY\t5\nB\tY\t15\nC\tZ\t100",
    "count",
    "ID\tX\tY\tZ\nA\t2\t\t\nB\t\t2\t\nC\t\t\t1";
    "aggregation_count"
)]
fn test_wider_count_op(input: &str, op: &str, expected: &str) {
    let args: Vec<&str> = vec![
        "wider",
        "--names-from",
        "name",
        "--id-cols",
        "ID",
        "--op",
        op,
    ];
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert_eq!(stdout.trim(), expected.trim());
}

#[test_case("min", "A\t1", "B\t2"; "extended_min")]
#[test_case("max", "A\t5", "B\t8"; "extended_max")]
#[test_case("median", "A\t3", "B\t2"; "extended_median")]
#[test_case("mode", "A\t1", "B\t2"; "extended_mode")]
fn test_wider_extended_stats(op: &str, expected_a: &str, expected_b: &str) {
    let input = "ID\tKey\tVal\nA\tX\t1\nA\tX\t3\nA\tX\t5\nB\tX\t2\nB\tX\t2\nB\tX\t8";
    let args: Vec<&str> = vec![
        "wider",
        "--names-from",
        "Key",
        "--values-from",
        "Val",
        "--id-cols",
        "ID",
        "--op",
        op,
    ];
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert!(
        stdout.contains(expected_a),
        "Expected '{}' in output: {}",
        expected_a,
        stdout
    );
    assert!(
        stdout.contains(expected_b),
        "Expected '{}' in output: {}",
        expected_b,
        stdout
    );
}

#[test_case("first", "A\tfirst_val"; "first")]
#[test_case("last", "A\tlast_val"; "last")]
fn test_wider_first_last(op: &str, expected: &str) {
    let input = "ID\tKey\tVal\nA\tX\tfirst_val\nA\tX\tmiddle_val\nA\tX\tlast_val";
    let args: Vec<&str> = vec![
        "wider",
        "--names-from",
        "Key",
        "--values-from",
        "Val",
        "--id-cols",
        "ID",
        "--op",
        op,
    ];
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in output: {}",
        expected,
        stdout
    );
}

#[test_case("q1", "A\t2"; "quartiles_q1")]
#[test_case("q3", "A\t4"; "quartiles_q3")]
#[test_case("iqr", "A\t2"; "quartiles_iqr")]
fn test_wider_quartiles(op: &str, expected: &str) {
    let input = "ID\tKey\tVal\nA\tX\t1\nA\tX\t2\nA\tX\t3\nA\tX\t4\nA\tX\t5";
    let args: Vec<&str> = vec![
        "wider",
        "--names-from",
        "Key",
        "--values-from",
        "Val",
        "--id-cols",
        "ID",
        "--op",
        op,
    ];
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in output: {}",
        expected,
        stdout
    );
}

#[test_case("geomean", "A\t4"; "advanced_geomean")]
#[test_case("harmmean", "A\t3.2"; "advanced_harmmean")]
#[test_case("variance", "A\t18"; "advanced_variance")]
#[test_case("stdev", "A\t4.242"; "advanced_stdev")]
#[test_case("cv", "A\t0.848"; "advanced_cv")]
fn test_wider_advanced_math(op: &str, expected: &str) {
    let input = "ID\tKey\tVal\nA\tX\t2\nA\tX\t8";
    let args: Vec<&str> = vec![
        "wider",
        "--names-from",
        "Key",
        "--values-from",
        "Val",
        "--id-cols",
        "ID",
        "--op",
        op,
    ];
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input.trim()).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in output: {}",
        expected,
        stdout
    );
}

#[test]
fn wider_missing_values_from_error() {
    // Test --values-from required for non-count operations (covers L146-147)
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", "--names-from", "key", "--op", "sum"])
        .stdin("ID\tkey\tval\nA\tk1\t10\n")
        .run_fail();

    assert!(stderr.contains("--values-from is required"));
}

#[test]
fn wider_empty_file() {
    // Test empty file handling (covers L180-181)
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "key", "--values-from", "val"])
        .stdin("")
        .run();

    // Empty input produces empty output (or just newline)
    assert!(stdout.is_empty() || stdout == "\n");
}

#[test]
fn wider_count_no_values_from() {
    // Test count operation doesn't require --values-from (covers L227-228)
    // Note: without --id-cols, code uses default ID columns logic (all except names)
    let input = "ID\tname\nA\tX\nA\tX\nB\tY";
    let expected = "ID\tX\tY\nA\t2\t0\nB\t0\t1";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--op",
            "count",
            "--values-fill",
            "0",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_multi_column_names_from_error() {
    // Test multi-column --names-from error (covers L196-199)
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", "--names-from", "1,2", "--values-from", "3"])
        .stdin("A\tB\tC\n1\t2\t3")
        .run_fail();

    assert!(stderr.contains("only single column supported for --names-from"));
}

#[test]
fn wider_multi_column_values_from_error() {
    // Test multi-column --values-from error (covers L207-210)
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", "--names-from", "1", "--values-from", "2,3"])
        .stdin("A\tB\tC\n1\t2\t3")
        .run_fail();

    assert!(stderr.contains("only single column supported for --values-from"));
}

#[test]
fn wider_header_flag() {
    // Test --header flag (FirstLine mode)
    let input = "ID\tKey\tVal\nA\tX\t1\nA\tY\t2";
    let expected = "ID\tX\tY\nA\t1\t2";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--header",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_header_hash1() {
    // Test --header-hash1 flag (HashLines1 mode)
    let input = "# Comment line\nID\tKey\tVal\nA\tX\t1\nA\tY\t2";
    let expected = "ID\tX\tY\nA\t1\t2";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--header-hash1",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_header_hash1_no_hash_lines() {
    // Test --header-hash1 graceful degradation when no hash lines exist
    let input = "ID\tKey\tVal\nA\tX\t1\nA\tY\t2";
    let expected = "ID\tX\tY\nA\t1\t2";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--header-hash1",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_header_hash1_multiple_comments() {
    // Test --header-hash1 with multiple comment lines
    let input = "# First comment\n# Second comment\nID\tKey\tVal\nA\tX\t1";
    let expected = "ID\tX\nA\t1";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--header-hash1",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_no_header_numeric_fields() {
    // Test that without explicit header flag, first line is treated as header
    // This is the default behavior for backward compatibility
    let input = "A\tX\t1\nA\tY\t2";
    // First line "A	X	1" is treated as header
    // Column 1 = "A", Column 2 = "X", Column 3 = "1"
    // names-from=2 -> "X", values-from=3 -> "1", id-cols=1 -> "A"
    // Second line "A	Y	2" is data: ID="A", Name="Y", Value="2"
    let expected = "A\tY\nA\t2";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "2",
            "--values-from",
            "3",
            "--id-cols",
            "1",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_help_text_clarity() {
    // Verify help text mentions single column limitation
    let (stdout, _) = TvaCmd::new().args(&["wider", "--help"]).run();

    // Help should mention single column limitation
    assert!(stdout.contains("single column"));
}

#[test]
fn wider_range_op() {
    // Test range operation (max - min)
    let input = "ID\tKey\tVal\nA\tX\t5\nA\tX\t10\nA\tX\t15";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "range",
        ])
        .stdin(input)
        .run();

    // Range = 15 - 5 = 10
    assert!(stdout.contains("A\t10"));
}

#[test]
fn wider_single_row_single_column() {
    // Test with minimal data - single row
    let input = "ID\tname\tvalue\nA\tcost\t100";
    let expected = "ID\tcost\nA\t100";

    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_duplicate_id_name_combinations() {
    // Test aggregation when same ID+Name combination appears multiple times
    // Default op is "last", so last value should win
    let input = "ID\tname\tvalue\nA\tcost\t10\nA\tcost\t20\nA\tcost\t30";
    let expected = "ID\tcost\nA\t30";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_numeric_column_indices() {
    // Test using numeric column indices instead of names
    let input = "col1\tcol2\tcol3\nA\tX\t1\nA\tY\t2\nB\tX\t3";
    let expected = "col1\tX\tY\nA\t1\t2\nB\t3\t";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "2", // col2
            "--values-from",
            "3", // col3
            "--id-cols",
            "1", // col1
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_empty_values_with_fill() {
    // Test handling of missing columns with custom fill
    // Note: --values-fill is used when a column is missing for an ID,
    // not when the value is empty
    let input = "ID\tname\tvalue\nA\tx\t5\nA\ty\t10\nB\tx\t20";
    // B doesn't have 'y', so it should be filled with "NA"
    let expected = "ID\tx\ty\nA\t5\t10\nB\t20\tNA";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
            "--values-fill",
            "NA",
            "--names-sort",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_multiple_id_columns() {
    // Test with multiple ID columns
    let input = "Year\tMonth\tType\tValue\n2024\tJan\tA\t100\n2024\tJan\tB\t200\n2024\tFeb\tA\t150";
    let expected = "Year\tMonth\tA\tB\n2024\tJan\t100\t200\n2024\tFeb\t150";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Type",
            "--values-from",
            "Value",
            "--id-cols",
            "Year,Month",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_special_characters_in_names() {
    // Test handling of special characters in name column
    let input = "ID\tname\tvalue\nA\tX-Y\t1\nA\tX+Y\t2\nB\tX*Y\t3";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
            "--names-sort",
        ])
        .stdin(input)
        .run();

    // Should contain all special column names
    assert!(stdout.contains("X-Y"));
    assert!(stdout.contains("X+Y"));
    assert!(stdout.contains("X*Y"));
}

#[test]
fn wider_unicode_content() {
    // Test with unicode characters in data
    let input = "ID\tname\tvalue\n中文\t成本\t100\n中文\t尺寸\t50";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    // Should handle unicode correctly
    assert!(stdout.contains("中文"));
    assert!(stdout.contains("成本"));
    assert!(stdout.contains("尺寸"));
    assert!(stdout.contains("100"));
    assert!(stdout.contains("50"));
}

#[test]
fn wider_large_numbers() {
    // Test with large numbers
    let input = "ID\tname\tvalue\nA\tsales\t999999999.99\nB\tsales\t1000000000.00";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("999999999.99"));
    assert!(stdout.contains("1000000000.00"));
}

#[test]
fn wider_negative_numbers() {
    // Test with negative numbers
    let input = "ID\tname\tvalue\nA\tprofit\t-100\nA\tloss\t50";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("-100"));
    assert!(stdout.contains("50"));
}

#[test]
fn wider_float_precision() {
    // Test float precision handling
    let input = "ID\tname\tvalue\nA\tpi\t3.14159265359";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    assert!(stdout.contains("3.14159265359"));
}

#[test]
fn wider_only_header_no_data() {
    // Test with only header row, no data
    let input = "ID\tname\tvalue";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    // Should output just the header
    assert!(stdout.contains("ID"));
}

#[test]
fn wider_mixed_types_in_value_column() {
    // Test handling of mixed types (numbers and strings) in value column
    let input = "ID\tname\tvalue\nA\tnum\t100\nA\tstr\thello";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    // Should handle both numeric and string values
    assert!(stdout.contains("100"));
    assert!(stdout.contains("hello"));
}

#[test]
fn wider_id_cols_with_spaces() {
    // Test ID columns that contain spaces (should be preserved)
    let input = "ID\tname\tvalue\nA B\tx\t1\nA B\ty\t2";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--id-cols",
            "ID",
        ])
        .stdin(input)
        .run();

    // ID with space should be preserved
    assert!(stdout.contains("A B"));
}
