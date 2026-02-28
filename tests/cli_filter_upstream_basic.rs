#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_basic_eq_by_index_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    assert_eq!(stdout, "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n");
}

#[test]
fn upstream_basic_le_by_index_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--le",
            "2:101",
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
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_basic_ge_by_index_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "2:101",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_basic_lt_by_index_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--lt",
            "2:101",
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
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_basic_ne_by_index_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ne",
            "2:101",
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
fn upstream_basic_eq_by_name_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "-H",
            "--eq",
            "F2:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    assert_eq!(stdout, "F1\tF2\tF3\tF4\n1\t1.0\ta\tA\n");
}

#[test]
fn upstream_basic_count_eq_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "2:1",
            "--count",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn upstream_basic_count_le_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--le",
            "2:101",
            "-c",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    assert_eq!(stdout.trim(), "13");
}

#[test]
fn upstream_basic_pipe_delimiter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "2:100",
            "--delimiter",
            "|",
            "tests/data/filter/input2_pipe-sep.tsv",
        ])
        .run();
    let expected = concat!(
        "F1|F2|F3|F4\n",
        "100|100|abc|AbC\n",
        "0.|100.|àbc|ÀBC\n",
        "0.0|100.0|àßc|ÀssC\n",
        "100|100||AbC\n",
        "100|100|abc|\n",
    );
    assert_eq!(stdout, expected);
}
