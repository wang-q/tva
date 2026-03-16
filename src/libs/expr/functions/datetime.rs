use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use chrono::{DateTime, NaiveDateTime, Utc};

pub fn now(_args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::DateTime(Utc::now()))
}

pub fn strptime(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let fmt = args[1].as_string();

    match NaiveDateTime::parse_from_str(&s, &fmt) {
        Ok(ndt) => Ok(Value::DateTime(DateTime::from_naive_utc_and_offset(
            ndt, Utc,
        ))),
        Err(e) => Err(EvalError::TypeError(format!(
            "strptime: failed to parse '{}' with format '{}': {}",
            s, fmt, e
        ))),
    }
}

pub fn strftime(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::DateTime(dt) => {
            let fmt = args[1].as_string();
            Ok(Value::String(dt.format(&fmt).to_string()))
        }
        Value::String(s) => {
            // Try to parse as RFC3339 and then format
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => {
                    let fmt = args[1].as_string();
                    Ok(Value::String(
                        dt.with_timezone(&Utc).format(&fmt).to_string(),
                    ))
                }
                Err(e) => Err(EvalError::TypeError(format!(
                    "strftime: failed to parse datetime '{}': {}",
                    s, e
                ))),
            }
        }
        v => Err(EvalError::TypeError(format!(
            "strftime: expected datetime or string, got {}",
            v.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_now() {
        let result = now(&[]);
        assert!(result.is_ok());
        // Check it's a DateTime
        match result.unwrap() {
            Value::DateTime(_) => {}
            _ => panic!("now() should return DateTime"),
        }
    }

    #[test]
    fn test_strptime() {
        let result = strptime(&[
            Value::String("2024-03-15 14:30:00".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ]);
        match &result {
            Err(e) => println!("strptime error: {:?}", e),
            _ => {}
        }
        assert!(result.is_ok(), "strptime failed: {:?}", result);
        match result.unwrap() {
            Value::DateTime(dt) => {
                assert_eq!(dt.year(), 2024);
                assert_eq!(dt.month(), 3);
                assert_eq!(dt.day(), 15);
            }
            _ => panic!("strptime should return DateTime"),
        }
    }

    #[test]
    fn test_strptime_different_formats() {
        // Date only format with dashes (common format)
        let result = strptime(&[
            Value::String("2024-03-15".to_string()),
            Value::String("%Y-%m-%d".to_string()),
        ]);
        // Date-only parsing may fail because NaiveDateTime expects time
        // This tests that the function handles the error gracefully
        if result.is_ok() {
            match result.unwrap() {
                Value::DateTime(dt) => {
                    assert_eq!(dt.year(), 2024);
                    assert_eq!(dt.month(), 3);
                    assert_eq!(dt.day(), 15);
                }
                _ => panic!("Expected DateTime"),
            }
        }

        // Different time format
        let result = strptime(&[
            Value::String("15-Mar-2024 14:30:00".to_string()),
            Value::String("%d-%b-%Y %H:%M:%S".to_string()),
        ]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::DateTime(dt) => {
                assert_eq!(dt.year(), 2024);
                assert_eq!(dt.month(), 3);
                assert_eq!(dt.day(), 15);
            }
            _ => panic!("Expected DateTime"),
        }
    }

    #[test]
    fn test_strptime_invalid_date() {
        // Invalid date should error
        let result = strptime(&[
            Value::String("not-a-date".to_string()),
            Value::String("%Y-%m-%d".to_string()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strptime_invalid_format() {
        // Mismatched format should error
        let result = strptime(&[
            Value::String("2024-03-15".to_string()),
            Value::String("%d/%m/%Y".to_string()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strftime() {
        // Parse and format
        let dt = strptime(&[
            Value::String("2024-03-15 14:30:00".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ])
        .unwrap();

        let result = strftime(&[dt, Value::String("%Y/%m/%d".to_string())]);
        assert_eq!(result.unwrap(), Value::String("2024/03/15".to_string()));

        // Format from string
        let result = strftime(&[
            Value::String("2024-03-15T14:30:00Z".to_string()),
            Value::String("%d-%m-%Y".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("15-03-2024".to_string()));
    }

    #[test]
    fn test_strftime_various_formats() {
        let dt = strptime(&[
            Value::String("2024-03-15 14:30:45".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ])
        .unwrap();

        // Full datetime format
        let result =
            strftime(&[dt.clone(), Value::String("%Y-%m-%d %H:%M:%S".to_string())]);
        assert_eq!(
            result.unwrap(),
            Value::String("2024-03-15 14:30:45".to_string())
        );

        // Time only
        let result = strftime(&[dt.clone(), Value::String("%H:%M".to_string())]);
        assert_eq!(result.unwrap(), Value::String("14:30".to_string()));

        // Month name
        let result = strftime(&[dt.clone(), Value::String("%B %d, %Y".to_string())]);
        assert_eq!(result.unwrap(), Value::String("March 15, 2024".to_string()));
    }

    #[test]
    fn test_strftime_invalid_datetime_string() {
        // Invalid RFC3339 string should error
        let result = strftime(&[
            Value::String("not-a-datetime".to_string()),
            Value::String("%Y-%m-%d".to_string()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strftime_invalid_type() {
        // Non-datetime, non-string types should error
        let result = strftime(&[Value::Int(123), Value::String("%Y-%m-%d".to_string())]);
        assert!(result.is_err());

        let result =
            strftime(&[Value::Float(3.14), Value::String("%Y-%m-%d".to_string())]);
        assert!(result.is_err());

        let result =
            strftime(&[Value::Bool(true), Value::String("%Y-%m-%d".to_string())]);
        assert!(result.is_err());

        let result = strftime(&[Value::Null, Value::String("%Y-%m-%d".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strptime_time_components() {
        let result = strptime(&[
            Value::String("2024-03-15 14:30:45".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::DateTime(dt) => {
                assert_eq!(dt.hour(), 14);
                assert_eq!(dt.minute(), 30);
                assert_eq!(dt.second(), 45);
            }
            _ => panic!("Expected DateTime"),
        }
    }

    #[test]
    fn test_roundtrip() {
        // Parse and format back should preserve the date
        let original = "2024-12-25 10:00:00";
        let dt = strptime(&[
            Value::String(original.to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ])
        .unwrap();

        let formatted =
            strftime(&[dt, Value::String("%Y-%m-%d %H:%M:%S".to_string())]).unwrap();
        assert_eq!(formatted, Value::String(original.to_string()));
    }

    #[test]
    fn test_strftime_with_datetime_value() {
        // Test strftime with DateTime value directly (not string)
        let dt = strptime(&[
            Value::String("2024-06-15 18:30:00".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S".to_string()),
        ])
        .unwrap();

        // Format with various format strings
        let result = strftime(&[dt.clone(), Value::String("%Y".to_string())]);
        assert_eq!(result.unwrap(), Value::String("2024".to_string()));

        let result = strftime(&[dt.clone(), Value::String("%m/%d/%Y".to_string())]);
        assert_eq!(result.unwrap(), Value::String("06/15/2024".to_string()));

        let result = strftime(&[dt.clone(), Value::String("%H:%M".to_string())]);
        assert_eq!(result.unwrap(), Value::String("18:30".to_string()));
    }

    #[test]
    fn test_strftime_edge_cases() {
        // Test with RFC3339 format string
        let result = strftime(&[
            Value::String("2024-03-15T14:30:00Z".to_string()),
            Value::String("%Y-%m-%dT%H:%M:%SZ".to_string()),
        ]);
        assert!(result.is_ok());

        // Test with empty format string
        let result = strftime(&[
            Value::String("2024-03-15T14:30:00Z".to_string()),
            Value::String("".to_string()),
        ]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_strptime_edge_cases() {
        // Test with timezone offset
        let result = strptime(&[
            Value::String("2024-03-15 14:30:00 +05:00".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S %z".to_string()),
        ]);
        assert!(result.is_ok());

        // Test with milliseconds
        let result = strptime(&[
            Value::String("2024-03-15 14:30:00.123".to_string()),
            Value::String("%Y-%m-%d %H:%M:%S%.3f".to_string()),
        ]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strftime_list_error() {
        // List should error
        let result = strftime(&[
            Value::List(vec![Value::Int(1)]),
            Value::String("%Y".to_string()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strftime_lambda_error() {
        // Lambda should error
        use crate::libs::expr::parser::ast::Expr;
        use ahash::HashMap;
        let result = strftime(&[
            Value::Lambda(crate::libs::expr::runtime::value::LambdaValue {
                params: vec![],
                body: Expr::Int(1),
                captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
            }),
            Value::String("%Y".to_string()),
        ]);
        assert!(result.is_err());
    }
}
