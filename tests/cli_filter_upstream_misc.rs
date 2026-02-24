use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn upstream_no_header_str_in_fld_2_2() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--str-in-fld")
        .arg("2:2")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n", // Note: input1.tsv has header, but we are treating it as data line if --header not specified?
                           // Upstream test: [tsv-filter --str-in-fld 2:2 input1.tsv]
                           // input1.tsv:
                           // F1	F2	F3	F4
                           // 1	1.0	a	A
                           // 2	2.	b	B
                           // ...
                           // Row 1: "F2" contains "2"? Yes.
                           // Row 2: "1.0" contains "2"? No.
                           // Row 3: "2." contains "2"? Yes.
        "2\t2.\tb\tB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t102\tabc\tAbC\n",
    );
    // Wait, upstream output for this test:
    // F1	F2	F3	F4
    // 2	2.	b	B
    // -2	-2.0	ß	ss
    // 100	102	abc	AbC
    // So the header line IS included in the output because "F2" contains "2".
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_str_eq_3_a() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--str-eq")
        .arg("3:a")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "1\t1.0\ta\tA\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_eq_2_1() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--eq")
        .arg("2:1")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "1\t1.0\ta\tA\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_le_2_101() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--le")
        .arg("2:101")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_no_header_lt_2_101() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--lt")
        .arg("2:101")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_no_header_empty_3() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--empty")
        .arg("3")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_eq_1_100_empty_3() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--eq")
        .arg("1:100")
        .arg("--empty")
        .arg("3")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "100\t100\t\tAbC\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_str_eq_4_abc() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--str-eq")
        .arg("4:ABC")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "10\t10.1\tabc\tABC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_str_eq_3_beta() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--str-eq")
        .arg("3:ß")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "-2\t-2.0\tß\tss\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_regex_4_asc_c() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--regex")
        .arg("4:Às*C")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "0.0\t100.0\tàßc\tÀssC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_regex_4_a_b_b_c() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--regex")
        .arg("4:^A[b|B]C$")
        .arg("tests/data/filter/input1_noheader.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_no_header_ff_eq_1_2() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn upstream_or_eq_1_0_eq_2_101_str_in_fld_4_def() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--eq")
        .arg("1:0")
        .arg("--eq")
        .arg("2:101")
        .arg("--str-in-fld")
        .arg("4:def")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
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
    Ok(())
}

#[test]
fn upstream_or_le_1_neg_0_5_ge_2_101_5() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--or")
        .arg("--le")
        .arg("1:-0.5")
        .arg("--ge")
        .arg("2:101.5")
        .arg("tests/data/filter/input1.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_invert_ff_ne_1_2() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn upstream_invert_eq_1_0_eq_2_100() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--invert")
        .arg("--eq")
        .arg("1:0")
        .arg("--eq")
        .arg("2:100")
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
    Ok(())
}

#[test]
fn upstream_invert_or_eq_1_0_eq_2_101_str_in_fld_4_def() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--invert")
        .arg("--or")
        .arg("--eq")
        .arg("1:0")
        .arg("--eq")
        .arg("2:101")
        .arg("--str-in-fld")
        .arg("4:def")
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
        "-2\t-2.0\tß\tss\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_invert_or_le_1_neg_0_5_ge_2_101_5() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--invert")
        .arg("--or")
        .arg("--le")
        .arg("1:-0.5")
        .arg("--ge")
        .arg("2:101.5")
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
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_delimiter_pipe_eq_2_1() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--delimiter")
        .arg("|")
        .arg("--eq")
        .arg("2:1")
        .arg("tests/data/filter/input2_pipe-sep.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "1|1.0|a|A\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_multi_file_ge_2_23() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("2:23")
        .arg("tests/data/filter/input_3x2.tsv")
        .arg("tests/data/filter/input_emptyfile.tsv")
        .arg("tests/data/filter/input_3x1.tsv")
        .arg("tests/data/filter/input_3x0.tsv")
        .arg("tests/data/filter/input_3x3.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
        "3x1-r1\t201\t301\n",
        "3x3-r3\t23\t33\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_stdin_cat_ge_2_23() -> anyhow::Result<()> {
    let input = std::fs::read_to_string("tests/data/filter/input_3x2.tsv").unwrap();
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("2:23")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_stdin_mixed_ge_2_23() -> anyhow::Result<()> {
    let input = std::fs::read_to_string("tests/data/filter/input_3x3.tsv").unwrap();
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--header")
        .arg("--ge")
        .arg("2:23")
        .arg("--")
        .arg("tests/data/filter/input_3x2.tsv")
        .arg("-")
        .arg("tests/data/filter/input_3x1.tsv")
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Upstream: [cat input_3x2.tsv | tsv-filter --header --ge 2:23 -- input_3x3.tsv - input_3x1.tsv]
    // Order: input_3x2 (stdin? No, upstream example uses `cat input_3x2 | ... -- input_3x3 - input_3x1`)
    // So arguments are `input_3x3`, `-` (stdin which is 3x2), `input_3x1`.
    // Wait, in my test above I am piping `input_3x3` to stdin.
    // And args are `input_3x2`, `-`, `input_3x1`.
    // So order should be: input_3x2, stdin (3x3), input_3x1.
    // Let's verify expected output order.
    let expected = concat!(
        "f1\tf2\tf3\n",
        "3x2-r1\t2001\t3001\n",
        "3x2-r2\t2002\t3002\n",
        "3x3-r3\t23\t33\n",
        "3x1-r1\t201\t301\n",
    );
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn upstream_empty_file_ge_3_100() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("--ge")
        .arg("3:100")
        .arg("tests/data/filter/input_emptyfile.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty());
    Ok(())
}

#[test]
fn upstream_empty_file_header_ge_3_100() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("filter")
        .arg("-H")
        .arg("--ge")
        .arg("3:100")
        .arg("tests/data/filter/input_emptyfile.tsv")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty());
    Ok(())
}
