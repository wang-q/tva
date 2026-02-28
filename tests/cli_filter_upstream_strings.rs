#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_str_in_fld_3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_multi() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "3:b",
            "--str-in-fld",
            "4:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_3_b_upper() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "3:B",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_4_sc() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "4:Sc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_4_unicode_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "4:àsSC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_3_unicode_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "3:ẞ",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_not_in_fld_3_b_upper() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-not-in-fld",
            "3:B",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_not_in_fld_4_sc() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-not-in-fld",
            "4:Sc",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_f3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_f3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_f4_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "F4:àsSC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_not_in_fld_f4_sc() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-not-in-fld",
            "F4:Sc",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_not_in_fld_f4_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-not-in-fld",
            "F4:àsSC",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "3: ",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-1\t-0.1\tabc def\tabc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_space_val() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "4:abc def",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-1\t-0.1\tabc def\tabc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_f3_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "F3: ",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-1\t-0.1\tabc def\tabc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_f4_space_val() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "F4:abc def",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-1\t-0.1\tabc def\tabc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_3_sz() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "3:ß",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_not_in_fld_3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-not-in-fld",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_not_in_fld_multi() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-not-in-fld",
            "3:b",
            "--str-not-in-fld",
            "4:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_not_in_fld_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-not-in-fld",
            "3: ",
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
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
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
fn upstream_str_in_fld_range_input4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "2-3,7:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected =
        concat!("line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_range_input4_v2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_not_in_fld_range_input4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-not-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "5\tad\t\t30\t35\t25\tbcdef\t40\t15\n",
        "6\t\t\t-10\t-5\t-25\t\t-15\t-30\n",
        "7\tbcf\tcc\t-20\t-50\t0\tabc\t0\t-5\n",
        "8\tbd\t\t10\t20\t40\tbcd\t15\t25\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_in_fld_range_input4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-in-fld",
            "2-3:ab",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_in_fld_noheader() {
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
fn upstream_or_str_in_fld() {
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
fn upstream_invert_or_str_in_fld() {
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
fn upstream_pipe_str_in_fld_3_b() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-in-fld",
            "3:b",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "2|2.|b|B\n",
        "10|10.1|abc|ABC\n",
        "100|100|abc|AbC\n",
        "-1|-0.1|abc def|abc def\n",
        "0.|100.|àbc|ÀBC\n",
        "100|100|abc|\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_pipe_str_in_fld_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-in-fld",
            "3: ",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!("F1|F2|F3|F4\n", "-1|-0.1|abc def|abc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_pipe_str_in_fld_val() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-in-fld",
            "4:abc def",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!("F1|F2|F3|F4\n", "-1|-0.1|abc def|abc def\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_pipe_str_not_in_fld_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-not-in-fld",
            "3: ",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "1|1.0|a|A\n",
        "2|2.|b|B\n",
        "10|10.1|abc|ABC\n",
        "100|100|abc|AbC\n",
        "0|0.0|z|AzB\n",
        "-2|-2.0|ß|ss\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "-0.0|-100.0|àßc|ÀSSC\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}
#[test]
fn upstream_str_ne_none_100_0() {
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

#[test]
fn upstream_str_ne_none_100_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--not-blank",
            "f1",
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

#[test]
fn upstream_str_eq_none_100_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--blank",
            "1",
            "--str-eq",
            "1:none",
            "--eq",
            "1:100",
            "tests/data/filter/input_num_or_empty.tsv",
        ])
        .run();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "100\t21\t31\n",
        "  \t22\t32\n",
        "\t23\t33\n",
        "100\t24\t33\n",
        "none\t25\t34\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ne_none_100_3() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--not-blank",
            "1",
            "--str-ne",
            "1:none",
            "--eq",
            "1:100",
            "tests/data/filter/input_num_or_empty.tsv",
        ])
        .run();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "  \t22\t32\n",
        "\t23\t33\n",
        "none\t25\t34\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_none_100_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--invert",
            "--or",
            "--blank",
            "1",
            "--str-eq",
            "1:none",
            "--eq",
            "1:100",
            "tests/data/filter/input_num_or_empty.tsv",
        ])
        .run();
    let expected = concat!("f1\tf2\tf3\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_a_5() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "3:a",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "1\t1.0\ta\tA\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_b_6() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "2\t2.\tb\tB\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_abc_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "3:abc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_abc_8() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "4:ABC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "10\t10.1\tabc\tABC\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_9() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "3:ß",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_c_10() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "3:àßc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ne_b_11() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-ne",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
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
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_le_b_12() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-le",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_lt_b_13() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-lt",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ge_b_14() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-ge",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_gt_b_15() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-gt",
            "3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_abc_16() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "4:ABC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_abc_17() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "4:aBc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_ssc_18() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "4:ÀSSC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_ssc_19() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "4:àssc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_20() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "3:ß",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_21() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "3:ẞ",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_c_22() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "3:ÀßC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_ne_abc_23() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-ne",
            "4:ABC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_ne_ssc_24() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-ne",
            "4:ÀSSC",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_abc_25() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "F3:abc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_abc_26() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "F4:ABC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "10\t10.1\tabc\tABC\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_27() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "F3:ß",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_c_28() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "F3:àßc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ne_b_29() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-ne",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
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
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_le_b_30() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-le",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_lt_b_31() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-lt",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ge_b_32() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-ge",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_gt_b_33() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-gt",
            "F3:b",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_abc_34() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "F4:aBc",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_c_35() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-eq",
            "F3:ÀßC",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_ne_ssc_36() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--istr-ne",
            "F4:ÀSSC",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_0_input4_37() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--str-eq",
            "4-6:0",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "9\t\t\t0\t0\t0\t\t0\t0\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_abc_input4_38() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--istr-eq",
            "2-3:abc",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_istr_eq_abc_input4_39() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--or",
            "--istr-eq",
            "2-3:ABC",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "2\tabcd\tabc\t20\t5\t35\tbcd\t15\t40\n",
        "10\tABCD\tABC\t20\t5\t35\tBCD\t15\t40\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_a_40() {
    let (stdout, _) = TvaCmd::new()
        .args(&["filter", "--str-eq", "3:a", "tests/data/filter/input1.tsv"])
        .run();
    let expected = concat!("1\t1.0\ta\tA\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_abc_noheader_41() {
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
fn upstream_str_eq_noheader_42() {
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
fn upstream_str_eq_a_pipe_43() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-eq",
            "3:a",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!("F1|F2|F3|F4\n", "1|1.0|a|A\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_pipe_44() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-eq",
            "3:ß",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!("F1|F2|F3|F4\n", "-2|-2.0|ß|ss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_eq_c_pipe_45() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-eq",
            "3:àßc",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "0.0|100.0|àßc|ÀssC\n",
        "-0.0|-100.0|àßc|ÀSSC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_ne_b_pipe_46() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-ne",
            "3:b",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "1|1.0|a|A\n",
        "10|10.1|abc|ABC\n",
        "100|100|abc|AbC\n",
        "0|0.0|z|AzB\n",
        "-1|-0.1|abc def|abc def\n",
        "-2|-2.0|ß|ss\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "-0.0|-100.0|àßc|ÀSSC\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_str_lt_b_pipe_47() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--str-lt",
            "3:b",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "1|1.0|a|A\n",
        "10|10.1|abc|ABC\n",
        "100|100|abc|AbC\n",
        "-1|-0.1|abc def|abc def\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}
