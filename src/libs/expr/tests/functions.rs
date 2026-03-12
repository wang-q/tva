use crate::libs::expr::eval_expr;

fn row(data: &[&str]) -> Vec<String> {
    data.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_string_trim() {
    let r = row(&["  hello  "]);
    assert_eq!(
        eval_expr("trim(@1)", &r, None).unwrap().to_string(),
        "hello"
    );
}

#[test]
fn test_string_upper() {
    let r = row(&["hello"]);
    assert_eq!(
        eval_expr("upper(@1)", &r, None).unwrap().to_string(),
        "HELLO"
    );
}

#[test]
fn test_string_lower() {
    let r = row(&["HELLO"]);
    assert_eq!(
        eval_expr("lower(@1)", &r, None).unwrap().to_string(),
        "hello"
    );
}

#[test]
fn test_string_len() {
    let r = row(&["hello"]);
    assert_eq!(eval_expr("len(@1)", &r, None).unwrap().to_string(), "5");
}

#[test]
fn test_numeric_abs() {
    let r = row(&["-5"]);
    assert_eq!(eval_expr("abs(@1)", &r, None).unwrap().to_string(), "5");

    let r = row(&["-3.14"]);
    assert_eq!(eval_expr("abs(@1)", &r, None).unwrap().to_string(), "3.14");
}

#[test]
fn test_numeric_round() {
    let r = row(&["3.7"]);
    assert_eq!(eval_expr("round(@1)", &r, None).unwrap().to_string(), "4");

    let r = row(&["3.2"]);
    assert_eq!(eval_expr("round(@1)", &r, None).unwrap().to_string(), "3");
}

#[test]
fn test_numeric_min() {
    let r: Vec<String> = vec![];
    assert_eq!(
        eval_expr("min(3, 1, 2)", &r, None).unwrap().to_string(),
        "1"
    );

    assert_eq!(eval_expr("min(10, 5)", &r, None).unwrap().to_string(), "5");
}

#[test]
fn test_numeric_max() {
    let r: Vec<String> = vec![];
    assert_eq!(
        eval_expr("max(3, 1, 2)", &r, None).unwrap().to_string(),
        "3"
    );

    assert_eq!(eval_expr("max(10, 5)", &r, None).unwrap().to_string(), "10");
}

#[test]
fn test_logical_if() {
    let r: Vec<String> = vec![];
    assert_eq!(
        eval_expr("if(true, 1, 0)", &r, None).unwrap().to_string(),
        "1"
    );

    assert_eq!(
        eval_expr("if(false, 1, 0)", &r, None).unwrap().to_string(),
        "0"
    );

    // Using expression as condition
    let r = row(&["10", "5"]);
    assert_eq!(
        eval_expr("if(@1 > @2, \"greater\", \"less or equal\")", &r, None)
            .unwrap()
            .to_string(),
        "greater"
    );
}

#[test]
fn test_logical_default() {
    let r: Vec<String> = vec![];
    assert_eq!(
        eval_expr("default(null, \"fallback\")", &r, None)
            .unwrap()
            .to_string(),
        "fallback"
    );

    assert_eq!(
        eval_expr("default(\"value\", \"fallback\")", &r, None)
            .unwrap()
            .to_string(),
        "value"
    );
}

#[test]
fn test_nested_function_calls() {
    let r = row(&["  hello  "]);
    assert_eq!(
        eval_expr("upper(trim(@1))", &r, None).unwrap().to_string(),
        "HELLO"
    );
}

#[test]
fn test_function_with_column_ref() {
    let r = row(&["  alice  "]);
    let headers = vec!["name".to_string()];
    assert_eq!(
        eval_expr("upper(trim(@name))", &r, Some(&headers))
            .unwrap()
            .to_string(),
        "ALICE"
    );
}

#[test]
fn test_unknown_function_error() {
    let r: Vec<String> = vec![];
    let result = eval_expr("unknown(1)", &r, None);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Unknown function"));
}

#[test]
fn test_wrong_arity_error() {
    let r: Vec<String> = vec![];
    let result = eval_expr("trim()", &r, None);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expected"));
}
