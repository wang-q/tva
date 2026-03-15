use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn print(args: &[Value]) -> Result<Value, EvalError> {
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", parts.join(" "));
    Ok(args.last().unwrap().clone())
}

pub fn eprint(args: &[Value]) -> Result<Value, EvalError> {
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    eprintln!("{}", parts.join(" "));
    Ok(args.last().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        let result = print(&[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_print_multiple_args() {
        let result = print(&[
            Value::String("a".to_string()),
            Value::Int(1),
            Value::Bool(true),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_print_empty_args() {
        // print with no args should panic due to unwrap() on empty slice
        // This documents the current behavior - print requires at least one argument
        let result = std::panic::catch_unwind(|| print(&[]));
        assert!(result.is_err(), "print with no args should panic");
    }

    #[test]
    fn test_print_single_int() {
        let result = print(&[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_print_single_float() {
        let result = print(&[Value::Float(3.14)]);
        assert_eq!(result.unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_print_single_bool() {
        let result = print(&[Value::Bool(false)]);
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_print_null() {
        let result = print(&[Value::Null]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_print_list() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = print(&[list.clone()]);
        assert_eq!(result.unwrap(), list);
    }

    #[test]
    fn test_print_mixed_types() {
        // Test various type combinations
        let result = print(&[
            Value::Null,
            Value::Int(0),
            Value::Float(-1.5),
            Value::Bool(true),
            Value::String("".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_print_special_chars() {
        // Test strings with special characters
        let result = print(&[
            Value::String("hello\nworld\t!".to_string()),
            Value::String("unicode: 你好".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("unicode: 你好".to_string()));
    }

    #[test]
    fn test_eprint() {
        let result = eprint(&[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eprint_empty_args() {
        // eprint with no args should panic due to unwrap() on empty slice
        let result = std::panic::catch_unwind(|| eprint(&[]));
        assert!(result.is_err(), "eprint with no args should panic");
    }

    #[test]
    fn test_eprint_multiple_args() {
        let result = eprint(&[
            Value::String("error".to_string()),
            Value::Int(500),
            Value::Null,
        ]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_eprint_various_types() {
        // Test eprint with various types
        let result = eprint(&[
            Value::Float(f64::NAN),
            Value::Float(f64::INFINITY),
            Value::List(vec![]),
        ]);
        assert_eq!(result.unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_print_eprint_return_last() {
        // Both functions should return the last argument
        let result1 = print(&[Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(result1.unwrap(), Value::Int(3));

        let result2 = eprint(&[
            Value::String("first".to_string()),
            Value::String("last".to_string()),
        ]);
        assert_eq!(result2.unwrap(), Value::String("last".to_string()));
    }

    #[test]
    fn test_print_nested_list() {
        // Test with nested list
        let nested = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::List(vec![Value::Int(3), Value::Int(4)]),
        ]);
        let result = print(&[nested.clone()]);
        assert_eq!(result.unwrap(), nested);
    }
}
