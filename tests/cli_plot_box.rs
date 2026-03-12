use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::str::contains;

fn new_cmd() -> Command {
    cargo_bin_cmd!("tva")
}

#[test]
fn test_plot_box_basic() {
    let mut cmd = new_cmd();

    let input = "value\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10";

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("value")
        .write_stdin(input);

    cmd.assert().success();
}

#[test]
fn test_plot_box_with_color() {
    let mut cmd = new_cmd();

    let input = "group\tvalue\nA\t1\nA\t2\nA\t3\nB\t8\nB\t9\nB\t10";

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("value")
        .arg("--color")
        .arg("group")
        .write_stdin(input);

    cmd.assert().success();
}

#[test]
fn test_plot_box_with_outliers() {
    let mut cmd = new_cmd();

    let input = "value\n1\n2\n3\n4\n5\n6\n7\n8\n9\n100"; // 100 is an outlier

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("value")
        .arg("--outliers")
        .write_stdin(input);

    cmd.assert().success();
}

#[test]
fn test_plot_box_ignore_errors() {
    let mut cmd = new_cmd();

    let input = "value\n1\n2\ninvalid\n4\n5";

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("value")
        .arg("--ignore")
        .write_stdin(input);

    cmd.assert().success();
}

#[test]
fn test_plot_box_error_no_data() {
    let mut cmd = new_cmd();

    let input = "value\ninvalid\nalso_invalid";

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("value")
        .write_stdin(input);

    cmd.assert().failure().stderr(contains("Cannot parse"));
}

#[test]
fn test_plot_box_multiple_y_columns() {
    let mut cmd = new_cmd();

    let input = "x\ty\tz\n1\t2\t3\n4\t5\t6\n7\t8\t9";

    cmd.arg("plot")
        .arg("box")
        .arg("-y")
        .arg("y,z")
        .write_stdin(input);

    cmd.assert().success();
}

#[test]
fn test_plot_box_help() {
    let mut cmd = new_cmd();

    cmd.arg("plot").arg("box").arg("--help");

    cmd.assert().success().stdout(contains("box plot"));
}
