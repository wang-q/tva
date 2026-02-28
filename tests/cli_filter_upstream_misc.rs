#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_no_header_str_in_fld_2_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--str-in-fld",
            "2:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t102\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_str_eq_3_a() {
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--str-eq", "3:a", "tests/data/filter/input1.tsv"])
        .run();
    let expected = concat!("1\t1.0\ta\tA\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_eq_2_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--eq",
            "2:1",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("1\t1.0\ta\tA\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_le_2_101() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--le",
            "2:101",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!(
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_lt_2_101() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--lt",
            "2:101",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!(
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_empty_3() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--empty",
            "3",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_eq_1_100_empty_3() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--eq",
            "1:100",
            "--empty",
            "3",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_str_eq_4_abc() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--str-eq",
            "4:ABC",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("10\t10.1\tabc\tABC\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_str_eq_3_beta() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--str-eq",
            "3:ß",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_regex_4_asc_c() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--regex",
            "4:Às*C",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!("0.0\t100.0\tàßc\tÀssC\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_regex_4_a_b_b_c() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--regex",
            "4:^A[b|B]C$",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!(
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_no_header_ff_eq_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--ff-eq",
            "1:2",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = concat!(
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_or_eq_1_0_eq_2_101_str_in_fld_4_def() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--eq",
            "1:0",
            "--eq",
            "2:101",
            "--str-in-fld",
            "4:def",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_or_le_1_neg_0_5_ge_2_101_5() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--le",
            "1:-0.5",
            "--ge",
            "2:101.5",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_ff_ne_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--ff-ne",
            "1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_eq_1_0_eq_2_100() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--eq",
            "1:0",
            "--eq",
            "2:100",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_or_eq_1_0_eq_2_101_str_in_fld_4_def() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--or",
            "--eq",
            "1:0",
            "--eq",
            "2:101",
            "--str-in-fld",
            "4:def",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-2\t-2.0\tß\tss\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_or_le_1_neg_0_5_ge_2_101_5() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--or",
            "--le",
            "1:-0.5",
            "--ge",
            "2:101.5",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_eq_2_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--eq",
            "2:1",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!("F1|F2|F3|F4\n", "1|1.0|a|A\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_multi_file_ge_2_23() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "2:23",
            "tests/data/filter/input_3x2.tsv",
            "tests/data/filter/input_emptyfile.tsv",
            "tests/data/filter/input_3x1.tsv",
            "tests/data/filter/input_3x0.tsv",
            "tests/data/filter/input_3x3.tsv",
        ])
        .run();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
        "3x1-r1\t201\t301\n",
        "3x3-r3\t23\t33\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_stdin_cat_ge_2_23() {
    let input = std::fs::read_to_string("tests/data/filter/input_3x2.tsv").unwrap();
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--header", "--ge", "2:23"])
        .stdin(input)
        .run();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_stdin_mixed_ge_2_23() {
    let input = std::fs::read_to_string("tests/data/filter/input_3x3.tsv").unwrap();
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "2:23",
            "--",
            "tests/data/filter/input_3x2.tsv",
            "-",
            "tests/data/filter/input_3x1.tsv",
        ])
        .stdin(input)
        .run();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
        "3x3-r3\t23\t33\n",
        "3x1-r1\t201\t301\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_empty_file_ge_3_100() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--ge",
            "3:100",
            "tests/data/filter/input_emptyfile.tsv",
        ])
        .run();
    assert!(stdout.is_empty());
}

#[test]
fn upstream_empty_file_header_ge_3_100() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "-H",
            "--ge",
            "3:100",
            "tests/data/filter/input_emptyfile.tsv",
        ])
        .run();
    assert!(stdout.is_empty());
}
