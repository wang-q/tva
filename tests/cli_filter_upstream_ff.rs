use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_ff_eq_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-eq")
        .arg("1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_rev_2_1() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("2:1:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_0_02() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-gt")
        .arg("1:2:0.02")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_7() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("1:2:1e-7")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected =
        "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_6() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("1:2:1e-6")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_7() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("1:2:1e-7")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.001\t  \t3 2-spaces\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_pipe() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-eq")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n100|100|abc|AbC\n0|0.0|z|AzB\n-2|-2.0|ß|ss\n100|100||AbC\n100|100|abc|\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_pipe() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-ne")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1|F2|F3|F4\n10|10.1|abc|ABC\n-1|-0.1|abc def|abc def\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n-0.0|-100.0|àßc|ÀSSC\n100|101||\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_pipe() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-le")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n10|10.1|abc|ABC\n100|100|abc|AbC\n0|0.0|z|AzB\n-1|-0.1|abc def|abc def\n-2|-2.0|ß|ss\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n100|100||AbC\n100|100|abc|\n100|101||\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_pipe() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-str-eq")
        .arg("3:4")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1|F2|F3|F4\n-1|-0.1|abc def|abc def\n100|101||\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_pipe() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-str-ne")
        .arg("3:4")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1|F2|F3|F4\n1|1.0|a|A\n2|2.|b|B\n10|10.1|abc|ABC\n100|100|abc|AbC\n0|0.0|z|AzB\n-2|-2.0|ß|ss\n0.|100.|àbc|ÀBC\n0.0|100.0|àßc|ÀssC\n-0.0|-100.0|àßc|ÀSSC\n100|100||AbC\n100|100|abc|\n100|102|abc|AbC\n100|103|abc|AbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_no_header() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ff-eq")
        .arg("1:2")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-ne")
        .arg("1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-gt")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ge_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-ge")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-1\t-0.1\tabc def\tabc def\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-lt")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-le")
        .arg("2:1")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-eq")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-ne")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_0_01() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("F1:F2:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_0_02() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("F1:F2:0.02")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_0_01() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-gt")
        .arg("F1:F2:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_5() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("F1:F2:1e-5")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_1e_6() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("F1:F2:1e-6")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.9999\t a \t3 space prefix&suffix \n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_1e_5() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("F1:F2:1e-5")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-eq")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n-1\t-0.1\tabc def\tabc def\n100\t101\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_basic() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_eq_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-eq")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ne_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-ne")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_le_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-le")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-1\t-0.1\tabc def\tabc def\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-lt")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_lt_named_both() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-lt")
        .arg("F1:F2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n10\t10.1\tabc\tABC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_ge_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-ge")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-gt")
        .arg("F1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_gt_named_both() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-gt")
        .arg("F1:F2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n-0.0\t-100.0\tàßc\tÀSSC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-eq")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n-1\t-0.1\tabc def\tabc def\n100\t101\t\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_ne_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.\t100.\tàbc\tÀBC\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_eq_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-eq")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n10\t10.1\tabc\tABC\n100\t100\tabc\tAbC\n-1\t-0.1\tabc def\tabc def\n0.\t100.\tàbc\tÀBC\n100\t101\t\t\n100\t102\tabc\tAbC\n100\t103\tabc\tAbC\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_istr_ne_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-ne")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n0.0\t100.0\tàßc\tÀssC\n-0.0\t-100.0\tàßc\tÀSSC\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("F1:F2:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named_rev() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("F2:F1:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_le_named_0_02() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-le")
        .arg("F1:F2:0.02")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-gt")
        .arg("F1:F2:0.01")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_absdiff_gt_named_0_02() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-absdiff-gt")
        .arg("F1:F2:0.02")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("F1:F2:1e-5")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.999\t abc\t3 space prefix\n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named_1e_6() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("F1:F2:1e-6")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n1000\t1000.001\t  \t3 2-spaces\n1000\t999.9999\t a \t3 space prefix&suffix \n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_le_named_1e_7() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-le")
        .arg("F1:F2:1e-7")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected =
        "F1\tF2\tF3\tF4\n1000\t1000.0\t\t3 empty\n1000\t1000.0\t \t3 1-space\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("F1:F2:1e-5")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999\tabc \t3 space suffix \n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named_1e_6() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("F1:F2:1e-6")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_reldiff_gt_named_1e_7() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-reldiff-gt")
        .arg("F1:F2:1e-7")
        .arg("tests/data/filter/input2.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1000\t1000.001\t  \t3 2-spaces\n1000\t1001\tabc\t3 no space\n1000\t999.999\t abc\t3 space prefix\n1000\t999\tabc \t3 space suffix \n1000\t999.9999\t a \t3 space prefix&suffix \n999.999\t1000\tx\tx\n999.999\t1000.999\tx\tx\n1000\t1001.1\tx\tx\n-999.99\t-1000\tx\tx\n-999.98\t-1000\tx\tx\n-999.99\t1000\tx\tx\n999.99\t-1000\tx\tx\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_invert_ff_ne() {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--invert")
        .arg("--ff-ne")
        .arg("1:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n2\t2.\tb\tB\n100\t100\tabc\tAbC\n0\t0.0\tz\tAzB\n-2\t-2.0\tß\tss\n100\t100\t\tAbC\n100\t100\tabc\t\n";
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_ff_str_eq_3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-eq")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_ff_str_ne_3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_ff_str_eq_f3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-eq")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_ff_str_ne_f3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-str-ne")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_ff_istr_eq_3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-eq")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_ff_istr_ne_3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-ne")
        .arg("3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_ff_istr_eq_f3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-eq")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_ff_istr_ne_f3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ff-istr-ne")
        .arg("F3:4")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_ff_eq_1_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-eq")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_ff_ne_1_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-ne")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_ff_le_1_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-le")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_ff_lt_1_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-lt")
        .arg("1:2")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_ff_str_ne_3_4() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--ff-str-ne")
        .arg("3:4")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}
