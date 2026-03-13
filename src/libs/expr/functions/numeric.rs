use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn abs(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        Value::String(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Ok(Value::Int(n.abs()))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Float(f.abs()))
            } else {
                Err(EvalError::TypeError(format!(
                    "abs: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError("abs: cannot convert list to number".to_string())),
    }
}

pub fn round(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.round() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.round() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "round: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError("round: cannot convert list to number".to_string())),
    }
}

pub fn min(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "min".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let mut min_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Int(n) => *n as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                EvalError::TypeError(format!(
                    "min: cannot convert '{}' to number",
                    s
                ))
            })?,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => continue,
            Value::List(_) => continue,
        };
        min_val = Some(min_val.map_or(val, |m| m.min(val)));
    }

    match min_val {
        Some(v) if v == v.floor() => Ok(Value::Int(v as i64)),
        Some(v) => Ok(Value::Float(v)),
        None => Ok(Value::Null),
    }
}

pub fn max(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "max".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let mut max_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Int(n) => *n as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                EvalError::TypeError(format!(
                    "max: cannot convert '{}' to number",
                    s
                ))
            })?,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => continue,
            Value::List(_) => continue,
        };
        max_val = Some(max_val.map_or(val, |m| m.max(val)));
    }

    match max_val {
        Some(v) if v == v.floor() => Ok(Value::Int(v as i64)),
        Some(v) => Ok(Value::Float(v)),
        None => Ok(Value::Null),
    }
}

pub fn int(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Ok(Value::Int(n))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "int: cannot convert '{}' to integer",
                    s
                )))
            }
        }
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::Null => Ok(Value::Null),
        Value::List(_) => Err(EvalError::TypeError("int: cannot convert list to integer".to_string())),
    }
}

pub fn float(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| EvalError::TypeError(format!(
                "float: cannot convert '{}' to float",
                s
            ))),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::Null => Ok(Value::Null),
        Value::List(_) => Err(EvalError::TypeError("float: cannot convert list to float".to_string())),
    }
}

pub fn ceil(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.ceil() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "ceil: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError("ceil: cannot convert list to number".to_string())),
    }
}

pub fn floor(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.floor() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "floor: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError("floor: cannot convert list to number".to_string())),
    }
}
