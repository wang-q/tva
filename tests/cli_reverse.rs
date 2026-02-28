#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn reverse_basic() {
    let input = "1\n2\n3\n";
    let expected = "3\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_header() {
    let input = "H\n1\n2\n3\n";
    let expected = "H\n3\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse", "--header"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}

#[test]
fn reverse_stdin() {
    let input = "1\n2\n3\n";
    let expected = "3\n2\n1\n";

    let (stdout, _) = TvaCmd::new()
        .args(&["reverse"])
        .stdin(input)
        .run();

    assert_eq!(stdout, expected);
}
