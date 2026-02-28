#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_label_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "Pass",
            "--label-values",
            "Y:N",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\tY\n",
        "2\t2.\tb\tB\tN\n",
        "10\t10.1\tabc\tABC\tN\n",
        "100\t100\tabc\tAbC\tN\n",
        "0\t0.0\tz\tAzB\tN\n",
        "-1\t-0.1\tabc def\tabc def\tN\n",
        "-2\t-2.0\tß\tss\tN\n",
        "0.\t100.\tàbc\tÀBC\tN\n",
        "0.0\t100.0\tàßc\tÀssC\tN\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tN\n",
        "100\t100\t\tAbC\tN\n",
        "100\t100\tabc\t\tN\n",
        "100\t101\t\t\tN\n",
        "100\t102\tabc\tAbC\tN\n",
        "100\t103\tabc\tAbC\tN\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_label_default_values() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "Pass",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\t1\n",
        "2\t2.\tb\tB\t0\n",
        "10\t10.1\tabc\tABC\t0\n",
        "100\t100\tabc\tAbC\t0\n",
        "0\t0.0\tz\tAzB\t0\n",
        "-1\t-0.1\tabc def\tabc def\t0\n",
        "-2\t-2.0\tß\tss\t0\n",
        "0.\t100.\tàbc\tÀBC\t0\n",
        "0.0\t100.0\tàßc\tÀssC\t0\n",
        "-0.0\t-100.0\tàßc\tÀSSC\t0\n",
        "100\t100\t\tAbC\t0\n",
        "100\t100\tabc\t\t0\n",
        "100\t101\t\t\t0\n",
        "100\t102\tabc\tAbC\t0\n",
        "100\t103\tabc\tAbC\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_label_values_custom() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "Pass",
            "--label-values",
            "Yes:No",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\tYes\n",
        "2\t2.\tb\tB\tNo\n",
        "10\t10.1\tabc\tABC\tNo\n",
        "100\t100\tabc\tAbC\tNo\n",
        "0\t0.0\tz\tAzB\tNo\n",
        "-1\t-0.1\tabc def\tabc def\tNo\n",
        "-2\t-2.0\tß\tss\tNo\n",
        "0.\t100.\tàbc\tÀBC\tNo\n",
        "0.0\t100.0\tàßc\tÀssC\tNo\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tNo\n",
        "100\t100\t\tAbC\tNo\n",
        "100\t100\tabc\t\tNo\n",
        "100\t101\t\t\tNo\n",
        "100\t102\tabc\tAbC\tNo\n",
        "100\t103\tabc\tAbC\tNo\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_label_values_missing_second() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "Pass",
            "--label-values",
            "Yes:",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tPass\n",
        "1\t1.0\ta\tA\tYes\n",
        "2\t2.\tb\tB\t\n",
        "10\t10.1\tabc\tABC\t\n",
        "100\t100\tabc\tAbC\t\n",
        "0\t0.0\tz\tAzB\t\n",
        "-1\t-0.1\tabc def\tabc def\t\n",
        "-2\t-2.0\tß\tss\t\n",
        "0.\t100.\tàbc\tÀBC\t\n",
        "0.0\t100.0\tàßc\tÀssC\t\n",
        "-0.0\t-100.0\tàßc\tÀSSC\t\n",
        "100\t100\t\tAbC\t\n",
        "100\t100\tabc\t\t\n",
        "100\t101\t\t\t\n",
        "100\t102\tabc\tAbC\t\n",
        "100\t103\tabc\tAbC\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_label_values_missing_first() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "P",
            "--label-values",
            ":No",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\tP\n",
        "1\t1.0\ta\tA\t\n",
        "2\t2.\tb\tB\tNo\n",
        "10\t10.1\tabc\tABC\tNo\n",
        "100\t100\tabc\tAbC\tNo\n",
        "0\t0.0\tz\tAzB\tNo\n",
        "-1\t-0.1\tabc def\tabc def\tNo\n",
        "-2\t-2.0\tß\tss\tNo\n",
        "0.\t100.\tàbc\tÀBC\tNo\n",
        "0.0\t100.0\tàßc\tÀssC\tNo\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tNo\n",
        "100\t100\t\tAbC\tNo\n",
        "100\t100\tabc\t\tNo\n",
        "100\t101\t\t\tNo\n",
        "100\t102\tabc\tAbC\tNo\n",
        "100\t103\tabc\tAbC\tNo\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_label_no_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--label",
            "Pass",
            "--label-values",
            "Y:N",
            "--eq",
            "2:1",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!(
        "1\t1.0\ta\tA\tY\n",
        "2\t2.\tb\tB\tN\n",
        "10\t10.1\tabc\tABC\tN\n",
        "100\t100\tabc\tAbC\tN\n",
        "0\t0.0\tz\tAzB\tN\n",
        "-1\t-0.1\tabc def\tabc def\tN\n",
        "-2\t-2.0\tß\tss\tN\n",
        "0.\t100.\tàbc\tÀBC\tN\n",
        "0.0\t100.0\tàßc\tÀssC\tN\n",
        "-0.0\t-100.0\tàßc\tÀSSC\tN\n",
        "100\t100\t\tAbC\tN\n",
        "100\t100\tabc\t\tN\n",
        "100\t101\t\t\tN\n",
        "100\t102\tabc\tAbC\tN\n",
        "100\t103\tabc\tAbC\tN\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_empty_with_other_filter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "1:100",
            "--empty",
            "3",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_not_blank_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-blank",
            "3",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    // input2.tsv has 14 lines (header + 13 data)
    // Blank 3rd field:
    // 1000	1000.0		3 empty
    // 1000	1000.0	 	3 1-space
    // 1000	1000.001	  	3 2-spaces
    // 3 lines blank. 10 lines remain.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 12); // Header + 11 lines
    assert_eq!(lines[0], "F1\tF2\tF3\tF4");
}

#[test]
fn upstream_empty_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--empty",
            "3",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_not_empty_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-empty",
            "3",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 14); // Header + 13 lines
    assert_eq!(lines[0], "F1\tF2\tF3\tF4");
}

#[test]
fn upstream_blank_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--blank",
            "3",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1000\t1000.0\t\t3 empty\n",
        "1000\t1000.0\t \t3 1-space\n",
        "1000\t1000.001\t  \t3 2-spaces\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_empty_header_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--empty",
            "F3",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_not_empty_header_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "F1:100",
            "--not-empty",
            "F4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_blank_header_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--blank",
            "F3",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1000\t1000.0\t\t3 empty\n",
        "1000\t1000.0\t \t3 1-space\n",
        "1000\t1000.001\t  \t3 2-spaces\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_not_blank_header_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-blank",
            "F3",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 12);
    assert_eq!(lines[0], "F1\tF2\tF3\tF4");
}

#[test]
fn upstream_blank_multi_field() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--blank",
            "2,3",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_empty_multi_field() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--empty",
            "2,3,7",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_not_blank_onefield() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--not-blank",
            "1",
            "tests/data/filter/input_onefield.txt",
        ])
        .run();
    let expected = concat!(
        "no header\n",
        "no real fields, just some text\n",
        "abc def\n",
        "abc def\n",
        "abc def ghi\n",
        "previous line empty\n",
        "previous line empty, 2-back 1 space, 3-back 2 spaces\n",
        "previous line empty, 2-back 1 space, 3-back 2 spaces\n",
        "last line\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_empty_onefield() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--empty",
            "1",
            "tests/data/filter/input_onefield.txt",
        ])
        .run();
    let expected = concat!("\n", "\n", "\n", "\n", "\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_num_or_empty() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-blank",
            "1",
            "--str-ne",
            "1:none",
            "--eq",
            "1:100",
            "tests/data/filter/input_num_or_empty.tsv",
        ])
        .run();
    let expected = concat!("f1\tf2\tf3\n", "100\t21\t31\n", "100\t24\t33\n",);
    assert_eq!(stdout, expected);
}
