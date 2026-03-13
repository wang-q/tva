use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn join(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let sep = args[1].as_string();
            let parts: Vec<String> = list.iter().map(|v| v.to_string()).collect();
            Ok(Value::String(parts.join(&sep)))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("join: first argument must be a list".to_string())),
    }
}

pub fn first(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            if let Some(first) = list.first() {
                Ok(first.clone())
            } else {
                Ok(Value::Null)
            }
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("first: argument must be a list".to_string())),
    }
}

pub fn last(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            if let Some(last) = list.last() {
                Ok(last.clone())
            } else {
                Ok(Value::Null)
            }
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("last: argument must be a list".to_string())),
    }
}

pub fn reverse(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut reversed = list.clone();
            reversed.reverse();
            Ok(Value::List(reversed))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("reverse: argument must be a list".to_string())),
    }
}

pub fn nth(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let n = match &args[1] {
                Value::Int(i) => *i as usize,
                Value::Float(f) => f.round() as usize,
                v => return Err(EvalError::TypeError(format!(
                    "nth: index must be a number, got {}",
                    v.type_name()
                ))),
            };
            // Support negative indexing like Python
            let idx = if n >= list.len() {
                return Ok(Value::Null);
            } else {
                n
            };
            Ok(list.get(idx).cloned().unwrap_or(Value::Null))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("nth: first argument must be a list".to_string())),
    }
}

pub fn sort(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut sorted = list.clone();
            sorted.sort_by(|a, b| {
                // Try numeric comparison first
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
                    // Fall back to string comparison
                    _ => a.to_string().cmp(&b.to_string()),
                }
            });
            Ok(Value::List(sorted))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("sort: argument must be a list".to_string())),
    }
}

pub fn unique(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for item in list {
                // Use string representation for comparison
                let key = item.to_string();
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("unique: argument must be a list".to_string())),
    }
}

pub fn slice(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let start = match &args[1] {
                Value::Int(i) => *i as usize,
                Value::Float(f) => f.round() as usize,
                v => return Err(EvalError::TypeError(format!(
                    "slice: start must be a number, got {}",
                    v.type_name()
                ))),
            };
            let end = if args.len() > 2 {
                match &args[2] {
                    Value::Int(i) => Some(*i as usize),
                    Value::Float(f) => Some(f.round() as usize),
                    v => return Err(EvalError::TypeError(format!(
                        "slice: end must be a number, got {}",
                        v.type_name()
                    ))),
                }
            } else {
                None
            };

            let start = start.min(list.len());
            let end = end.map(|e| e.min(list.len())).unwrap_or(list.len());
            let end = if end < start { start } else { end };

            Ok(Value::List(list[start..end].to_vec()))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError("slice: first argument must be a list".to_string())),
    }
}
