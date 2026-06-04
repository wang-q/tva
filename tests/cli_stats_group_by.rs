#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use std::io::Write;
use tempfile::NamedTempFile;
use test_case::test_case;

// --- Helper Functions ---

fn create_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    write!(file, "{}", content).expect("failed to write to temp file");
    file
}

// --- Test Data Constants ---

// tsv-summarize file1: 3 fields, 6 rows, includes empty values and empty key
const FILE1: &str = "fld1\tfld2\tfld3
a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";

// Basic input from cli_stats.rs
const INPUT_BASIC: &str = "header1\theader2\tvalue
A\tX\t10
A\tX\t20
A\tY\t30
B\tX\t40
B\tY\t50
B\tY\t60
";

// 5-field input A
const INPUT_5FIELD_A: &str = "color\tpattern\tlength\twidth\theight
red\tsolid\t10\t4\t7
red\tstriped\t8\t6\t6
blue\tsolid\t16\t2\t4
green\tsolid\t11\t5.5\t3.2
blue\tstriped\t12\t1\t2
blue\tsolid\t14\t4\t3
green\tsolid\t7.4\t6.0\t5.4
";

// 5-field input B (with unicode)
const INPUT_5FIELD_B: &str = "color\tpattern\tlength\twidth\theight
red\tsolid\t6\t2\t5
赤\t水玉模様\t8\t6\t6
青\t弁慶縞\t10\t5.5\t4.5
赤\t水玉模様\t9\t7\t8
";

// 5-field input C (single row)
const INPUT_5FIELD_C: &str = "color\tpattern\tlength\twidth\theight
red\tchecked\t10\t4\t7
";

// 5-field input D (small decimals)
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

const INPUT_1FIELD_B: &str = "size
9
medium
10
";

const INPUT_NEW: &str = "A\t10
A\t20
B\t30
B\t40
B\t50
";

// ============================================================================
// Single Key Group-By Tests (from tsv-summarize unittest-sk-*)
// ============================================================================

#[test]
fn group_by_single_key_values_1() {
    // unittest-sk-1: -H -g 1 --values 1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld1_collapse\na\ta|a\nc\tc|c|c\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_1_named() {
    // unittest-sk-1-named: -H -g fld1 --values fld1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "fld1", "--values", "fld1"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld1_collapse\na\ta|a\nc\tc|c|c\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_2() {
    // unittest-sk-2: -H -g 1 --values 2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "2"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld2_collapse\na\ta|c\nc\ta|bc|bc\n\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_2_named() {
    // unittest-sk-2-named: -H -g fld1 --values fld2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "fld1", "--values", "fld2"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld2_collapse\na\ta|c\nc\ta|bc|bc\n\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_3() {
    // unittest-sk-3: -H -g 1 --values 3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld3_collapse\na\t3|2b\nc\t2b||3\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_multi_field() {
    // unittest-sk-4: -H -g 1 --values 1,2,3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1,2,3"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld1_collapse\tfld2_collapse\tfld3_collapse\na\ta|a\ta|c\t3|2b\nc\tc|c|c\ta|bc|bc\t2b||3\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_multi_field_named() {
    // unittest-sk-4-named-a: -H -g fld1 --values fld1,fld2,fld3
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1",
            "--values",
            "fld1,fld2,fld3",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld1_collapse\tfld2_collapse\tfld3_collapse\na\ta|a\ta|c\t3|2b\nc\tc|c|c\ta|bc|bc\t2b||3\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_range() {
    // unittest-sk-5: -H -g 1 --values 1-3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1-3"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld1_collapse\tfld2_collapse\tfld3_collapse\na\ta|a\ta|c\t3|2b\nc\tc|c|c\ta|bc|bc\t2b||3\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_reverse_order() {
    // unittest-sk-6: -H -g 1 --values 3,2,1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3,2,1"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld3_collapse\tfld2_collapse\tfld1_collapse\na\t3|2b\ta|c\ta|a\nc\t2b||3\ta|bc|bc\tc|c|c\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_values_reverse_range() {
    // unittest-sk-7: -H -g 1 --values 3-1 (descending range)
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3-1"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld3_collapse\tfld2_collapse\tfld1_collapse\na\t3|2b\ta|c\ta|a\nc\t2b||3\ta|bc|bc\tc|c|c\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field2() {
    // unittest-sk-8: -H -g 2 --values 1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld1_collapse\na\ta|c\nbc\tc||c\nc\ta\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field2_values_2() {
    // unittest-sk-9: -H -g 2 --values 2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "2"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld2_collapse\na\ta|a\nbc\tbc|bc|bc\nc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field2_values_3() {
    // unittest-sk-10: -H -g 2 --values 3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "3"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld3_collapse\na\t3|2b\nbc\t||3\nc\t2b\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field2_values_1_3() {
    // unittest-sk-11: -H -g 2 --values 1,3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "1,3"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld1_collapse\tfld3_collapse\na\ta|c\t3|2b\nbc\tc||c\t||3\nc\ta\t2b\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field2_values_3_1() {
    // unittest-sk-12: -H -g 2 --values 3,1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "3,1"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld3_collapse\tfld1_collapse\na\t3|2b\ta|c\nbc\t||3\tc||c\nc\t2b\ta\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field3() {
    // unittest-sk-13: -H -g 3 --values 1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "3", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected = "fld3\tfld1_collapse\n3\ta|c\n2b\tc|a\n\tc|\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field3_values_2() {
    // unittest-sk-14: -H -g 3 --values 2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "3", "--values", "2"])
        .stdin(FILE1)
        .run();

    let expected = "fld3\tfld2_collapse\n3\ta|bc\n2b\ta|c\n\tbc|bc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field3_values_1_2() {
    // unittest-sk-15: -H -g 3 --values 1,2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "3", "--values", "1,2"])
        .stdin(FILE1)
        .run();

    let expected = "fld3\tfld1_collapse\tfld2_collapse\n3\ta|c\ta|bc\n2b\tc|a\ta|c\n\tc|\tbc|bc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_single_key_field3_named() {
    // unittest-sk-15-named: -H -g fld3 --values fld1,fld2
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld3",
            "--values",
            "fld1,fld2",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld3\tfld1_collapse\tfld2_collapse\n3\ta|c\ta|bc\n2b\tc|a\ta|c\n\tc|\tbc|bc\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Multi Key Group-By Tests (from tsv-summarize unittest-mk-*)
// ============================================================================

#[test]
fn group_by_multi_key_mk1() {
    // unittest-mk-1: -H -g 1,2 --values 1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1,2", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld2\tfld1_collapse\na\ta\ta\nc\ta\tc\nc\tbc\tc|c\na\tc\ta\n\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk2() {
    // unittest-mk-2: -H -g 1,2 --values 2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1,2", "--values", "2"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld2_collapse\na\ta\ta\nc\ta\ta\nc\tbc\tbc|bc\na\tc\tc\n\tbc\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk3() {
    // unittest-mk-3: -H -g 1,2 --values 3
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1,2", "--values", "3"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld3_collapse\na\ta\t3\nc\ta\t2b\nc\tbc\t|3\na\tc\t2b\n\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk4() {
    // unittest-mk-4: -H -g 1,2 --values 3,1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1,2", "--values", "3,1"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld3_collapse\tfld1_collapse\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\t|3\tc|c\na\tc\t2b\ta\n\tbc\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk4_named() {
    // unittest-mk-4-named: -H -g fld1,fld2 --values fld3,fld1
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1,fld2",
            "--values",
            "fld3,fld1",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld3_collapse\tfld1_collapse\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\t|3\tc|c\na\tc\t2b\ta\n\tbc\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk5() {
    // unittest-mk-5: -H -g 3,2 --values 1
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "3,2", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld3\tfld2\tfld1_collapse\n3\ta\ta\n2b\ta\tc\n\tbc\tc|\n2b\tc\ta\n3\tbc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk6() {
    // unittest-mk-6: -H -g 3-2 --values 1 (descending range)
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "3-2", "--values", "1"])
        .stdin(FILE1)
        .run();

    let expected =
        "fld3\tfld2\tfld1_collapse\n3\ta\ta\n2b\ta\tc\n\tbc\tc|\n2b\tc\ta\n3\tbc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_multi_key_mk7() {
    // unittest-mk-7: -H -g 2,1,3 --values 2
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1,3", "--values", "2"])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld1\tfld3\tfld2_collapse\na\ta\t3\ta\na\tc\t2b\ta\nbc\tc\t\tbc\nc\ta\t2b\tc\nbc\t\t\tbc\nbc\tc\t3\tbc\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Missing Policy Tests (from tsv-summarize unittest-mis-*)
// ============================================================================

#[test]
fn group_by_missing_policy_mis1() {
    // unittest-mis-1: -H -g 1 --values 1 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "1",
            "--exclude-missing",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld1_collapse\na\ta|a\nc\tc|c|c\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis2() {
    // unittest-mis-2: -H -g 1 --values 2 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "2",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld2_collapse\na\ta|c\nc\ta|bc|bc\n\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis3() {
    // unittest-mis-3: -H -g 1 --values 3 -x  (empty values excluded)
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "3",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld3_collapse\na\t3|2b\nc\t2b|3\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis4() {
    // unittest-mis-4: -H -g 1 --values 1,2,3 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "1,2,3",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld1_collapse\tfld2_collapse\tfld3_collapse\na\ta|a\ta|c\t3|2b\nc\tc|c|c\ta|bc|bc\t2b|3\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis5() {
    // unittest-mis-5: -H -g 1 --values 1 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "1",
            "--replace-missing",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld1_collapse\na\ta|a\nc\tc|c|c\n\tNA\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis6() {
    // unittest-mis-6: -H -g 1 --values 2 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "2",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld2_collapse\na\ta|c\nc\ta|bc|bc\n\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis7() {
    // unittest-mis-7: -H -g 1 --values 3 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "3",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld3_collapse\na\t3|2b\nc\t2b|NA|3\n\tNA\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis7_named() {
    // unittest-mis-7-named: -H -g fld1 --values fld3 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1",
            "--values",
            "fld3",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld1\tfld3_collapse\na\t3|2b\nc\t2b|NA|3\n\tNA\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis8() {
    // unittest-mis-8: -H -g 1 --values 1,2,3 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1",
            "--values",
            "1,2,3",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld1_collapse\tfld2_collapse\tfld3_collapse\na\ta|a\ta|c\t3|2b\nc\tc|c|c\ta|bc|bc\t2b|NA|3\n\tNA\tbc\tNA\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis9() {
    // unittest-mis-9: -H -g 1,2 --values 3,1 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1,2",
            "--values",
            "3,1",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld3_collapse\tfld1_collapse\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\t3\tc|c\na\tc\t2b\ta\n\tbc\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis10() {
    // unittest-mis-10: -H -g 3,2 --values 1 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "3,2",
            "--values",
            "1",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld3\tfld2\tfld1_collapse\n3\ta\ta\n2b\ta\tc\n\tbc\tc\n2b\tc\ta\n3\tbc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis11() {
    // unittest-mis-11: -H -g 2,1,3 --values 2 -x
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "2,1,3",
            "--values",
            "2",
            "-x",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld1\tfld3\tfld2_collapse\na\ta\t3\ta\na\tc\t2b\ta\nbc\tc\t\tbc\nc\ta\t2b\tc\nbc\t\t\tbc\nbc\tc\t3\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis12() {
    // unittest-mis-12: -H -g 1,2 --values 3,1 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1,2",
            "--values",
            "3,1",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tfld3_collapse\tfld1_collapse\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\tNA|3\tc|c\na\tc\t2b\ta\n\tbc\tNA\tNA\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis13() {
    // unittest-mis-13: -H -g 3,2 --values 1 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "3,2",
            "--values",
            "1",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld3\tfld2\tfld1_collapse\n3\ta\ta\n2b\ta\tc\n\tbc\tc|NA\n2b\tc\ta\n3\tbc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_missing_policy_mis14() {
    // unittest-mis-14: -H -g 2,1,3 --values 2 -r NA
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "2,1,3",
            "--values",
            "2",
            "-r",
            "NA",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tfld1\tfld3\tfld2_collapse\na\ta\t3\ta\na\tc\t2b\ta\nbc\tc\t\tbc\nc\ta\t2b\tc\nbc\t\t\tbc\nbc\tc\t3\tbc\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Header Variations (from tsv-summarize unittest-hdr-*)
// ============================================================================

#[test]
fn group_by_no_header_hdr1() {
    // unittest-hdr-1: -g 1 --values 1 (no --header, no --write-header)
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--values", "1"])
        .stdin(input_no_header)
        .run();

    let expected = "a\ta|a\nc\tc|c|c\n\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_no_header_multi_hdr2() {
    // unittest-hdr-2: -g 1,2 --values 2 (no header)
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1,2", "--values", "2"])
        .stdin(input_no_header)
        .run();

    let expected = "a\ta\ta\nc\ta\ta\nc\tbc\tbc|bc\na\tc\tc\n\tbc\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_write_header_hdr3() {
    // unittest-hdr-3: -w -g 2 --values 1
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "2",
            "--values",
            "1",
        ])
        .stdin(input_no_header)
        .run();

    let expected = "field2\tfield1_collapse\na\ta|c\nbc\tc||c\nc\ta\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_write_header_multi_hdr4() {
    // unittest-hdr-4: -w -g 3,2 --values 1
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "3,2",
            "--values",
            "1",
        ])
        .stdin(input_no_header)
        .run();

    let expected =
        "field3\tfield2\tfield1_collapse\n3\ta\ta\n2b\ta\tc\n\tbc\tc|\n2b\tc\ta\n3\tbc\tc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_custom_header_hdr5() {
    // unittest-hdr-5: -H -g 2 --values 3:Field3Values
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "2",
            "--values",
            "3:Field3Values",
        ])
        .stdin(FILE1)
        .run();

    let expected = "fld2\tField3Values\na\t3|2b\nbc\t||3\nc\t2b\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_custom_header_multi_hdr6() {
    // unittest-hdr-6: -H -g 1,2 --values 3:FieldThreeValues --values 1:FieldOneValues
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "1,2",
            "--values",
            "3:FieldThreeValues",
            "--values",
            "1:FieldOneValues",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tFieldThreeValues\tFieldOneValues\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\t|3\tc|c\na\tc\t2b\ta\n\tbc\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_custom_header_multi_named() {
    // unittest-hdr-6-named-a: -H -g fld1,fld2 --values fld3:FieldThreeValues --values fld1:FieldOneValues
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1,fld2",
            "--values",
            "fld3:FieldThreeValues",
            "--values",
            "fld1:FieldOneValues",
        ])
        .stdin(FILE1)
        .run();

    let expected =
        "fld1\tfld2\tFieldThreeValues\tFieldOneValues\na\ta\t3\ta\nc\ta\t2b\tc\nc\tbc\t|3\tc|c\na\tc\t2b\ta\n\tbc\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_write_header_custom_hdr7() {
    // unittest-hdr-7: -w -g 1 --values 3:f3_vals --values 2:f2_vals --values 1:f1_vals
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1",
            "--values",
            "3:f3_vals",
            "--values",
            "2:f2_vals",
            "--values",
            "1:f1_vals",
        ])
        .stdin(input_no_header)
        .run();

    let expected =
        "field1\tf3_vals\tf2_vals\tf1_vals\na\t3|2b\ta|c\ta|a\nc\t2b||3\ta|bc|bc\tc|c|c\n\t\tbc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_write_header_custom_multi_hdr8() {
    // unittest-hdr-8: -w -g 1,3,2 --values 3 --values 1:ValsField1 --values 2:ValsField2
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1,3,2",
            "--values",
            "3",
            "--values",
            "1:ValsField1",
            "--values",
            "2:ValsField2",
        ])
        .stdin(input_no_header)
        .run();

    let expected = "field1\tfield3\tfield2\tfield3_collapse\tValsField1\tValsField2\na\t3\ta\t3\ta\ta\nc\t2b\ta\t2b\tc\ta\nc\t\tbc\t\tc\tbc\na\t2b\tc\t2b\ta\tc\n\t\tbc\t\t\tbc\nc\t3\tbc\t3\tc\tbc\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_write_header_range_custom_hdr9() {
    // unittest-hdr-9: -w -g 1,3-2 --values 3 --values 1:ValsField1 --values 2:ValsField2
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1,3-2",
            "--values",
            "3",
            "--values",
            "1:ValsField1",
            "--values",
            "2:ValsField2",
        ])
        .stdin(input_no_header)
        .run();

    let expected = "field1\tfield3\tfield2\tfield3_collapse\tValsField1\tValsField2\na\t3\ta\t3\ta\ta\nc\t2b\ta\t2b\tc\ta\nc\t\tbc\t\tc\tbc\na\t2b\tc\t2b\ta\tc\n\t\tbc\t\t\tbc\nc\t3\tbc\t3\tc\tbc\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Variable File Sizes (from tsv-summarize unittest-3x2-*, 3x1-*, 3x0-*, etc.)
// ============================================================================

#[test]
fn group_by_3x2_1() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\nc\tb\ta\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld3_collapse\na\tc\nc\ta\n");
}

#[test]
fn group_by_3x2_2() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\nc\tb\ta\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld3_collapse\nb\tc|a\n");
}

#[test]
fn group_by_3x2_3() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\nc\tb\ta\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld3_collapse\nb\ta\tc\nb\tc\ta\n");
}

#[test]
fn group_by_3x1_1() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld3_collapse\na\tc\n");
}

#[test]
fn group_by_3x1_2() {
    // No header
    let input = "a\tb\tc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "a\tc\n");
}

#[test]
fn group_by_3x1_3() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld3_collapse\nb\ta\tc\n");
}

#[test]
fn group_by_3x1_3_named() {
    let input = "fld1\tfld2\tfld3\na\tb\tc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld2,fld1",
            "--values",
            "fld3",
        ])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld3_collapse\nb\ta\tc\n");
}

#[test]
fn group_by_3x1_4() {
    let input = "a\tb\tc\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "2,1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "b\ta\tc\n");
}

#[test]
fn group_by_3x0_1() {
    // Header only, no data
    let input = "fld1\tfld2\tfld3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld3_collapse\n");
}

#[test]
fn group_by_3x0_1_named() {
    let input = "fld1\tfld2\tfld3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1",
            "--values",
            "fld3",
        ])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld3_collapse\n");
}

#[test]
fn group_by_3x0_2() {
    // Empty file (no header)
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--values", "3"])
        .stdin("")
        .run();
    assert!(stdout.trim().is_empty());
}

#[test]
fn group_by_3x0_3() {
    // Empty file with --write-header
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1",
            "--values",
            "3",
        ])
        .stdin("")
        .run();
    assert_eq!(stdout, "field1\tfield3_collapse\n");
}

#[test]
fn group_by_3x0_4() {
    let input = "fld1\tfld2\tfld3\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1", "--values", "3"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld3_collapse\n");
}

#[test]
fn group_by_3x0_5() {
    // Empty file, multi key, no header
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "2,1", "--values", "3"])
        .stdin("")
        .run();
    assert!(stdout.trim().is_empty());
}

#[test]
fn group_by_3x0_6() {
    // Empty file with --write-header multi key
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "2,1",
            "--values",
            "3",
        ])
        .stdin("")
        .run();
    assert_eq!(stdout, "field2\tfield1\tfield3_collapse\n");
}

#[test]
fn group_by_2x1_1() {
    let input = "fld1\tfld2\na\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "2"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld2_collapse\na\tb\n");
}

#[test]
fn group_by_2x1_2() {
    let input = "fld1\tfld2\na\tb\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld1_collapse\nb\ta\ta\n");
}

#[test]
fn group_by_2x0_1() {
    let input = "fld1\tfld2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "2"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld2_collapse\n");
}

#[test]
fn group_by_2x0_2() {
    let input = "fld1\tfld2\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "2,1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld2\tfld1\tfld1_collapse\n");
}

#[test]
fn group_by_1x2_1() {
    let input = "fld1\na\n\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\na\ta\n\t\n");
}

#[test]
fn group_by_1x2b_1() {
    let input = "fld1\n\n\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\n\t|\n");
}

#[test]
fn group_by_1x1_1() {
    let input = "fld1\nx\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\nx\tx\n");
}

#[test]
fn group_by_1x1_1_named() {
    let input = "fld1\nx\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--header",
            "--group-by",
            "fld1",
            "--values",
            "fld1",
        ])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\nx\tx\n");
}

#[test]
fn group_by_1x1_2() {
    // No header
    let input = "x\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "x\tx\n");
}

#[test]
fn group_by_1x1_3() {
    // --write-header
    let input = "x\n";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1",
            "--values",
            "1",
        ])
        .stdin(input)
        .run();
    assert_eq!(stdout, "field1\tfield1_collapse\nx\tx\n");
}

#[test]
fn group_by_1x1b_1() {
    let input = "fld1\n\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\n\t\n");
}

#[test]
fn group_by_1x0_1() {
    let input = "fld1\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "1", "--values", "1"])
        .stdin(input)
        .run();
    assert_eq!(stdout, "fld1\tfld1_collapse\n");
}

#[test]
fn group_by_1x0_2() {
    // Empty file, no header
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--values", "1"])
        .stdin("")
        .run();
    assert!(stdout.trim().is_empty());
}

#[test]
fn group_by_1x0_3() {
    // Empty file with --write-header
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "1",
            "--values",
            "1",
        ])
        .stdin("")
        .run();
    assert_eq!(stdout, "field1\tfield1_collapse\n");
}

// ============================================================================
// Delimiter Tests (from tsv-summarize unittest-delim-*)
// ============================================================================

#[test]
fn group_by_delim_4() {
    // unittest-delim-4: -w -g 2 --values 1 -d ^ -v :
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--write-header",
            "--group-by",
            "2",
            "--values",
            "1",
            "--values-delimiter",
            ":",
        ])
        .stdin(input_no_header)
        .run();

    let expected = "field2\tfield1_collapse\na\ta:c\nbc\tc::c\nc\ta\n";
    assert_eq!(stdout, expected);
}

#[test]
fn group_by_delim_5() {
    // unittest-delim-5: -g 1,2 --values 2 -d / -v \
    let input_no_header: &str = "a\ta\t3
c\ta\t2b
c\tbc\t
a\tc\t2b
\tbc\t
c\tbc\t3
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "stats",
            "--group-by",
            "1,2",
            "--values",
            "2",
            "--values-delimiter",
            "\\",
        ])
        .stdin(input_no_header)
        .run();

    let expected = "a\ta\ta\nc\ta\ta\nc\tbc\tbc\\bc\na\tc\tc\n\tbc\tbc\n";
    assert_eq!(stdout, expected);
}

// ============================================================================
// Tests from cli_stats.rs (group-by related)
// ============================================================================

#[test_case(
    "--group-by header1 --count --sum value",
    "header1\tcount\tvalue_sum\nA\t3\t60\nB\t3\t150\n";
    "group_by_single"
)]
#[test_case(
    "--group-by header1,header2 --sum value",
    "header1\theader2\tvalue_sum\nA\tX\t30\nA\tY\t30\nB\tX\t40\nB\tY\t110\n";
    "group_by_multiple"
)]
fn test_stats_group_by_with_header(args: &str, expected: &str) {
    let args: Vec<&str> = std::iter::once("stats")
        .chain(std::iter::once("--header"))
        .chain(args.split_whitespace())
        .collect();
    let (stdout, _) = TvaCmd::new().args(&args).stdin(INPUT_BASIC).run();
    assert_eq!(stdout, expected);
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

    assert!(stdout.contains(
        "color\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"
    ));
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

    assert!(stdout.contains(
        "color\tpattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"
    ));
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

    assert!(
        stdout.contains(
            "color\tpattern\tcount\tlength_min\twidth_min\theight_min\theight_max\twidth_max\tlength_max"
        ),
        "Header mismatch. Actual: {}",
        stdout
    );
    assert!(
        stdout.contains("red\tsolid\t1\t10\t4\t7\t7\t4\t10"),
        "Data mismatch. Actual: {}",
        stdout
    );
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
    common::assert_close(
        blue_parts[blue_parts.len() - 3].parse().unwrap(),
        0.1055,
        1e-4,
    );
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

    assert!(stdout.contains(
        "pattern\tcount\tlength_min\twidth_min\theight_min\tlength_max\twidth_max\theight_max"
    ));
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
fn tsv_utils_test_header_as_data() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1,2", "--count"])
        .stdin(INPUT_5FIELD_A)
        .run();

    assert!(stdout.contains("color\tpattern"));
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
fn stats_group_by_single_group() {
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
    assert!(stdout.contains("60"));
    assert!(stdout.contains("3"));
}

#[test]
fn stats_group_by_many_groups() {
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
fn stats_group_by_no_header() {
    let input = "A\t10\nA\t20\nB\t30\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--group-by", "1", "--sum", "2"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t30"));
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

    assert!(stdout.contains("A"));
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
    common::assert_close(blue_parts[1].parse().unwrap(), 0.1, 1e-4);
    common::assert_close(blue_parts[3].parse().unwrap(), 0.1235, 1e-4);
    common::assert_close(blue_parts[7].parse().unwrap(), 0.1055, 1e-4);

    let red_line = lines
        .iter()
        .find(|l| l.starts_with("red"))
        .expect("Should have red line");
    let red_parts: Vec<&str> = red_line.split('\t').collect();
    common::assert_close(red_parts[7].parse().unwrap(), 0.0774, 1e-4);
}

// ============================================================================
// Error/Edge Cases
// ============================================================================

#[test]
fn stats_group_by_field_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--header", "--group-by", "nonexistent", "--count"])
        .stdin(INPUT_BASIC)
        .run_fail();

    assert!(stderr.contains("not found") || stderr.contains("field"));
}

#[test_case(
    "--group-by x --count",
    INPUT_5FIELD_A,
    "requires header";
    "non_numeric_group_by"
)]
#[test_case(
    "--header --group-by 2 --sum width,len",
    INPUT_5FIELD_A,
    "Field not found";
    "field_not_found_header"
)]
fn test_group_by_errors(args: &str, input: &str, expected_err: &str) {
    let args: Vec<&str> = std::iter::once("stats")
        .chain(args.split_whitespace())
        .collect();
    let (_, stderr) = TvaCmd::new().args(&args).stdin(input).run_fail();
    assert!(
        stderr.contains(expected_err),
        "Expected error containing '{}' in: {}",
        expected_err,
        stderr
    );
}

// ============================================================================
// Strict mode: tsv-summarize errors when row has insufficient fields
// ============================================================================

#[test]
fn group_by_strict_insufficient_fields() {
    // file_1field (1 column only) lacks field 2 needed by --group-by 2
    let input = "size\n10\nsmall\n";
    let (_, stderr) = TvaCmd::new()
        .args(&["stats", "--group-by", "2", "--nunique", "1"])
        .stdin(input)
        .run_fail();

    assert!(
        stderr.contains("Not enough fields"),
        "Expected strict mode error, got: {}",
        stderr
    );
}

#[test]
fn group_by_strict_multi_file_insufficient_fields() {
    // file_1field (1 column only) lacks field 2 needed by --group-by 2
    // With strict mode, this should error even when mixed with valid files
    let file_5field = create_file(INPUT_5FIELD_A);
    let file_1field = create_file(INPUT_1FIELD_A);

    let (_, stderr) = TvaCmd::new()
        .args(&[
            "stats",
            "--group-by",
            "2",
            "--nunique",
            "1",
            file_5field.path().to_str().unwrap(),
            file_1field.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("Not enough fields"),
        "Expected strict mode error for multi-file, got: {}",
        stderr
    );
}