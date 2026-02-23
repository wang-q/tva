use assert_cmd::cargo::cargo_bin_cmd;

fn read_golden(name: &str) -> String {
    let path = format!("tests/data/keep_header/{}", name);
    std::fs::read_to_string(path).unwrap()
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn keep_header_single_file_sort() -> anyhow::Result<()> {
    let expected = read_golden("gold_single_sort.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/data/keep_header/input1.csv")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn keep_header_multi_file_sort() -> anyhow::Result<()> {
    let expected = read_golden("gold_multi_sort.txt");

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/data/keep_header/input1.csv")
        .arg("tests/data/keep_header/input2.csv")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn keep_header_multi_line_header_with_lines_option() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("--lines")
        .arg("2")
        .arg("-")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .write_stdin(read_golden("multi_header.txt"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2);
    assert_eq!(lines[0], "H1");
    assert_eq!(lines[1], "H2");

    Ok(())
}

#[test]
fn keep_header_single_file_sort_reverse() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/data/keep_header/input1.csv")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .arg("-r")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn keep_header_single_file_sort_numeric_second_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/data/keep_header/input1.csv")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .arg("-t")
        .arg(",")
        .arg("-k")
        .arg("2")
        .arg("-n")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn keep_header_input1_twice_numeric_second_field() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("keep-header")
        .arg("tests/data/keep_header/input1.csv")
        .arg("tests/data/keep_header/input1.csv")
        .arg("--")
        .arg(env!("CARGO_BIN_EXE_tva"))
        .arg("sort")
        .arg("-t")
        .arg(",")
        .arg("-k")
        .arg("2")
        .arg("-n")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = normalize_newlines(&stdout);

    let expected = "file.row,field1,field2,field3\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.3,10,绿色|蓝色\n\
input1.txt.2,20,緑|青\n\
input1.txt.2,20,緑|青\n\
input1.txt.1,30,green|blue\n\
input1.txt.1,30,green|blue\n";

    assert_eq!(stdout, expected);

    Ok(())
}
