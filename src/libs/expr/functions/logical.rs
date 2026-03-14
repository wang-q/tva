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
}
