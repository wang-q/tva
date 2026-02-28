#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn append_basic_input3x2_input3x5() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_input1x3_input1x4() {
    let expected = "field1
row 1
row 2
field1
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_four_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
field1
row 1
row 2
field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
field1
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_basic_single_file() {
    let expected = "field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_two_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_two_single_column_files() {
    let expected = "field1
row 1
row 2
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_four_files() {
    let expected = "field1\tfield2\tfield3
abc\tdef\tghi
row 1
row 2
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
next-empty

last-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_single_file() {
    let expected = "field1\tfield2\tfield3
jkl\tmno\tpqr
123\t456\t789
xy1\txy2\txy3
pqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_two_files() {
    let expected = "input3x2\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_two_single_column_files() {
    let expected = "input1x3\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tfield1
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-t",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_four_files() {
    let expected = "input3x2\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\tfield1
input1x3\trow 1
input1x3\trow 2
input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tfield1
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-t",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_track_source_single_file() {
    let expected = "input3x5\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-t", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_two_files() {
    let expected = "file\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--header",
            "--track-source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_two_single_column_files() {
    let expected = "file\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_four_files() {
    let expected = "file\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\trow 1
input1x3\trow 2
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-H",
            "-t",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_header_and_track_source_single_file() {
    let expected = "file\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-H", "-t", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_two_files() {
    let expected = "source\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input3x5.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_two_single_column_files() {
    let expected = "source\tfield1
input1x3\trow 1
input1x3\trow 2
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "source",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_four_files() {
    let expected = "source\tfield1\tfield2\tfield3
input3x2\tabc\tdef\tghi
input1x3\trow 1
input1x3\trow 2
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
input1x4\tnext-empty
input1x4\t
input1x4\tlast-line
";
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "-s",
            "source",
            "tests/data/append/input3x2.tsv",
            "tests/data/append/input1x3.tsv",
            "tests/data/append/input3x5.tsv",
            "tests/data/append/input1x4.tsv",
        ])
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn append_source_header_single_file() {
    let expected = "source\tfield1\tfield2\tfield3
input3x5\tjkl\tmno\tpqr
input3x5\t123\t456\t789
input3x5\txy1\txy2\txy3
input3x5\tpqx\tpqy\tpqz
";
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "-s", "source", "tests/data/append/input3x5.tsv"])
        .run();

    assert_eq!(stdout, expected);
}
