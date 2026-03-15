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

    // type_fn tests
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
    fn test_type_bool_false() {
        let result = type_fn(&[Value::Bool(false)]);
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
    fn test_type_list_with_elements() {
        let result = type_fn(&[Value::List(vec![Value::Int(1), Value::Int(2)])]);
        assert_eq!(result.unwrap(), Value::String("list".to_string()));
    }

    #[test]
    fn test_type_wrong_arity_zero_args() {
        let result = type_fn(&[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "type");
                assert_eq!(expected, 1);
                assert_eq!(got, 0);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    #[test]
    fn test_type_wrong_arity_multiple_args() {
        let result = type_fn(&[Value::Int(1), Value::Int(2)]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "type");
                assert_eq!(expected, 1);
                assert_eq!(got, 2);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    // env_fn tests
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
    fn test_env_empty_string() {
        env::set_var("TVA_EMPTY_VAR", "");
        let result = env_fn(&[Value::String("TVA_EMPTY_VAR".to_string())]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
        env::remove_var("TVA_EMPTY_VAR");
    }

    #[test]
    fn test_env_with_special_chars() {
        env::set_var("TVA_SPECIAL_VAR", "hello\nworld\t!");
        let result = env_fn(&[Value::String("TVA_SPECIAL_VAR".to_string())]);
        assert_eq!(
            result.unwrap(),
            Value::String("hello\nworld\t!".to_string())
        );
        env::remove_var("TVA_SPECIAL_VAR");
    }

    #[test]
    fn test_env_non_string_arg() {
        let result = env_fn(&[Value::Int(42)]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::TypeError(msg) => {
                assert!(msg.contains("requires a string argument"));
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_env_wrong_arity_zero_args() {
        let result = env_fn(&[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "env");
                assert_eq!(expected, 1);
                assert_eq!(got, 0);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    #[test]
    fn test_env_wrong_arity_multiple_args() {
        let result = env_fn(&[
            Value::String("VAR1".to_string()),
            Value::String("VAR2".to_string()),
        ]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "env");
                assert_eq!(expected, 1);
                assert_eq!(got, 2);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    // cwd_fn tests
    #[test]
    fn test_cwd() {
        let result = cwd_fn(&[]);
        assert!(matches!(result, Ok(Value::String(_))));
    }

    #[test]
    fn test_cwd_returns_valid_path() {
        let result = cwd_fn(&[]).unwrap();
        if let Value::String(path) = result {
            // Path should not be empty
            assert!(!path.is_empty());
            // On Windows, path might contain backslashes
            // On Unix, path should contain forward slashes
            assert!(path.contains('/') || path.contains('\\'));
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_cwd_wrong_arity_with_args() {
        let result = cwd_fn(&[Value::String("arg".to_string())]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "cwd");
                assert_eq!(expected, 0);
                assert_eq!(got, 1);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    // version_fn tests
    #[test]
    fn test_version() {
        let result = version_fn(&[]);
        assert!(matches!(result, Ok(Value::String(s)) if !s.is_empty()));
    }

    #[test]
    fn test_version_format() {
        let result = version_fn(&[]).unwrap();
        if let Value::String(version) = result {
            // Version should follow semver format (e.g., "0.2.5")
            assert!(!version.is_empty());
            // Should contain at least one dot
            assert!(version.contains('.'));
            // Should only contain digits and dots
            assert!(version.chars().all(|c| c.is_ascii_digit() || c == '.'));
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_version_wrong_arity_with_args() {
        let result = version_fn(&[Value::String("arg".to_string())]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "version");
                assert_eq!(expected, 0);
                assert_eq!(got, 1);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    // platform_fn tests
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

    #[test]
    fn test_platform_not_empty() {
        let result = platform_fn(&[]).unwrap();
        if let Value::String(s) = result {
            assert!(!s.is_empty());
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_platform_wrong_arity_with_args() {
        let result = platform_fn(&[Value::String("arg".to_string())]);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::WrongArity {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "platform");
                assert_eq!(expected, 0);
                assert_eq!(got, 1);
            }
            _ => panic!("Expected WrongArity error"),
        }
    }

    // Integration tests
    #[test]
    fn test_type_of_env_result() {
        // Test that env() returns a string type when successful
        env::set_var("TVA_TYPE_TEST", "value");
        let env_result = env_fn(&[Value::String("TVA_TYPE_TEST".to_string())]).unwrap();
        let type_result = type_fn(&[env_result]).unwrap();
        assert_eq!(type_result, Value::String("string".to_string()));
        env::remove_var("TVA_TYPE_TEST");
    }

    #[test]
    fn test_type_of_env_nonexistent() {
        // Test that env() returns null type for nonexistent var
        let env_result =
            env_fn(&[Value::String("TVA_NONEXISTENT_99999".to_string())]).unwrap();
        let type_result = type_fn(&[env_result]).unwrap();
        assert_eq!(type_result, Value::String("null".to_string()));
    }

    #[test]
    fn test_type_of_cwd_result() {
        // Test that cwd() returns a string type
        let cwd_result = cwd_fn(&[]).unwrap();
        let type_result = type_fn(&[cwd_result]).unwrap();
        assert_eq!(type_result, Value::String("string".to_string()));
    }

    #[test]
    fn test_type_of_version_result() {
        // Test that version() returns a string type
        let version_result = version_fn(&[]).unwrap();
        let type_result = type_fn(&[version_result]).unwrap();
        assert_eq!(type_result, Value::String("string".to_string()));
    }

    #[test]
    fn test_type_of_platform_result() {
        // Test that platform() returns a string type
        let platform_result = platform_fn(&[]).unwrap();
        let type_result = type_fn(&[platform_result]).unwrap();
        assert_eq!(type_result, Value::String("string".to_string()));
    }
}
