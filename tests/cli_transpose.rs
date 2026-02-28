#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn transpose_simple_matrix_from_stdin() {
    let input = "a\tb\tc\n1\t2\t3\n";

    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin(input).run();

    assert_eq!(stdout, "a\t1\nb\t2\nc\t3\n");
}

#[test]
fn transpose_empty_input_produces_no_output() {
    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin("").run();

    assert!(stdout.is_empty());
}

#[test]
fn transpose_invalid_structure_from_stdin() {
    let input = "a\tb\tc\n1\t2\n";

    let (_, stderr) = TvaCmd::new().args(&["transpose"]).stdin(input).run_fail();

    assert!(stderr.contains("line 2 (2 fields):"));
    assert!(stderr.contains(
        "tva transpose: structure check failed: line 2 has 2 fields (expected 3)"
    ));
}

#[test]
fn transpose_single_column() {
    let input = "A\nB\nC\n";

    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin(input).run();

    assert_eq!(stdout, "A\tB\tC\n");
}

#[test]
fn transpose_single_row() {
    let input = "A\tB\tC\tD\n";

    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin(input).run();

    assert_eq!(stdout, "A\nB\nC\nD\n");
}

#[test]
fn transpose_single_field() {
    let input = "A\n";

    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin(input).run();

    assert_eq!(stdout, "A\n");
}

#[test]
fn transpose_rectangular_matrix_3x4() {
    let input = "a\tb\tc\td\n1\t2\t3\t4\n5\t6\t7\t8\n";

    let (stdout, _) = TvaCmd::new().args(&["transpose"]).stdin(input).run();

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
}
