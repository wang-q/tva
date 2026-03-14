use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the type name of a value
pub fn type_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            name: "type".to_string(),
            expected: 1,
            got: args.len(),
        });
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

/// Returns an environment variable value
pub fn env_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            name: "env".to_string(),
            expected: 1,
            got: args.len(),
        });
    }
    let name = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(EvalError::TypeError(
                "env() requires a string argument".to_string(),
            ))
        }
    };
    match env::var(name) {
        Ok(val) => Ok(Value::String(val)),
        Err(_) => Ok(Value::Null),
    }
}

/// Returns the current working directory
pub fn cwd_fn(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "cwd".to_string(),
            expected: 0,
            got: args.len(),
        });
    }
    match env::current_dir() {
        Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
        Err(_) => Ok(Value::Null),
    }
}

/// Returns the TVA version
pub fn version_fn(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "version".to_string(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::String(VERSION.to_string()))
}

/// Returns the platform name
pub fn platform_fn(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "platform".to_string(),
            expected: 0,
            got: args.len(),
        });
    }
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };
    Ok(Value::String(platform.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_int() {
        let result = type_fn(&[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::String("int".to_string()));
    }

    #[test]
    fn test_type_float() {
        let result = type_fn(&[Value::Float(3.14)]);
        assert_eq!(result.unwrap(), Value::String("float".to_string()));
    }

    #[test]
    fn test_type_string() {
        let result = type_fn(&[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("string".to_string()));
    }

    #[test]
    fn test_type_bool() {
        let result = type_fn(&[Value::Bool(true)]);
        assert_eq!(result.unwrap(), Value::String("bool".to_string()));
    }

    #[test]
    fn test_type_null() {
        let result = type_fn(&[Value::Null]);
        assert_eq!(result.unwrap(), Value::String("null".to_string()));
    }

    #[test]
    fn test_type_list() {
        let result = type_fn(&[Value::List(vec![])]);
        assert_eq!(result.unwrap(), Value::String("list".to_string()));
    }

    #[test]
    fn test_env_existing() {
        // Test with a commonly available environment variable
        env::set_var("TVA_TEST_VAR", "test_value");
        let result = env_fn(&[Value::String("TVA_TEST_VAR".to_string())]);
        assert_eq!(result.unwrap(), Value::String("test_value".to_string()));
        env::remove_var("TVA_TEST_VAR");
    }

    #[test]
    fn test_env_nonexistent() {
        let result = env_fn(&[Value::String("TVA_NONEXISTENT_VAR_12345".to_string())]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_cwd() {
        let result = cwd_fn(&[]);
        assert!(matches!(result, Ok(Value::String(_))));
    }

    #[test]
    fn test_version() {
        let result = version_fn(&[]);
        assert!(matches!(result, Ok(Value::String(s)) if !s.is_empty()));
    }

    #[test]
    fn test_platform() {
        let result = platform_fn(&[]);
        let platform = result.unwrap();
        match platform {
            Value::String(s) => {
                assert!(
                    s == "windows" || s == "macos" || s == "linux" || s == "unknown",
                    "Unexpected platform: {}",
                    s
                );
            }
            _ => panic!("Expected String, got {:?}", platform),
        }
    }
}
