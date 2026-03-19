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

// --- Common Test Inputs ---

/// Basic three-column input: ID, name, value
const INPUT_BASIC_3COL: &str =
    "ID\tname\tvalue\nA\tcost\t10\nA\tsize\t5\nB\tcost\t20\nB\tsize\t8";

/// Multi-column ID input
const INPUT_MULTI_ID: &str =
    "Year\tMonth\tType\tValue\n2024\tJan\tA\t100\n2024\tJan\tB\t200\n2024\tFeb\tA\t150";

/// Single row input
const INPUT_SINGLE_ROW: &str = "ID\tname\tvalue\nA\tcost\t100";

/// Duplicate ID-name combinations
const INPUT_DUPLICATE_ID: &str =
    "ID\tname\tvalue\nA\tcost\t10\nA\tcost\t20\nA\tcost\t30";

/// Input with missing values
const INPUT_MISSING_VAL: &str = "ID\tname\tvalue\nA\tx\t5\nA\ty\t10\nB\tx\t20";

/// Header-only input
const INPUT_HEADER_ONLY: &str = "ID\tname\tvalue";

/// Empty input
const INPUT_EMPTY: &str = "";

/// Key-Value header input (for header mode tests)
const INPUT_KEY_VAL_HEADER: &str = "ID\tKey\tVal\nA\tX\t1\nA\tY\t2";

/// Key-Value with comment header
const INPUT_KEY_VAL_COMMENT: &str = "# Comment line\nID\tKey\tVal\nA\tX\t1\nA\tY\t2";

/// Key-Value with multiple comment lines
const INPUT_KEY_VAL_MULTI_COMMENT: &str =
    "# First comment\n# Second comment\nID\tKey\tVal\nA\tX\t1";

#[test_case(
    INPUT_BASIC_3COL,
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
    let file1 = create_file(INPUT_BASIC_3COL);
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
        .stdin(INPUT_EMPTY)
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

#[test_case(
    "--names-from", "1,2", "--values-from", "3",
    "only single column supported for --names-from";
    "multi_column_names_from"
)]
#[test_case(
    "--names-from", "1", "--values-from", "2,3",
    "only single column supported for --values-from";
    "multi_column_values_from"
)]
fn test_wider_multi_column_errors(
    names_flag: &str,
    names_val: &str,
    values_flag: &str,
    values_val: &str,
    expected_err: &str,
) {
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", names_flag, names_val, values_flag, values_val])
        .stdin("A\tB\tC\n1\t2\t3")
        .run_fail();

    assert!(stderr.contains(expected_err));
}

#[test_case(
    "--header",
    INPUT_KEY_VAL_HEADER,
    "ID\tX\tY\nA\t1\t2";
    "header_flag"
)]
#[test_case(
    "--header-hash1",
    INPUT_KEY_VAL_COMMENT,
    "ID\tX\tY\nA\t1\t2";
    "header_hash1"
)]
#[test_case(
    "--header-hash1",
    INPUT_KEY_VAL_HEADER,
    "ID\tX\tY\nA\t1\t2";
    "header_hash1_no_hash"
)]
#[test_case(
    "--header-hash1",
    INPUT_KEY_VAL_MULTI_COMMENT,
    "ID\tX\nA\t1";
    "header_hash1_multi_comments"
)]
fn test_wider_header_modes(header_flag: &str, input: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            header_flag,
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
fn wider_help_text_clarity() {
    // Verify help text mentions single column limitation
    let (stdout, _) = TvaCmd::new().args(&["wider", "--help"]).run();

    // Help should mention single column limitation
    assert!(stdout.contains("single column"));
}

#[test_case(
    "A\tX\t1\nA\tY\t2",
    "2", "3", "1",
    "A\tY\nA\t2";
    "no_header_numeric_fields"
)]
#[test_case(
    "col1\tcol2\tcol3\nA\tX\t1\nA\tY\t2\nB\tX\t3",
    "2", "3", "1",
    "col1\tX\tY\nA\t1\t2\nB\t3\t";
    "numeric_column_indices"
)]
fn test_wider_numeric_indices(
    input: &str,
    names_from: &str,
    values_from: &str,
    id_cols: &str,
    expected: &str,
) {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            names_from,
            "--values-from",
            values_from,
            "--id-cols",
            id_cols,
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test_case(
    "ID\tKey\tVal\nA\tX\t5\nA\tX\t10\nA\tX\t15",
    "range",
    "A\t10";
    "range_op"
)]
fn test_wider_single_ops(input: &str, op: &str, expected_contains: &str) {
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
            op,
        ])
        .stdin(input)
        .run();

    assert!(
        stdout.contains(expected_contains),
        "Expected '{}' in output: {}",
        expected_contains,
        stdout
    );
}

#[test_case(
    INPUT_SINGLE_ROW,
    "--names-from name --values-from value",
    "ID\tcost\nA\t100";
    "single_row"
)]
#[test_case(
    INPUT_DUPLICATE_ID,
    "--names-from name --values-from value --id-cols ID",
    "ID\tcost\nA\t30";
    "duplicate_id_last_wins"
)]
fn test_wider_basic_scenarios(input: &str, args: &str, expected: &str) {
    let args: Vec<&str> = std::iter::once("wider")
        .chain(args.split_whitespace())
        .collect();
    let (stdout, _) = TvaCmd::new().args(&args).stdin(input).run();
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_empty_values_with_fill() {
    // Test handling of missing columns with custom fill
    // Note: --values-fill is used when a column is missing for an ID,
    // not when the value is empty
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
        .stdin(INPUT_MISSING_VAL)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn wider_multiple_id_columns() {
    // Test with multiple ID columns
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
        .stdin(INPUT_MULTI_ID)
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test_case(
    "ID\tname\tvalue\nA\tX-Y\t1\nA\tX+Y\t2\nB\tX*Y\t3",
    &["X-Y", "X+Y", "X*Y"];
    "special_characters"
)]
#[test_case(
    "ID\tname\tvalue\n中文\t成本\t100\n中文\t尺寸\t50",
    &["中文", "成本", "尺寸", "100", "50"];
    "unicode_content"
)]
fn test_wider_content_types(input: &str, expected_contains: &[&str]) {
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

    for expected in expected_contains {
        assert!(
            stdout.contains(expected),
            "Expected '{}' in output: {}",
            expected,
            stdout
        );
    }
}

#[test_case(
    "ID\tname\tvalue\nA\tsales\t999999999.99\nB\tsales\t1000000000.00",
    &["999999999.99", "1000000000.00"];
    "large_numbers"
)]
#[test_case(
    "ID\tname\tvalue\nA\tprofit\t-100\nA\tloss\t50",
    &["-100", "50"];
    "negative_numbers"
)]
#[test_case(
    "ID\tname\tvalue\nA\tpi\t3.14159265359",
    &["3.14159265359"];
    "float_precision"
)]
fn test_wider_numeric_types(input: &str, expected_contains: &[&str]) {
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

    for expected in expected_contains {
        assert!(
            stdout.contains(expected),
            "Expected '{}' in output: {}",
            expected,
            stdout
        );
    }
}

#[test]
fn wider_only_header_no_data() {
    // Test with only header row, no data
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
        .stdin(INPUT_HEADER_ONLY)
        .run();

    // Should output just the header
    assert!(stdout.contains("ID"));
}

#[test_case(
    "ID\tname\tvalue\nA\tnum\t100\nA\tstr\thello",
    &["100", "hello"];
    "mixed_types"
)]
#[test_case(
    "ID\tname\tvalue\nA B\tx\t1\nA B\ty\t2",
    &["A B"];
    "id_with_spaces"
)]
fn test_wider_special_cases(input: &str, expected_contains: &[&str]) {
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

    for expected in expected_contains {
        assert!(
            stdout.contains(expected),
            "Expected '{}' in output: {}",
            expected,
            stdout
        );
    }
}
