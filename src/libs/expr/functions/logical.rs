use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn if_fn(args: &[Value]) -> Result<Value, EvalError> {
    let condition = args[0].as_bool();
    if condition {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

pub fn default_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args[0].is_null() || args[0].as_bool() == false {
        Ok(args[1].clone())
    } else {
        Ok(args[0].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_fn() {
        assert_eq!(
            if_fn(&[Value::Bool(true), Value::Int(1), Value::Int(0)]).unwrap(),
            Value::Int(1)
        );
        assert_eq!(
            if_fn(&[Value::Bool(false), Value::Int(1), Value::Int(0)]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_if_fn_different_types() {
        // if can return different types in branches
        assert_eq!(
            if_fn(&[
                Value::Bool(true),
                Value::String("yes".to_string()),
                Value::Null
            ])
            .unwrap(),
            Value::String("yes".to_string())
        );
        assert_eq!(
            if_fn(&[Value::Bool(false), Value::Int(1), Value::Float(2.5)]).unwrap(),
            Value::Float(2.5)
        );
    }

    #[test]
    fn test_if_fn_truthy_values() {
        // Non-boolean values should be converted to bool via as_bool()
        // Int non-zero is true
        assert_eq!(
            if_fn(&[
                Value::Int(1),
                Value::String("true".to_string()),
                Value::String("false".to_string())
            ])
            .unwrap(),
            Value::String("true".to_string())
        );
        // Int zero is false
        assert_eq!(
            if_fn(&[
                Value::Int(0),
                Value::String("true".to_string()),
                Value::String("false".to_string())
            ])
            .unwrap(),
            Value::String("false".to_string())
        );
        // Non-empty string is true
        assert_eq!(
            if_fn(&[
                Value::String("hello".to_string()),
                Value::Int(1),
                Value::Int(0)
            ])
            .unwrap(),
            Value::Int(1)
        );
        // Empty string is false
        assert_eq!(
            if_fn(&[Value::String("".to_string()), Value::Int(1), Value::Int(0)])
                .unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_if_fn_with_floats() {
        // Non-zero float is true
        assert_eq!(
            if_fn(&[Value::Float(0.1), Value::Int(1), Value::Int(0)]).unwrap(),
            Value::Int(1)
        );
        // Zero float is false
        assert_eq!(
            if_fn(&[Value::Float(0.0), Value::Int(1), Value::Int(0)]).unwrap(),
            Value::Int(0)
        );
        // Negative float is true
        assert_eq!(
            if_fn(&[Value::Float(-1.5), Value::Int(1), Value::Int(0)]).unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_if_fn_with_lists() {
        // Non-empty list is true
        assert_eq!(
            if_fn(&[
                Value::List(vec![Value::Int(1), Value::Int(2)]),
                Value::String("true".to_string()),
                Value::String("false".to_string())
            ])
            .unwrap(),
            Value::String("true".to_string())
        );
        // Empty list is false
        assert_eq!(
            if_fn(&[
                Value::List(vec![]),
                Value::String("true".to_string()),
                Value::String("false".to_string())
            ])
            .unwrap(),
            Value::String("false".to_string())
        );
    }

    #[test]
    fn test_if_fn_nested() {
        // Nested if expressions
        let result = if_fn(&[
            Value::Bool(true),
            if_fn(&[Value::Bool(false), Value::Int(1), Value::Int(2)]).unwrap(),
            Value::Int(3),
        ]);
        assert_eq!(result.unwrap(), Value::Int(2));
    }

    #[test]
    fn test_default_fn() {
        // null returns fallback
        assert_eq!(
            default_fn(&[Value::Null, Value::String("fallback".to_string())]).unwrap(),
            Value::String("fallback".to_string())
        );
        // false returns fallback
        assert_eq!(
            default_fn(&[Value::Bool(false), Value::Int(0)]).unwrap(),
            Value::Int(0)
        );
        // non-null returns original
        assert_eq!(
            default_fn(&[
                Value::String("value".to_string()),
                Value::String("fallback".to_string()),
            ])
            .unwrap(),
            Value::String("value".to_string())
        );
    }

    #[test]
    fn test_default_fn_with_truthy_values() {
        // true returns original
        assert_eq!(
            default_fn(&[Value::Bool(true), Value::String("fallback".to_string())])
                .unwrap(),
            Value::Bool(true)
        );
        // Non-zero int returns original
        assert_eq!(
            default_fn(&[Value::Int(42), Value::Int(0)]).unwrap(),
            Value::Int(42)
        );
        // Zero int returns fallback (treated as falsy)
        assert_eq!(
            default_fn(&[Value::Int(0), Value::Int(999)]).unwrap(),
            Value::Int(999)
        );
    }

    #[test]
    fn test_default_fn_with_floats() {
        // Non-zero float returns original
        assert_eq!(
            default_fn(&[Value::Float(3.14), Value::Float(0.0)]).unwrap(),
            Value::Float(3.14)
        );
        // Zero float returns fallback
        assert_eq!(
            default_fn(&[Value::Float(0.0), Value::Float(999.0)]).unwrap(),
            Value::Float(999.0)
        );
    }

    #[test]
    fn test_default_fn_with_strings() {
        // Non-empty string returns original
        assert_eq!(
            default_fn(&[
                Value::String("hello".to_string()),
                Value::String("fallback".to_string())
            ])
            .unwrap(),
            Value::String("hello".to_string())
        );
        // Empty string returns fallback (treated as falsy)
        assert_eq!(
            default_fn(&[
                Value::String("".to_string()),
                Value::String("fallback".to_string())
            ])
            .unwrap(),
            Value::String("fallback".to_string())
        );
    }

    #[test]
    fn test_default_fn_with_lists() {
        // Non-empty list returns original
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(default_fn(&[list.clone(), Value::Null]).unwrap(), list);
        // Empty list returns fallback
        assert_eq!(
            default_fn(&[Value::List(vec![]), Value::String("empty".to_string())])
                .unwrap(),
            Value::String("empty".to_string())
        );
    }

    #[test]
    fn test_default_fn_chaining() {
        // Chain multiple defaults
        let result = default_fn(&[
            Value::Null,
            default_fn(&[Value::Null, Value::String("final".to_string())]).unwrap(),
        ]);
        assert_eq!(result.unwrap(), Value::String("final".to_string()));

        // First non-null wins
        let result = default_fn(&[
            Value::Null,
            default_fn(&[Value::Int(42), Value::String("final".to_string())]).unwrap(),
        ]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_default_fn_same_type() {
        // Both value and fallback are same type
        assert_eq!(
            default_fn(&[Value::Int(10), Value::Int(20)]).unwrap(),
            Value::Int(10)
        );
        assert_eq!(
            default_fn(&[Value::Int(0), Value::Int(20)]).unwrap(),
            Value::Int(20)
        );
    }
}
