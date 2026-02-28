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
        .args(&[
            "longer",
            "tests/data/longer/input1.tsv",
            "--cols",
            "2-3",
        ]) // Melt Q1 and Q2, leaving Q3 as ID
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
