use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn transpose_simple_matrix_from_stdin() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a\t1\nb\t2\nc\t3\n");

    Ok(())
}

#[test]
fn transpose_empty_input_produces_no_output() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin("").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.is_empty());

    Ok(())
}

#[test]
fn transpose_invalid_structure_from_stdin() -> anyhow::Result<()> {
    let input = "a\tb\tc\n1\t2\n";

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("transpose").write_stdin(input);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("line 2 (2 fields):").and(predicate::str::contains(
                "tva transpose: structure check failed: line 2 has 2 fields (expected 3)",
            )),
        );

    Ok(())
}

#[test]
fn transpose_single_column() -> anyhow::Result<()> {
    let input = "A\nB\nC\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "A\tB\tC\n");

    Ok(())
}

#[test]
fn transpose_single_row() -> anyhow::Result<()> {
    let input = "A\tB\tC\tD\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "A\nB\nC\nD\n");

    Ok(())
}

#[test]
fn transpose_single_field() -> anyhow::Result<()> {
    let input = "A\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "A\n");

    Ok(())
}

#[test]
fn transpose_rectangular_matrix_3x4() -> anyhow::Result<()> {
    let input = "a\tb\tc\td\n1\t2\t3\t4\n5\t6\t7\t8\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("transpose").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let rows: Vec<&str> = input.trim_end().split('\n').collect();
    let mut matrix: Vec<Vec<&str>> = Vec::new();
    for line in rows {
        let fields: Vec<&str> = line.split('\t').collect();
        matrix.push(fields);
    }

    let nrows = matrix.len();
    let ncols = if nrows > 0 { matrix[0].len() } else { 0 };

    let mut expected_lines: Vec<String> = Vec::new();
    for c in 0..ncols {
        let mut fields: Vec<&str> = Vec::new();
        for r in 0..nrows {
            fields.push(matrix[r][c]);
        }
        expected_lines.push(fields.join("\t"));
    }
    let expected = expected_lines.join("\n") + "\n";

    assert_eq!(stdout, expected);

    Ok(())
}
