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
        Ok(ndt) => Ok(Value::DateTime(DateTime::from_naive_utc_and_offset(ndt, Utc))),
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
                    Ok(Value::String(dt.with_timezone(&Utc).format(&fmt).to_string()))
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
