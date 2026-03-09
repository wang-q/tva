#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn longer_names_sep() {
    let input = "\
ID\twk_1\twk_2
A\t1\t2
B\t3\t4
";
    let expected = "\
ID\tunit\tnum\tvalue
A\twk\t1\t1
A\twk\t2\t2
B\twk\t1\t3
B\twk\t2\t4
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "--cols",
            "2-3",
            "--names-sep",
            "_",
            "--names-to",
            "unit",
            "num",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_invalid_col_index_zero() {
    // Test that column index 0 is rejected (1-based indexing)
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_zero_index.tsv",
            "--cols",
            "0",
        ])
        .run_fail();

    // Error comes from field parser, not our validation
    assert!(stderr.contains("field index must be >= 1"));
}

#[test]
fn longer_only_header_no_data() {
    // Test file with only header row (no data rows)
    // Columns 2-3 (Q1, Q2) are melted, leaving ID and Q3 as id columns
    let expected = "ID\tQ3\tname\tvalue\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_only_header.tsv",
            "--cols",
            "2-3",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_col_index_exceeds_actual_columns() {
    // Test that column index exceeding actual columns is rejected
    let (_, stderr) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input1.tsv", "--cols", "10"])
        .run_fail();

    assert!(stderr.contains("Invalid column index"));
    // Error message should show actual column count (4), not max(1, 4)
    assert!(stderr.contains("4 columns") || stderr.contains("only"));
}

#[test]
fn longer_names_pattern_partial_capture() {
    // Test when regex matches but has fewer capture groups than names-to columns
    // Should fill remaining columns with empty values
    let input = "\
ID	new_sp_m014	new_sp_f014
A	1	2
";
    // Pattern with only 1 capture group, but 2 names-to columns
    let expected = "\
ID	diagnosis	gender_age	value
A	sp_m014\t\t1
A\tsp_f014\t\t2
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "--cols",
            "2-3",
            "--names-pattern",
            "new_(.*)", // Only 1 capture group
            "--names-to",
            "diagnosis",
            "gender_age",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_whitespace_value_not_dropped() {
    // Test that values with only whitespace are NOT dropped by --values-drop-na
    // Current implementation only drops completely empty values (is_empty() check)
    // Whitespace-only values are kept because they are not truly "empty"
    let input = "\
ID	Q1	Q2
A	  	17
B	18	  
";
    // All rows are output because whitespace-only values are not dropped
    // A-Q1 has "  " (kept), A-Q2 has "17" (kept)
    // B-Q1 has "18" (kept), B-Q2 has "  " (kept)
    let expected = "\
ID	name	value
A	Q1	  
A	Q2	17
B	Q1	18
B	Q2	  
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "--cols", "2-3", "--values-drop-na"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_names_pattern() {
    let input = "\
ID\tnew_sp_m014\tnew_sp_f014
A\t1\t2
B\t3\t4
";
    let expected = "\
ID\tdiagnosis\tgender_age\tvalue
A\tsp\tm014\t1
A\tsp\tf014\t2
B\tsp\tm014\t3
B\tsp\tf014\t4
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "--cols",
            "2-3",
            "--names-pattern",
            "new_?(.*)_(.*)",
            "--names-to",
            "diagnosis",
            "gender_age",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_basic() {
    let expected = "\
ID\tname\tvalue
A\tQ1\t1
A\tQ2\t2
A\tQ3\t3
B\tQ1\t4
B\tQ2\t5
B\tQ3\t6
C\tQ1\t7
C\tQ2\t8
C\tQ3\t9
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input1.tsv", "--cols", "2-4"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_names_prefix() {
    let expected = "\
ID\tname\tvalue
A\t1\t1
A\t2\t2
A\t3\t3
B\t1\t4
B\t2\t5
B\t3\t6
C\t1\t7
C\t2\t8
C\t3\t9
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input1.tsv",
            "--cols",
            "2-4",
            "--names-prefix",
            "Q",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_interleaved() {
    let expected = "\
ID\tExtra\tname\tvalue
A\tx\tM1\t1
A\tx\tM2\t2
B\ty\tM1\t3
B\ty\tM2\t4
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_interleaved.tsv",
            "--cols",
            "2,4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_quotes() {
    // Note: tva currently treats quotes as part of the header name in TSV
    let expected = "\
ID\tname\tvalue
A\t\"col 1\"\t1
A\tcol 2\t2
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_quotes.tsv",
            "--cols",
            "2-3",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_mixed() {
    let expected = "\
ID\tname\tvalue
A\tnum\t1
A\ttext\tfoo
B\tnum\t2
B\ttext\tbar
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_mixed.tsv",
            "--cols",
            "2-3",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_cols_by_name_range() {
    let expected = "\
ID\tname\tvalue
A\tnum\t1
A\ttext\tfoo
B\tnum\t2
B\ttext\tbar
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_mixed.tsv",
            "--cols",
            "num-text",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_cols_by_wildcard() {
    let expected = "\
ID\tname\tvalue
A\tQ1\t1
A\tQ2\t2
A\tQ3\t3
B\tQ1\t4
B\tQ2\t5
B\tQ3\t6
C\tQ1\t7
C\tQ2\t8
C\tQ3\t9
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input1.tsv", "--cols", "Q*"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_dup_cols() {
    // When selecting by index (2-3), it should pick the 2nd and 3rd columns
    // regardless of their names being identical.
    // The "name" column in output will contain the column header names.
    // Since both are "val", we expect "val" in the name column for both.

    let expected = "\
ID\textra\tname\tvalue
A\tx\tval\t1
A\tx\tval\t2
B\ty\tval\t3
B\ty\tval\t4
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_dup_cols.tsv",
            "--cols",
            "2-3",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_output_order() {
    // Verifies that output is row-major:
    // For each input row, it outputs all melted columns in order.
    let expected = "\
ID\tQ3\tname\tvalue
A\t3\tQ1\t1
A\t3\tQ2\t2
B\t6\tQ1\t4
B\t6\tQ2\t5
C\t9\tQ1\t7
C\t9\tQ2\t8
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input1.tsv", "--cols", "2-3"]) // Melt Q1 and Q2, leaving Q3 as ID
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_empty_input() {
    let expected = "ID\tname\tvalue\n";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_empty.tsv",
            "--cols",
            "2-3",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_invalid_col() {
    let (_, stderr) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input1.tsv", "--cols", "99"])
        .run_fail();

    assert!(stderr.contains("Invalid column index"));
}

#[test]
fn longer_keep_na() {
    let expected = "\
ID\tname\tvalue
F\tQ1\t16
F\tQ2\t
G\tQ1\t
G\tQ2\t17
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "tests/data/longer/input_na.tsv", "--cols", "2-3"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_multi_id() {
    let expected = "\
ID\tCategory\tname\tvalue
A\tX\tQ1\t1
A\tX\tQ2\t2
B\tY\tQ1\t3
B\tY\tQ2\t4
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_multi_id.tsv",
            "--cols",
            "3-4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_custom_names() {
    let expected = "\
ID\tQuestion\tAnswer
A\tQ1\t1
A\tQ2\t2
A\tQ3\t3
B\tQ1\t4
B\tQ2\t5
B\tQ3\t6
C\tQ1\t7
C\tQ2\t8
C\tQ3\t9
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input1.tsv",
            "--cols",
            "2-4",
            "--names-to",
            "Question",
            "--values-to",
            "Answer",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_drop_na() {
    let expected = "\
ID\tname\tvalue
F\tQ1\t16
G\tQ2\t17
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input_na.tsv",
            "--cols",
            "2-3",
            "--values-drop-na",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_multiple_files() {
    let expected = "\
ID\tname\tvalue
A\tQ1\t1
A\tQ2\t2
A\tQ3\t3
B\tQ1\t4
B\tQ2\t5
B\tQ3\t6
C\tQ1\t7
C\tQ2\t8
C\tQ3\t9
D\tQ1\t10
D\tQ2\t11
D\tQ3\t12
E\tQ1\t13
E\tQ2\t14
E\tQ3\t15
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "tests/data/longer/input1.tsv",
            "tests/data/longer/input2.tsv",
            "--cols",
            "2-4",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_multi_names_to_without_sep_or_pattern() {
    // Test error when multiple names-to provided without --names-sep or --names-pattern (covers L90-93)
    // Note: --names-to accepts multiple values (num_args(1..))
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "longer",
            "--cols",
            "2-3",
            "--names-to",
            "col1",
            "col2", // Two separate arguments
        ])
        .stdin("ID\tA\tB\n1\t2\t3\n")
        .run_fail();

    assert!(stderr.contains("names-sep") || stderr.contains("names-pattern"));
}

#[test]
fn longer_empty_file_skip() {
    // Test empty file handling (covers L107-108)
    let input1 = "ID\tA\tB\n1\ta\tb\n";
    let input2 = ""; // Empty file

    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.tsv");
    let file2 = temp_dir.path().join("file2.tsv");
    std::fs::write(&file1, input1).unwrap();
    std::fs::write(&file2, input2).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
            "--cols",
            "2-3",
        ])
        .run();

    // Should process file1 and skip empty file2
    assert!(stdout.contains("1\tA\ta"));
}

#[test]
fn longer_pattern_no_match_fallback() {
    // Test pattern fallback when regex doesn't match (covers L183-187)
    let input = "\
ID	col_A	col_B
1	2	3
";
    // Pattern expects "new_" prefix which doesn't exist
    let expected = "\
ID	diagnosis	gender_age	value
1\tcol_A\t\t2
1\tcol_B\t\t3
";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "longer",
            "--cols",
            "2-3",
            "--names-pattern",
            "new_?(.*)_(.*)",
            "--names-to",
            "diagnosis",
            "gender_age",
        ])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_empty_lines_in_data() {
    // Test handling of empty lines in data (covers L217-218)
    let input = "\
ID	A	B
1	a	b

2	c	d
";
    let expected = "\
ID	name	value
1	A	a
1	B	b
2	A	c
2	B	d
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "--cols", "2-3"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn longer_field_out_of_bounds() {
    // Test handling when accessing field beyond row length (covers L231-232)
    // This can happen with ragged rows
    let input = "\
ID	A	B
1	a
2	a	b	c
";
    let expected = "\
ID	name	value
1	A	a
1	B	
2	A	a
2	B	b
";

    let (stdout, _) = TvaCmd::new()
        .args(&["longer", "--cols", "2-3"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}
