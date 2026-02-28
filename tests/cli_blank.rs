#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn blank_basic() {
    let input = "
a\tb\tc
1\tx\t10
1\ty\t20
2\tx\t30
2\tz\t40
";
    let expected = "
a\tb\tc
1\tx\t10
\ty\t20
2\tx\t30
\tz\t40
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_from_file() {
    let expected = "
a\tb\tc
1\tx\t10
\ty\t20
2\tx\t30
\tz\t40
";
    let (result, _) = TvaCmd::new()
        .args(&[
            "blank",
            "--header",
            "--field",
            "1",
            "tests/data/blank/input1.tsv",
        ])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_no_header() {
    let input = "
1\tx\t10
1\ty\t20
2\tx\t30
2\tz\t40
";
    // Without header, first row is data.
    // Row 1: 1, x, 10
    // Row 2: 1 (same), y, 20 -> blank
    // Row 3: 2, x, 30
    // Row 4: 2 (same), z, 40 -> blank
    let expected = "
1\tx\t10
\ty\t20
2\tx\t30
\tz\t40
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--field", "1"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_multi_file() {
    // input1.tsv ends with: 2, z, 40
    // input2.tsv starts with: 2, z, 50
    // If blanking column 1 ("a") and 2 ("b"):
    // input2 first row: 2==2 -> blank, z==z -> blank.
    let expected = "
a\tb\tc
1\tx\t10
\ty\t20
2\tx\t30
\tz\t40
\t\t50
3\t\t60
\tw\t70
";
    let (result, _) = TvaCmd::new()
        .args(&[
            "blank",
            "--header",
            "--field",
            "1",
            "--field",
            "2",
            "tests/data/blank/input1.tsv",
            "tests/data/blank/input2.tsv",
        ])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_with_replacement() {
    let input = "
a\tb\tc
1\tx\t10
1\ty\t20
2\tx\t30
2\tz\t40
";
    let expected = "
a\tb\tc
1\tx\t10
---\ty\t20
2\tx\t30
---\tz\t40
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1:---"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_multiple_columns() {
    let input = "
g1\tg2\tval
A\tX\t1
A\tX\t2
A\tY\t3
B\tY\t4
B\tY\t5
";
    // Expected logic:
    // Row 1: prev=[A, X], out=A\tX
    // Row 2: cur=[A, X]. A==A -> blank. X==X -> blank. prev=[A, X]. out=\t\t
    // Row 3: cur=[A, Y]. A==A -> blank. Y!=X -> Y. prev=[A, Y]. out=\tY
    // Row 4: cur=[B, Y]. B!=A -> B. Y==Y -> blank. prev=[B, Y]. out=B\t
    // Row 5: cur=[B, Y]. B==B -> blank. Y==Y -> blank. prev=[B, Y]. out=\t\t

    let expected = "
g1\tg2\tval
A\tX\t1
\t\t2
\tY\t3
B\t\t4
\t\t5
";

    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1", "--field", "2"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn blank_ignore_case() {
    let input = "
a
A
a
B
";
    // With -i:
    // Row 1: a
    // Row 2: A == a (case-insensitive) -> blank
    // Row 3: a == A (case-insensitive) -> blank
    // Row 4: B != a -> B

    let expected_case_insensitive = "
a


B
";

    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "--field", "1", "-i"])
        .run();

    assert_eq!(result.trim(), expected_case_insensitive.trim());
}

#[test]
fn blank_mixed_replacements() {
    let input = "
c1\tc2
A\t10
A\t10
B\t10
";
    let expected = "
c1\tc2
A\t10
.\t-
B\t-
";
    // Col 1 replace with ".", Col 2 replace with "-"

    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["blank", "--header", "-f", "1:.", "-f", "2:-"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}
