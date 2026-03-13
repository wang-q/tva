use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use regex::Regex;

pub fn regex_match(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();

    match Regex::new(&pattern) {
        Ok(re) => Ok(Value::Bool(re.is_match(&s))),
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_match: invalid pattern: {}",
            e
        ))),
    }
}

pub fn regex_extract(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();
    let group = if args.len() > 2 {
        match &args[2] {
            Value::Int(n) => *n as usize,
            Value::Float(f) => f.round() as usize,
            v => return Err(EvalError::TypeError(format!(
                "regex_extract: group must be a number, got {}",
                v.type_name()
            ))),
        }
    } else {
        0
    };

    match Regex::new(&pattern) {
        Ok(re) => {
            if let Some(caps) = re.captures(&s) {
                if let Some(m) = caps.get(group) {
                    Ok(Value::String(m.as_str().to_string()))
                } else {
                    Ok(Value::Null)
                }
            } else {
                Ok(Value::Null)
            }
        }
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_extract: invalid pattern: {}",
            e
        ))),
    }
}

pub fn regex_replace(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();
    let replacement = args[2].as_string();

    match Regex::new(&pattern) {
        Ok(re) => Ok(Value::String(re.replace_all(&s, &replacement).to_string())),
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_replace: invalid pattern: {}",
            e
        ))),
    }
}
