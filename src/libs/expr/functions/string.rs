use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn trim(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().trim().to_string())),
    }
}

pub fn upper(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_uppercase())),
    }
}

pub fn lower(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_lowercase())),
    }
}

pub fn len(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Null => Ok(Value::Int(0)),
        v => Ok(Value::Int(v.to_string().len() as i64)),
    }
}

pub fn substr(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let start = args[1].as_int().unwrap_or(0) as usize;
    let len = args[2].as_int().unwrap_or(s.len() as i64) as usize;

    if start >= s.len() {
        return Ok(Value::String(String::new()));
    }

    let end = (start + len).min(s.len());
    Ok(Value::String(s[start..end].to_string()))
}

pub fn split(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let delim = args[1].as_string();

    let parts: Vec<Value> = s
        .split(&delim)
        .map(|p| Value::String(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

pub fn contains(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let substr = args[1].as_string();
    Ok(Value::Bool(s.contains(&substr)))
}

pub fn starts_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let prefix = args[1].as_string();
    Ok(Value::Bool(s.starts_with(&prefix)))
}

pub fn ends_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let suffix = args[1].as_string();
    Ok(Value::Bool(s.ends_with(&suffix)))
}
