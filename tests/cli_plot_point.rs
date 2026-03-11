use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::PathBuf;

mod common;
use common::TvaCmd;

fn data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data/plot");
    path.push(filename);
    path
}

#[test]
fn test_plot_point_help() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.args(["plot", "point", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Draw a scatter plot or line chart",
        ))
        .stdout(predicate::str::contains("-x, --x"))
        .stdout(predicate::str::contains("-y, --y"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("-l, --line"));
}

#[test]
fn test_plot_point_basic() {
    let tva = TvaCmd::new();

    // Create a simple TSV file with numeric data
    let input = "x\ty\n1\t2\n2\t4\n3\t6\n4\t8\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    // The command should produce some output (terminal graphics)
    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_with_column_names() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n3\t6\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-x", "x", "-y", "y"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_line() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n3\t6\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-l", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_with_color() {
    let tva = TvaCmd::new();

    let input = "x\ty\tgroup\n1\t2\tA\n2\t4\tA\n3\t6\tB\n4\t8\tB\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2", "--color", "3"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_ignore_errors() {
    let tva = TvaCmd::new();

    // Data with some non-numeric values
    let input = "x\ty\n1\t2\n2\tabc\n3\t6\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2", "--ignore"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_error_on_invalid_data() {
    let tva = TvaCmd::new();

    // Data with non-numeric values and no --ignore flag
    let input = "x\ty\n1\t2\n2\tabc\n3\t6\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("Cannot parse") || stderr.contains("abc"));
}

#[test]
fn test_plot_point_empty_data() {
    let tva = TvaCmd::new();

    // Empty data (only headers)
    let input = "x\ty\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("No valid data points"));
}

#[test]
fn test_plot_point_invalid_column() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "point", "-x", "nonexistent", "-y", "2"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("Invalid X column spec"));
}

#[test]
fn test_plot_point_zero_based_index() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "point", "-x", "0", "-y", "2"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("field index must be >= 1"));
}

// Iris dataset tests
#[test]
fn test_plot_point_iris_basic() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "sepal_length",
            "-y",
            "sepal_width",
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_iris_with_color() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "petal_length",
            "-y",
            "petal_width",
            "--color",
            "label",
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_iris_by_index() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    // Use column indices (1-based)
    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "1", // sepal_length
            "-y",
            "3", // petal_length
            "--color",
            "5", // label
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_iris_with_size() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "sepal_length",
            "-y",
            "petal_length",
            "--color",
            "label",
            "--cols",
            "80",
            "--rows",
            "24",
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
}

// --path parameter tests (geom_path behavior)
#[test]
fn test_plot_point_path_basic() {
    let tva = TvaCmd::new();

    // Trajectory data - order matters
    let input = "x\ty\n0\t0\n1\t1\n0\t2\n-1\t1\n0\t0\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "--path", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_point_path_vs_line() {
    // Data where order matters: a zigzag pattern
    let input = "x\ty\n1\t1\n3\t2\n2\t3\n4\t4\n";

    // --path should preserve order
    let tva = TvaCmd::new();
    let (stdout_path, _stderr) = tva
        .args(&["plot", "point", "--path", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    // --line should sort by x
    let tva = TvaCmd::new();
    let (stdout_line, _stderr) = tva
        .args(&["plot", "point", "--line", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    // Both should produce output
    assert!(!stdout_path.is_empty());
    assert!(!stdout_line.is_empty());
}

#[test]
fn test_plot_point_path_and_line_mutual_exclusion() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "point", "--line", "--path", "-x", "1", "-y", "2"])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("Cannot use both"));
}

// Multi-Y column tests
#[test]
fn test_plot_point_multi_y_basic() {
    let tva = TvaCmd::new();

    // Data with multiple Y columns
    let input = "x\ty1\ty2\n0\t0\t0\n1\t2\t1\n2\t4\t2\n3\t6\t3\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "-x", "1", "-y", "2,3"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
    // Should show two series in legend
    assert!(stdout.contains("y1") || stdout.contains("y2"));
}

#[test]
fn test_plot_point_multi_y_with_names() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    // Plot sepal_length vs both petal_length and petal_width
    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "sepal_length",
            "-y",
            "petal_length,petal_width",
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
    // Should show both column names in legend or axis label
    assert!(stdout.contains("petal_length") || stdout.contains("petal_width"));
}

#[test]
fn test_plot_point_multi_y_with_color() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    // Multiple Y columns with color grouping
    let (stdout, _stderr) = tva
        .args(&[
            "plot",
            "point",
            "-x",
            "sepal_length",
            "-y",
            "petal_length,petal_width",
            "--color",
            "label",
            "--cols",
            "100",
            "--rows",
            "30",
            iris_path.to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.is_empty());
    // Should show axis label with series count
    assert!(stdout.contains("2 series"));
}

// Regression line tests
#[test]
fn test_plot_point_regression_basic() {
    let tva = TvaCmd::new();

    // Linear data: y = 2x
    let input = "x\ty\n0\t0\n1\t2\n2\t4\n3\t6\n4\t8\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "point", "--regression", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
    // Chart renders successfully with regression line
    // Note: Legend only shows when there are multiple datasets
}

#[test]
fn test_plot_point_regression_with_color() {
    let tva = TvaCmd::new();
    let iris_path = data_path("iris.tsv");

    let (stdout, stderr) = tva
        .args(&[
            "plot",
            "point",
            "--regression",
            "-x",
            "sepal_length",
            "-y",
            "petal_length",
            "--color",
            "label",
            iris_path.to_str().unwrap(),
        ])
        .run();

    if stdout.is_empty() {
        eprintln!("stderr: {}", stderr);
    }
    assert!(!stdout.is_empty());
    // Should show regression equations in legend (y = mx + b format)
    assert!(stdout.contains("y =") && stdout.contains("x +"));
}

#[test]
fn test_plot_point_regression_and_line_mutual_exclusion() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n";

    let (_stdout, stderr) = tva
        .args(&[
            "plot",
            "point",
            "--regression",
            "--line",
            "-x",
            "1",
            "-y",
            "2",
        ])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("Cannot use"));
}

#[test]
fn test_plot_point_regression_and_path_mutual_exclusion() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n";

    let (_stdout, stderr) = tva
        .args(&[
            "plot",
            "point",
            "--regression",
            "--path",
            "-x",
            "1",
            "-y",
            "2",
        ])
        .stdin(input)
        .run_fail();

    assert!(stderr.contains("Cannot use"));
}
