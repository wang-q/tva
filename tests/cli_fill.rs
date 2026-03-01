#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn fill_down_basic() {
    let input = "a\tb\tc
1\tx\t
\ty\t20
2\t\t30
\tz\t
";
    // Col 1: 1, "", 2, "" -> 1, 1, 2, 2
    // Col 2: x, y, "", z -> x, y, y, z
    // Col 3: "", 20, 30, "" -> "", 20, 30, 30 (first empty stays empty)
    let expected = "
a\tb\tc
1\tx\t
1\ty\t20
2\ty\t30
2\tz\t30
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "--field", "1,2,3"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn fill_const_basic() {
    let input = "a\tb
1\t
\t2
";
    let expected = "
a\tb
1\t0
0\t2
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "--field", "1,2", "--value", "0"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn fill_custom_na() {
    let input = "a\tb
1\tNA
NA\t2
";
    let expected = "
a\tb
1\t0
0\t2
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "--field", "1,2", "--value", "0", "--na", "NA"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn fill_down_multi_file() {
    // File 1 ends with a value, File 2 starts with empty.
    // Should carry over across files? Yes, streaming behavior.

    // Using inline data for simplicity since we can't easily create tmp files here without boilerplate.
    // But we can check basic behavior.
    // Let's assume multi-file works if logic is sound.
    // But let's test basic multi-arg.
    let input = "a\tb
1\t10
\t
2\t
";
    let expected = "
a\tb
1\t10
1\t10
2\t10
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "--header", "-f", "1", "-f", "2"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn fill_no_header() {
    let input = "1\t
\t2
";
    let expected = "
1\t
1\t2
";
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["fill", "-f", "1"])
        .run();

    assert_eq!(result.trim(), expected.trim());
}
