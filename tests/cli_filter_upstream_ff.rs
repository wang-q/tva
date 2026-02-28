#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_ff_eq_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-eq",
            "1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_rev_2_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "2:1:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_0_02() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-gt",
            "1:2:0.02",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "1:2:1e-7",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected =
        "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_6() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "1:2:1e-6",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "1:2:1e-7",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.001\t  \t3 2-spaces\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-eq",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n100|100|abc|AbC\n0|0.0|z|AzB\n-2|-2.0|ß|ss\n100|100||AbC\n100|100|abc|\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-ne",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = "F1|F2|F3|F4\n10|10.1|abc|ABC\n-1|-0.1|abc def|abc def\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n-0.0|-100.0|àßc|ÀSSC\n100|101||\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-le",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n10|10.1|abc|ABC\n100|100|abc|AbC\n0|0.0|z|AzB\n-1|-0.1|abc def|abc def\n-2|-2.0|ß|ss\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n100|100||AbC\n100|100|abc|\n100|101||\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-str-eq",
            "3:4",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = "F1|F2|F3|F4\n-1|-0.1|abc def|abc def\n100|101||\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-str-ne",
            "3:4",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n10|10.1|abc|ABC\n100|100|abc|AbC\n0|0.0|z|AzB\n-2|-2.0|ß|ss\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n-0.0|-100.0|àßc|ÀSSC\n100|100||AbC\n100|100|abc|\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_no_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--ff-eq",
            "1:2",
            "tests/data/filter/input1_noheader.tsv",
        ])
        .run();
    let expected = "1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-ne",
            "1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-gt",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ge_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-ge",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-1\t-0.1\tabc def\tabc def\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-lt",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-le",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-eq",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-ne",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_0_01() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "F1:F2:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_0_02() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "F1:F2:0.02",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_0_01() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-gt",
            "F1:F2:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_5() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "F1:F2:1e-5",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_6() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "F1:F2:1e-6",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.9999\t a \t3 space prefix&suffix \n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_5() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "F1:F2:1e-5",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-eq",
            "3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n-1\t-0.1\tabc def\tabc def\n100\t101\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-ne",
            "3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-eq",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-ne",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-le",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-1\t-0.1\tabc def\tabc def\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-lt",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_named_both() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-lt",
            "F1:F2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ge_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-ge",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-gt",
            "F1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_named_both() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-gt",
            "F1:F2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-eq",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n-1\t-0.1\tabc def\tabc def\n100\t101\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-ne",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-eq",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-ne",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "F1:F2:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named_rev() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "F2:F1:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named_0_02() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "F1:F2:0.02",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-gt",
            "F1:F2:0.01",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_named_0_02() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-gt",
            "F1:F2:0.02",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "F1:F2:1e-5",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named_1e_6() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "F1:F2:1e-6",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.9999\t a \t3 space prefix&suffix \n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named_1e_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-le",
            "F1:F2:1e-7",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected =
        "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "F1:F2:1e-5",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named_1e_6() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "F1:F2:1e-6",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named_1e_7() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-reldiff-gt",
            "F1:F2:1e-7",
            "tests/data/filter/input2.tsv",
        ])
        .run();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.001\t  \t3 2-spaces\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_ff_ne() {
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
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-eq",
            "3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-ne",
            "3:4",
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
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_f3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-eq",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_f3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-ne",
            "F3:4",
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
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-eq",
            "3:4",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-ne",
            "3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_f3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-eq",
            "F3:4",
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
        "0.\t100.\tàbc\tÀBC\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_f3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-istr-ne",
            "F3:4",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_ff_eq_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-eq",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "1|1.0|a|A\n",
        "2|2.|b|B\n",
        "100|100|abc|AbC\n",
        "0|0.0|z|AzB\n",
        "-2|-2.0|ß|ss\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_ff_ne_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-ne",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "10|10.1|abc|ABC\n",
        "-1|-0.1|abc def|abc def\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "-0.0|-100.0|àßc|ÀSSC\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_ff_le_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-le",
            "1:2",
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
        "-1|-0.1|abc def|abc def\n",
        "-2|-2.0|ß|ss\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_ff_lt_1_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-lt",
            "1:2",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "10|10.1|abc|ABC\n",
        "-1|-0.1|abc def|abc def\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "100|101||\n",
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_delimiter_pipe_ff_str_ne_3_4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--delimiter",
            "|",
            "--ff-str-ne",
            "3:4",
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
        "100|102|abc|AbC\n",
        "100|103|abc|AbC\n",
    );
    assert_eq!(stdout, expected);
}
