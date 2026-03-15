use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

mod common;
use common::TvaCmd;

#[test]
fn test_plot_bin2d_help() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.args(["plot", "bin2d", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Draw a 2D binning heatmap"))
        .stdout(predicate::str::contains("-x, --x"))
        .stdout(predicate::str::contains("-y, --y"))
        .stdout(predicate::str::contains("-b, --bins"))
        .stdout(predicate::str::contains("--binwidth"));
}

#[test]
fn test_plot_bin2d_basic() {
    let tva = TvaCmd::new();

    // Create a simple TSV file with numeric data
    let input = "x\ty\n1\t2\n2\t4\n3\t6\n4\t8\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    // The command should produce some output (terminal graphics)
    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_with_column_names() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t2\n2\t4\n3\t6\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "x", "-y", "y"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_with_bins() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t1\n2\t2\n3\t3\n4\t4\n5\t5\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2", "-b", "10"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_with_different_bins() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t1\n2\t2\n3\t3\n4\t4\n5\t5\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2", "-b", "5,10"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_with_binwidth() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t1\n2\t2\n3\t3\n4\t4\n5\t5\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2", "--binwidth", "1.0"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_ignore_errors() {
    let tva = TvaCmd::new();

    // Data with some non-numeric values
    let input = "x\ty\n1\t2\n2\tabc\n3\t6\n";

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2", "--ignore"])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_clustered_data() {
    let tva = TvaCmd::new();

    // Data clustered in two regions
    let mut input = String::from("x\ty\n");
    for _ in 0..20 {
        input.push_str("1\t1\n");
        input.push_str("1\t2\n");
    }
    for _ in 0..5 {
        input.push_str("10\t10\n");
    }

    let (stdout, _stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2", "-b", "5"])
        .stdin(&input)
        .run();

    assert!(!stdout.is_empty());
}

#[test]
fn test_plot_bin2d_empty_data() {
    let tva = TvaCmd::new();

    // Only header, no data
    let input = "x\ty\n";

    let (_stdout, stderr) = tva
        .args(&["plot", "bin2d", "-x", "1", "-y", "2"])
        .stdin(input)
        .run();

    // Should report no valid data
    assert!(stderr.contains("No valid data") || stderr.contains("Cannot parse"));
}

#[test]
fn test_plot_bin2d_with_custom_dimensions() {
    let tva = TvaCmd::new();

    let input = "x\ty\n1\t1\n2\t2\n3\t3\n";

    let (stdout, _stderr) = tva
        .args(&[
            "plot", "bin2d", "-x", "1", "-y", "2", "--cols", "60", "--rows", "15",
        ])
        .stdin(input)
        .run();

    assert!(!stdout.is_empty());
}
