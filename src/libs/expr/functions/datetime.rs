use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use chrono::{DateTime, NaiveDateTime, Utc};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

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
}

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
