#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

#[test]
fn wider_basic() {
    let input = "
ID\tname\tvalue
A\tcost\t10
A\tsize\t5
B\tcost\t20
B\tsize\t8
";
    let expected = "
ID\tcost\tsize
A\t10\t5
B\t20\t8
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_implicit_id_multi_col() {
    let input = "
A\tB\tkey\tval
1\tx\tk1\t10
1\tx\tk2\t20
2\ty\tk1\t30
";
    // Expected:
    // A  B  k1  k2
    // 1  x  10  20
    // 2  y  30
    let expected = "
A\tB\tk1\tk2
1\tx\t10\t20
2\ty\t30\t
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "key", "--values-from", "val"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_names_sort() {
    let input = "
ID\tkey\tval
1\tb\t2
1\ta\t1
1\tc\t3
";
    let expected = "
ID\ta\tb\tc
1\t1\t2\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "key",
            "--values-from",
            "val",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_custom_fill_string() {
    let input = "
ID\tkey\tval
1\ta\t1
2\tb\t2
";
    let expected = "
ID\ta\tb
1\t1\tmissing
2\tmissing\t2
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "key",
            "--values-from",
            "val",
            "--values-fill",
            "missing",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_missing_values() {
    let input = "
ID\tname\tvalue
A\tcost\t10
B\tsize\t8
";
    let expected = "
ID\tcost\tsize
A\t10\t0
B\t0\t8
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "value",
            "--values-fill",
            "0",
            "--names-sort",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_explicit_id() {
    let input = "
ID\tDate\tname\tvalue
A\t2020\tcost\t10
A\t2021\tcost\t12
";
    let expected = "
ID\tcost
A\t12
";
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
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_doc_example_us_rent_income() {
    let expected = "
GEOID\tNAME\tincome\trent
01\tAlabama\t24476\t747
02\tAlaska\t32940\t1200
04\tArizona\t27517\t972
05\tArkansas\t23789\t709
06\tCalifornia\t29454\t1358
";
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

    assert_eq!(stdout.trim(), expected.trim());
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
    let input = "
ID\tname\tvalue
A\tcost\t 
";
    let expected = "
ID\tcost
A\t 
";
    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_datamash_scenarios() {
    // Scenario 1
    let input1 = "
ID\tKey\tVal
a\tx\t1
a\ty\t2
a\tx\t3
";
    let expected1 = "
ID\tx\ty
a\t3\t2
";
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
        .stdin(input1.trim())
        .run();

    assert_eq!(stdout1.trim(), expected1.trim());

    // Scenario 2
    let input2 = "
ID\tKey\tVal
a\tx\t1
a\ty\t2
b\tx\t3
";
    let expected2 = "
ID\tx\ty
a\t1\t2
b\t3\tXX
";
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
        .stdin(input2.trim())
        .run();

    assert_eq!(stdout2.trim(), expected2.trim());
}

#[test]
fn wider_aggregation_ops() {
    let input = "
ID\tname\tval
A\tX\t10
A\tX\t20
B\tY\t5
B\tY\t15
C\tZ\t100
";

    // 1. Test SUM
    let expected_sum = "
ID\tX\tY\tZ
A\t30\t\t
B\t\t20\t
C\t\t\t100
";
    let (stdout_sum, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "val",
            "--id-cols",
            "ID",
            "--op",
            "sum",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_sum.trim(), expected_sum.trim());

    // 2. Test MEAN
    let expected_mean = "
ID\tX\tY\tZ
A\t15\t\t
B\t\t10\t
C\t\t\t100
";
    let (stdout_mean, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--values-from",
            "val",
            "--id-cols",
            "ID",
            "--op",
            "mean",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_mean.trim(), expected_mean.trim());

    // 3. Test COUNT
    let expected_count = "
ID\tX\tY\tZ
A\t2\t\t
B\t\t2\t
C\t\t\t1
";
    let (stdout_count, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "name",
            "--id-cols",
            "ID",
            "--op",
            "count",
        ])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout_count.trim(), expected_count.trim());
}

#[test]
fn wider_extended_stats() {
    let input = "
ID\tKey\tVal
A\tX\t1
A\tX\t3
A\tX\t5
B\tX\t2
B\tX\t2
B\tX\t8
";

    // 1. Min
    let (stdout_min, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "min",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_min.contains("A\t1"));
    assert!(stdout_min.contains("B\t2"));

    // Max
    let (stdout_max, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "max",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_max.contains("A\t5"));
    assert!(stdout_max.contains("B\t8"));

    // 2. Median
    let (stdout_median, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "median",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_median.contains("A\t3"));
    assert!(stdout_median.contains("B\t2"));

    // 3. Mode
    let (stdout_mode, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "mode",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_mode.contains("A\t1"));
    assert!(stdout_mode.contains("B\t2"));
}

#[test]
fn wider_first_last() {
    let input = "
ID\tKey\tVal
A\tX\tfirst_val
A\tX\tmiddle_val
A\tX\tlast_val
";

    // First
    let (stdout_first, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "first",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_first.contains("A\tfirst_val"));

    // Last
    let (stdout_last, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "last",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_last.contains("A\tlast_val"));
}

#[test]
fn wider_quartiles_iqr() {
    let input = "
ID\tKey\tVal
A\tX\t1
A\tX\t2
A\tX\t3
A\tX\t4
A\tX\t5
";

    // Q1
    let (stdout_q1, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "q1",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_q1.contains("A\t2"));

    // Q3
    let (stdout_q3, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "q3",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_q3.contains("A\t4"));

    // IQR
    let (stdout_iqr, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "iqr",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_iqr.contains("A\t2"));
}

#[test]
fn wider_advanced_math_stats() {
    let input = "
ID\tKey\tVal
A\tX\t2
A\tX\t8
";

    // GeoMean
    let (stdout_geo, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "geomean",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_geo.contains("A\t4"));

    // HarmMean
    let (stdout_harm, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "harmmean",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_harm.contains("A\t3.2"));

    // Variance
    let (stdout_var, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "variance",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_var.contains("A\t18"));

    // Stdev
    let (stdout_std, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "stdev",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_std.contains("A\t4.242"));

    // CV
    let (stdout_cv, _) = TvaCmd::new()
        .args(&[
            "wider",
            "--names-from",
            "Key",
            "--values-from",
            "Val",
            "--id-cols",
            "ID",
            "--op",
            "cv",
        ])
        .stdin(input.trim())
        .run();
    assert!(stdout_cv.contains("A\t0.848"));
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
    let input = "
ID	name
A	X
A	X
B	Y
";
    let expected = "
ID	X	Y
A	2	0
B	0	1
";
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
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_multi_column_names_from_error() {
    // Test multi-column --names-from error (covers L196-199)
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", "--names-from", "1,2", "--values-from", "3"])
        .stdin(
            "A	B	C
1	2	3\n",
        )
        .run_fail();

    assert!(stderr.contains("only single column supported for --names-from"));
}

#[test]
fn wider_multi_column_values_from_error() {
    // Test multi-column --values-from error (covers L207-210)
    let (_, stderr) = TvaCmd::new()
        .args(&["wider", "--names-from", "1", "--values-from", "2,3"])
        .stdin(
            "A	B	C
1	2	3\n",
        )
        .run_fail();

    assert!(stderr.contains("only single column supported for --values-from"));
}

#[test]
fn wider_header_flag() {
    // Test --header flag (FirstLine mode)
    let input = "\
ID	Key	Val
A	X	1
A	Y	2
";
    let expected = "\
ID	X	Y
A	1	2
";

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

    assert_eq!(stdout, expected);
}

#[test]
fn wider_header_hash1() {
    // Test --header-hash1 flag (HashLines1 mode)
    let input = "\
# Comment line
ID	Key	Val
A	X	1
A	Y	2
";
    let expected = "\
ID	X	Y
A	1	2
";

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

    assert_eq!(stdout, expected);
}

#[test]
fn wider_header_hash1_no_hash_lines() {
    // Test --header-hash1 graceful degradation when no hash lines exist
    let input = "\
ID	Key	Val
A	X	1
A	Y	2
";
    let expected = "\
ID	X	Y
A	1	2
";

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

    assert_eq!(stdout, expected);
}

#[test]
fn wider_header_hash1_multiple_comments() {
    // Test --header-hash1 with multiple comment lines
    let input = "\
# First comment
# Second comment
ID	Key	Val
A	X	1
";
    let expected = "\
ID	X
A	1
";

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

    assert_eq!(stdout, expected);
}

#[test]
fn wider_no_header_numeric_fields() {
    // Test that without explicit header flag, first line is treated as header
    // This is the default behavior for backward compatibility
    let input = "\
A	X	1
A	Y	2
";
    // First line "A	X	1" is treated as header
    // Column 1 = "A", Column 2 = "X", Column 3 = "1"
    // names-from=2 -> "X", values-from=3 -> "1", id-cols=1 -> "A"
    // Second line "A	Y	2" is data: ID="A", Name="Y", Value="2"
    let expected = "\
A	Y
A	2
";

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

    assert_eq!(stdout, expected);
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
    let input = "
ID	Key	Val
A	X	5
A	X	10
A	X	15
";

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
        .stdin(input.trim())
        .run();

    // Range = 15 - 5 = 10
    assert!(stdout.contains("A	10"));
}

#[test]
fn wider_single_row_single_column() {
    // Test with minimal data - single row
    let input = "
ID	name	value
A	cost	100
";
    let expected = "
ID	cost
A	100
";

    let (stdout, _) = TvaCmd::new()
        .args(&["wider", "--names-from", "name", "--values-from", "value"])
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_duplicate_id_name_combinations() {
    // Test aggregation when same ID+Name combination appears multiple times
    // Default op is "last", so last value should win
    let input = "
ID	name	value
A	cost	10
A	cost	20
A	cost	30
";
    let expected = "
ID	cost
A	30
";

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
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_numeric_column_indices() {
    // Test using numeric column indices instead of names
    let input = "
col1	col2	col3
A	X	1
A	Y	2
B	X	3
";
    let expected = "
col1	X	Y
A	1	2
B	3	
";

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
    let input = "
ID	name	value
A	x	5
A	y	10
B	x	20
";
    // B doesn't have 'y', so it should be filled with "NA"
    let expected = "
ID	x	y
A	5	10
B	20	NA
";

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
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_multiple_id_columns() {
    // Test with multiple ID columns
    let input = "
Year	Month	Type	Value
2024	Jan	A	100
2024	Jan	B	200
2024	Feb	A	150
";
    let expected = "
Year	Month	A	B
2024	Jan	100	200
2024	Feb	150	
";

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
        .stdin(input.trim())
        .run();

    assert_eq!(stdout.trim(), expected.trim());
}

#[test]
fn wider_special_characters_in_names() {
    // Test handling of special characters in name column
    let input = "
ID	name	value
A	X-Y	1
A	X+Y	2
B	X*Y	3
";

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
        .stdin(input.trim())
        .run();

    // Should contain all special column names
    assert!(stdout.contains("X-Y"));
    assert!(stdout.contains("X+Y"));
    assert!(stdout.contains("X*Y"));
}

#[test]
fn wider_unicode_content() {
    // Test with unicode characters in data
    let input = "
ID	name	value
中文	成本	100
中文	尺寸	50
";

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
        .stdin(input.trim())
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
    let input = "
ID	name	value
A	sales	999999999.99
B	sales	1000000000.00
";

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
        .stdin(input.trim())
        .run();

    assert!(stdout.contains("999999999.99"));
    assert!(stdout.contains("1000000000.00"));
}

#[test]
fn wider_negative_numbers() {
    // Test with negative numbers
    let input = "
ID	name	value
A	profit	-100
A	loss	50
";

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
        .stdin(input.trim())
        .run();

    assert!(stdout.contains("-100"));
    assert!(stdout.contains("50"));
}

#[test]
fn wider_float_precision() {
    // Test float precision handling
    let input = "
ID	name	value
A	pi	3.14159265359
";

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
        .stdin(input.trim())
        .run();

    assert!(stdout.contains("3.14159265359"));
}

#[test]
fn wider_only_header_no_data() {
    // Test with only header row, no data
    let input = "
ID	name	value
";

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
        .stdin(input.trim())
        .run();

    // Should output just the header
    assert!(stdout.contains("ID"));
}

#[test]
fn wider_mixed_types_in_value_column() {
    // Test handling of mixed types (numbers and strings) in value column
    let input = "
ID	name	value
A	num	100
A	str	hello
";

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
        .stdin(input.trim())
        .run();

    // Should handle both numeric and string values
    assert!(stdout.contains("100"));
    assert!(stdout.contains("hello"));
}

#[test]
fn wider_id_cols_with_spaces() {
    // Test ID columns that contain spaces (should be preserved)
    let input = "
ID	name	value
A B	x	1
A B	y	2
";

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
        .stdin(input.trim())
        .run();

    // ID with space should be preserved
    assert!(stdout.contains("A B"));
}
