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

#[test]
fn append_track_source() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("input3x2\tfield1\tfield2\tfield3"));
}

#[test]
fn append_source_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--source-header",
            "filename",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.starts_with("filename\tfield1\tfield2\tfield3\n"));
}

#[test]
fn append_file_mapping() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--file",
            "custom_label=tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("custom_label\tfield1\tfield2\tfield3"));
}

#[test]
fn append_invalid_file_mapping() {
    let (_stdout, stderr) = TvaCmd::new()
        .args(&["append", "--file", "invalid_format"])
        .run_fail();

    assert!(stderr.contains(
        "invalid --file value `invalid_format`; expected LABEL=FILE"
    ));
}

#[test]
fn append_invalid_delimiter() {
    let (_stdout, stderr) = TvaCmd::new()
        .args(&["append", "--delimiter", "TooLong"])
        .run_fail();

    assert!(stderr.contains("delimiter must be a single byte"));
}

#[test]
fn append_stdin_default() {
    let (stdout, _) = TvaCmd::new()
        .args(&["append", "--track-source"])
        .stdin("field1\tfield2\nval1\tval2\n")
        .run();

    assert!(stdout.contains("stdin\tfield1\tfield2"));
}

#[test]
fn append_custom_delimiter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "--delimiter",
            ":",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("input3x2:field1\tfield2\tfield3"));
}

#[test]
fn append_subdir_filename_label() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "append",
            "--track-source",
            "tests/data/append/input3x2.tsv",
        ])
        .run();

    assert!(stdout.contains("input3x2\tfield1"));
}
