#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn md_basic() {
    let expected = "| H1  | H2  |\n| --- | --- |\n| A   | 1   |\n| B   | 2   |\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["md"])
        .stdin("H1\tH2\nA\t1\nB\t2\n")
        .run();

    // The markdown formatter aligns columns
    assert_eq!(stdout, expected);
}

#[test]
fn md_center() {
    let expected =
        "|  H1   | H2  |\n| :---: | --- |\n|   A   | 1   |\n|   B   | 2   |\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["md", "--center", "1"])
        .stdin("H1\tH2\nA\t1\nB\t2\n")
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn md_right() {
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   |    1 |\n| B   |    2 |\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["md", "--right", "2"])
        .stdin("H1\tH2\nA\t1\nB\t2\n")
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn md_num() {
    // H2 is numeric, so it should be right-aligned
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   |    1 |\n| B   |    2 |\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["md", "--num"])
        .stdin("H1\tH2\nA\t1\nB\t2\n")
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn md_fmt() {
    // H2 is numeric, should be right-aligned and formatted
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   | 1.00 |\n| B   | 2.57 |\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["md", "--fmt", "--digits", "2"])
        .stdin("H1\tH2\nA\t1\nB\t2.567\n")
        .run();

    assert_eq!(stdout, expected);
}
