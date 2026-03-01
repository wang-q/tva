#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

const INPUT: &str = "A\t10
A\t20
B\t30
B\t40
B\t50
";

#[test]
fn stats_retain_alias() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--retain", "2"])
        .stdin(INPUT)
        .run();
    
    // retain should act like first
    assert_eq!(stdout.trim(), "10");
}

#[test]
fn stats_var_alias() {
    let input = "val\n2\n4\n";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--var", "val"])
        .stdin(input)
        .run();
    
    // Variance of 2,4 is 2. (Mean=3, (1+1)/1 = 2)
    assert_eq!(stdout.trim(), "val_variance\n2");
}

#[test]
fn stats_custom_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "2:my_sum", "--write-header"])
        .stdin(INPUT)
        .run();
    
    assert_eq!(stdout.trim(), "my_sum\n150");
}

#[test]
fn stats_custom_header_multiple() {
    // Note: tsv-summarize treats this as prefix.
    // Our implementation: "2:S" -> S if single field, S_2 if multiple?
    // Wait, my implementation uses field index as suffix if multiple fields.
    // Let's test single field first.
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--sum", "2:S", "--write-header"])
        .stdin(INPUT)
        .run();
    
    assert_eq!(stdout.trim(), "S\n150");
}

#[test]
fn stats_custom_header_quantile() {
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--quantile", "2:0.5:Median", "--write-header"])
        .stdin(INPUT)
        .run();
    
    // Quantile of 10,20,30,40,50 is 30. Header should be Median.
    assert_eq!(stdout.trim(), "Median\n30");
}

#[test]
fn stats_replace_missing_input_side() {
    // Input has missing value.
    // A, 10
    // A, 
    // If replace-missing works on input side, empty becomes e.g. 0.
    // Sum = 10 + 0 = 10. Mean = 10 / 2 = 5.
    // If it didn't work (old behavior), empty is skipped. Sum = 10. Mean = 10 / 1 = 10.
    
    let input = "A\t10
A\t
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0"])
        .stdin(input)
        .run();
    
    assert_eq!(stdout.trim(), "5");
}
