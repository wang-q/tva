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
        Value::List(_) => Err(EvalError::TypeError(
            "abs: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "abs: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "abs: cannot convert lambda to number".to_string(),
        )),
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
        Value::List(_) => Err(EvalError::TypeError(
            "round: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "round: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "round: cannot convert lambda to number".to_string(),
        )),
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
                EvalError::TypeError(format!("min: cannot convert '{}' to number", s))
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
            Value::DateTime(_) => continue,
            Value::Lambda(_) => continue,
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
                EvalError::TypeError(format!("max: cannot convert '{}' to number", s))
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
            Value::DateTime(_) => continue,
            Value::Lambda(_) => continue,
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
        Value::List(_) => Err(EvalError::TypeError(
            "int: cannot convert list to integer".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "int: cannot convert datetime to integer".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "int: cannot convert lambda to integer".to_string(),
        )),
    }
}

pub fn float(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            EvalError::TypeError(format!("float: cannot convert '{}' to float", s))
        }),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::Null => Ok(Value::Null),
        Value::List(_) => Err(EvalError::TypeError(
            "float: cannot convert list to float".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "float: cannot convert datetime to float".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "float: cannot convert lambda to float".to_string(),
        )),
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
        Value::List(_) => Err(EvalError::TypeError(
            "ceil: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "ceil: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "ceil: cannot convert lambda to number".to_string(),
        )),
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
        Value::List(_) => Err(EvalError::TypeError(
            "floor: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "floor: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "floor: cannot convert lambda to number".to_string(),
        )),
    }
}

pub fn sqrt(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("sqrt: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n < 0.0 {
        return Err(EvalError::TypeError(
            "sqrt: cannot compute square root of negative number".to_string(),
        ));
    }
    Ok(Value::Float(n.sqrt()))
}

pub fn pow(args: &[Value]) -> Result<Value, EvalError> {
    let base = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("pow: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert lambda to number".to_string(),
            ))
        }
    };
    let exp = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("pow: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(base.powf(exp)))
}

// Trigonometric functions
pub fn sin(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("sin: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.sin()))
}

pub fn cos(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("cos: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.cos()))
}

pub fn tan(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("tan: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.tan()))
}

// Logarithmic and exponential functions
pub fn ln(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("ln: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n <= 0.0 {
        return Err(EvalError::TypeError(
            "ln: cannot compute logarithm of non-positive number".to_string(),
        ));
    }
    Ok(Value::Float(n.ln()))
}

pub fn log10(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("log10: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n <= 0.0 {
        return Err(EvalError::TypeError(
            "log10: cannot compute logarithm of non-positive number".to_string(),
        ));
    }
    Ok(Value::Float(n.log10()))
}

pub fn exp(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("exp: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::HashMapExt;
    use test_case::test_case;

    #[test_case(Value::Int(-5), Ok(Value::Int(5)) ; "abs_negative_int")]
    #[test_case(Value::Int(5), Ok(Value::Int(5)) ; "abs_positive_int")]
    #[test_case(Value::Int(0), Ok(Value::Int(0)) ; "abs_zero_int")]
    #[test_case(Value::Float(-3.5), Ok(Value::Float(3.5)) ; "abs_negative_float")]
    #[test_case(Value::Float(3.14), Ok(Value::Float(3.14)) ; "abs_positive_float")]
    #[test_case(Value::Float(0.0), Ok(Value::Float(0.0)) ; "abs_zero_float")]
    #[test_case(Value::Null, Ok(Value::Null) ; "abs_null")]
    #[test_case(Value::Bool(true), Ok(Value::Int(1)) ; "abs_true")]
    #[test_case(Value::Bool(false), Ok(Value::Int(0)) ; "abs_false")]
    fn test_abs_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(abs(&[input]).unwrap(), v),
            Err(_) => assert!(abs(&[input]).is_err()),
        }
    }

    #[test_case(Value::Float(3.7), Ok(Value::Int(4)) ; "round_up")]
    #[test_case(Value::Float(3.2), Ok(Value::Int(3)) ; "round_down")]
    #[test_case(Value::Float(5.0), Ok(Value::Int(5)) ; "round_exact")]
    #[test_case(Value::Float(2.5), Ok(Value::Int(3)) ; "round_half_up")]
    #[test_case(Value::Float(3.5), Ok(Value::Int(4)) ; "round_half_up_2")]
    #[test_case(Value::Float(-2.3), Ok(Value::Int(-2)) ; "round_negative_up")]
    #[test_case(Value::Float(-2.7), Ok(Value::Int(-3)) ; "round_negative_down")]
    #[test_case(Value::Int(5), Ok(Value::Int(5)) ; "round_int")]
    #[test_case(Value::Null, Ok(Value::Null) ; "round_null")]
    #[test_case(Value::Bool(true), Ok(Value::Int(1)) ; "round_true")]
    #[test_case(Value::Bool(false), Ok(Value::Int(0)) ; "round_false")]
    fn test_round_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(round(&[input]).unwrap(), v),
            Err(_) => assert!(round(&[input]).is_err()),
        }
    }

    #[test_case(vec![Value::Int(3), Value::Int(1), Value::Int(2)], Ok(Value::Int(1)) ; "min_int")]
    #[test_case(vec![Value::Float(3.5), Value::Float(1.2), Value::Float(2.8)], Ok(Value::Float(1.2)) ; "min_float")]
    #[test_case(vec![Value::Int(42)], Ok(Value::Int(42)) ; "min_single")]
    #[test_case(vec![Value::Null], Ok(Value::Null) ; "min_null")]
    #[test_case(vec![Value::Null, Value::Null], Ok(Value::Null) ; "min_all_null")]
    #[test_case(vec![Value::Bool(true), Value::Bool(false)], Ok(Value::Int(0)) ; "min_bool")]
    #[test_case(vec![Value::String("10".to_string()), Value::String("5".to_string())], Ok(Value::Int(5)) ; "min_string")]
    #[test_case(vec![Value::Int(3), Value::Float(1.5), Value::String("2".to_string())], Ok(Value::Float(1.5)) ; "min_mixed")]
    fn test_min_various(args: Vec<Value>, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(min(&args).unwrap(), v),
            Err(_) => assert!(min(&args).is_err()),
        }
    }

    #[test_case(vec![Value::Int(3), Value::Int(5), Value::Int(2)], Ok(Value::Int(5)) ; "max_int")]
    #[test_case(vec![Value::Float(3.5), Value::Float(5.2), Value::Float(2.8)], Ok(Value::Float(5.2)) ; "max_float")]
    #[test_case(vec![Value::Int(42)], Ok(Value::Int(42)) ; "max_single")]
    #[test_case(vec![Value::Null], Ok(Value::Null) ; "max_null")]
    #[test_case(vec![Value::Null, Value::Null], Ok(Value::Null) ; "max_all_null")]
    #[test_case(vec![Value::Bool(true), Value::Bool(false)], Ok(Value::Int(1)) ; "max_bool")]
    #[test_case(vec![Value::String("10".to_string()), Value::String("5".to_string())], Ok(Value::Int(10)) ; "max_string")]
    fn test_max_various(args: Vec<Value>, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(max(&args).unwrap(), v),
            Err(_) => assert!(max(&args).is_err()),
        }
    }

    #[test_case(Value::Float(3.7), Ok(Value::Int(3)) ; "int_from_float")]
    #[test_case(Value::Float(-3.7), Ok(Value::Int(-3)) ; "int_from_negative_float")]
    #[test_case(Value::Int(42), Ok(Value::Int(42)) ; "int_from_int")]
    #[test_case(Value::Null, Ok(Value::Null) ; "int_from_null")]
    #[test_case(Value::Bool(true), Ok(Value::Int(1)) ; "int_from_true")]
    #[test_case(Value::Bool(false), Ok(Value::Int(0)) ; "int_from_false")]
    fn test_int_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(int(&[input]).unwrap(), v),
            Err(_) => assert!(int(&[input]).is_err()),
        }
    }

    #[test_case(Value::Int(42), Ok(Value::Float(42.0)) ; "float_from_int")]
    #[test_case(Value::Int(-42), Ok(Value::Float(-42.0)) ; "float_from_negative_int")]
    #[test_case(Value::Null, Ok(Value::Null) ; "float_from_null")]
    #[test_case(Value::Bool(true), Ok(Value::Float(1.0)) ; "float_from_true")]
    #[test_case(Value::Bool(false), Ok(Value::Float(0.0)) ; "float_from_false")]
    fn test_float_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(float(&[input]).unwrap(), v),
            Err(_) => assert!(float(&[input]).is_err()),
        }
    }

    #[test_case(Value::Float(3.2), Ok(Value::Int(4)) ; "ceil_positive")]
    #[test_case(Value::Float(-3.7), Ok(Value::Int(-3)) ; "ceil_negative")]
    #[test_case(Value::Float(-3.2), Ok(Value::Int(-3)) ; "ceil_negative_2")]
    #[test_case(Value::Float(-3.8), Ok(Value::Int(-3)) ; "ceil_negative_3")]
    #[test_case(Value::Int(5), Ok(Value::Int(5)) ; "ceil_int")]
    #[test_case(Value::Null, Ok(Value::Null) ; "ceil_null")]
    #[test_case(Value::Bool(true), Ok(Value::Int(1)) ; "ceil_true")]
    #[test_case(Value::Bool(false), Ok(Value::Int(0)) ; "ceil_false")]
    fn test_ceil_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(ceil(&[input]).unwrap(), v),
            Err(_) => assert!(ceil(&[input]).is_err()),
        }
    }

    #[test_case(Value::Float(3.7), Ok(Value::Int(3)) ; "floor_positive")]
    #[test_case(Value::Float(-3.2), Ok(Value::Int(-4)) ; "floor_negative")]
    #[test_case(Value::Int(5), Ok(Value::Int(5)) ; "floor_int")]
    #[test_case(Value::Null, Ok(Value::Null) ; "floor_null")]
    #[test_case(Value::Bool(true), Ok(Value::Int(1)) ; "floor_true")]
    #[test_case(Value::Bool(false), Ok(Value::Int(0)) ; "floor_false")]
    fn test_floor_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(floor(&[input]).unwrap(), v),
            Err(_) => assert!(floor(&[input]).is_err()),
        }
    }

    #[test_case(Value::Float(16.0), Ok(Value::Float(4.0)) ; "sqrt_float")]
    #[test_case(Value::Int(9), Ok(Value::Float(3.0)) ; "sqrt_int")]
    #[test_case(Value::Int(16), Ok(Value::Float(4.0)) ; "sqrt_int_16")]
    #[test_case(Value::Float(0.0), Ok(Value::Float(0.0)) ; "sqrt_zero")]
    #[test_case(Value::Float(1.0), Ok(Value::Float(1.0)) ; "sqrt_one")]
    #[test_case(Value::Bool(true), Ok(Value::Float(1.0)) ; "sqrt_true")]
    #[test_case(Value::Bool(false), Ok(Value::Float(0.0)) ; "sqrt_false")]
    #[test_case(Value::Null, Ok(Value::Null) ; "sqrt_null")]
    #[test_case(Value::Float(-4.0), Err(()) ; "sqrt_negative")]
    fn test_sqrt_basic(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(sqrt(&[input]).unwrap(), v),
            Err(_) => assert!(sqrt(&[input]).is_err()),
        }
    }

    #[test_case(Value::Float(2.0), Value::Float(3.0), Ok(Value::Float(8.0)) ; "pow_float")]
    #[test_case(Value::Int(2), Value::Int(10), Ok(Value::Float(1024.0)) ; "pow_int")]
    #[test_case(Value::Int(5), Value::Int(0), Ok(Value::Float(1.0)) ; "pow_zero_exp")]
    #[test_case(Value::Int(0), Value::Int(5), Ok(Value::Float(0.0)) ; "pow_zero_base")]
    #[test_case(Value::Int(2), Value::Int(-1), Ok(Value::Float(0.5)) ; "pow_neg_exp")]
    #[test_case(Value::Null, Value::Int(2), Ok(Value::Null) ; "pow_null_base")]
    #[test_case(Value::Int(2), Value::Null, Ok(Value::Null) ; "pow_null_exp")]
    fn test_pow_basic(base: Value, exp: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(pow(&[base, exp]).unwrap(), v),
            Err(_) => assert!(pow(&[base, exp]).is_err()),
        }
    }

    #[test_case(Value::Float(0.0), 0.0 ; "sin_zero")]
    #[test_case(Value::Float(std::f64::consts::PI / 2.0), 1.0 ; "sin_pi_2")]
    #[test_case(Value::Null, f64::NAN ; "sin_null")]
    fn test_sin_basic(input: Value, expected: f64) {
        let result = sin(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case(Value::Float(0.0), 1.0 ; "cos_zero")]
    #[test_case(Value::Float(std::f64::consts::PI), -1.0 ; "cos_pi")]
    #[test_case(Value::Null, f64::NAN ; "cos_null")]
    fn test_cos_basic(input: Value, expected: f64) {
        let result = cos(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case(Value::Float(0.0), 0.0 ; "tan_zero")]
    #[test_case(Value::Float(std::f64::consts::PI / 4.0), 1.0 ; "tan_pi_4")]
    #[test_case(Value::Null, f64::NAN ; "tan_null")]
    fn test_tan_basic(input: Value, expected: f64) {
        let result = tan(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case(Value::Float(1.0), 0.0 ; "ln_one")]
    #[test_case(Value::Float(std::f64::consts::E), 1.0 ; "ln_e")]
    #[test_case(Value::Null, f64::NAN ; "ln_null")]
    #[test_case(Value::Bool(true), 0.0 ; "ln_true")]
    fn test_ln_basic(input: Value, expected: f64) {
        let result = ln(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case(Value::Float(1.0), 0.0 ; "log10_one")]
    #[test_case(Value::Float(10.0), 1.0 ; "log10_10")]
    #[test_case(Value::Float(100.0), 2.0 ; "log10_100")]
    #[test_case(Value::Null, f64::NAN ; "log10_null")]
    #[test_case(Value::Bool(true), 0.0 ; "log10_true")]
    fn test_log10_basic(input: Value, expected: f64) {
        let result = log10(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case(Value::Float(0.0), 1.0 ; "exp_zero")]
    #[test_case(Value::Float(1.0), std::f64::consts::E ; "exp_one")]
    #[test_case(Value::Null, f64::NAN ; "exp_null")]
    #[test_case(Value::Bool(true), std::f64::consts::E ; "exp_true")]
    fn test_exp_basic(input: Value, expected: f64) {
        let result = exp(&[input]).unwrap();
        if expected.is_nan() {
            assert_eq!(result, Value::Null);
        } else {
            assert!((result.as_float().unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test_case("-42", Ok(Value::Int(42)) ; "abs_int_string")]
    #[test_case("-3.14", Ok(Value::Float(3.14)) ; "abs_float_string")]
    #[test_case("0", Ok(Value::Int(0)) ; "abs_zero_string")]
    #[test_case("abc", Err(()) ; "abs_invalid_string")]
    fn test_abs_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(abs(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(abs(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test]
    fn test_min_no_args() {
        assert!(min(&[]).is_err());
    }

    #[test]
    fn test_min_with_null_and_list() {
        // Null should be skipped, List should be skipped
        assert_eq!(
            min(&[
                Value::Null,
                Value::Int(5),
                Value::List(vec![]),
                Value::Int(3)
            ])
            .unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_max_no_args() {
        assert!(max(&[]).is_err());
    }

    #[test_case("42", Ok(Value::Int(42)) ; "int_string_int")]
    #[test_case("3.14", Ok(Value::Int(3)) ; "int_string_float")]
    #[test_case("-42", Ok(Value::Int(-42)) ; "int_string_negative")]
    #[test_case("abc", Err(()) ; "int_string_invalid")]
    fn test_int_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(int(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(int(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test_case("3.14", Ok(Value::Float(3.14)) ; "float_string")]
    #[test_case("-3.14", Ok(Value::Float(-3.14)) ; "float_string_negative")]
    #[test_case("abc", Err(()) ; "float_string_invalid")]
    fn test_float_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(float(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(float(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test_case("3.2", Ok(Value::Int(4)) ; "ceil_string")]
    #[test_case("abc", Err(()) ; "ceil_string_invalid")]
    fn test_ceil_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(ceil(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(ceil(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test_case("3.7", Ok(Value::Int(3)) ; "floor_string")]
    #[test_case("abc", Err(()) ; "floor_string_invalid")]
    fn test_floor_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(floor(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(floor(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test_case("16", Ok(Value::Float(4.0)) ; "sqrt_string")]
    #[test_case("abc", Err(()) ; "sqrt_string_invalid")]
    fn test_sqrt_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(sqrt(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(sqrt(&[Value::String(input.to_string())]).is_err()),
        }
    }

    #[test]
    fn test_pow_string() {
        assert_eq!(
            pow(&[
                Value::String("2".to_string()),
                Value::String("3".to_string())
            ])
            .unwrap(),
            Value::Float(8.0)
        );
    }

    #[test]
    fn test_trigonometric_edge_cases() {
        // sin(π) ≈ 0
        assert!(
            (sin(&[Value::Float(std::f64::consts::PI)])
                .unwrap()
                .as_float()
                .unwrap()
                - 0.0)
                .abs()
                < 1e-10
        );
        // cos(π/2) ≈ 0
        assert!(
            (cos(&[Value::Float(std::f64::consts::PI / 2.0)])
                .unwrap()
                .as_float()
                .unwrap()
                - 0.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_ln_zero() {
        // ln(0) should error
        assert!(ln(&[Value::Float(0.0)]).is_err());
    }

    #[test]
    fn test_log10_negative() {
        assert!(log10(&[Value::Float(-1.0)]).is_err());
    }

    #[test]
    fn test_exp_zero() {
        assert_eq!(exp(&[Value::Float(0.0)]).unwrap(), Value::Float(1.0));
    }

    #[test]
    fn test_type_errors() {
        // Test that List, DateTime, Lambda produce type errors
        assert!(abs(&[Value::List(vec![])]).is_err());
        assert!(round(&[Value::List(vec![])]).is_err());
        assert!(int(&[Value::List(vec![])]).is_err());
        assert!(float(&[Value::List(vec![])]).is_err());
        assert!(ceil(&[Value::List(vec![])]).is_err());
        assert!(floor(&[Value::List(vec![])]).is_err());
        assert!(sqrt(&[Value::List(vec![])]).is_err());
    }

    // Additional tests for abs

    // Additional tests for round
    #[test_case("3.7", Ok(Value::Int(4)) ; "round_string_up")]
    #[test_case("3.2", Ok(Value::Int(3)) ; "round_string_down")]
    #[test_case("abc", Err(()) ; "round_string_invalid")]
    fn test_round_string(input: &str, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(round(&[Value::String(input.to_string())]).unwrap(), v),
            Err(_) => assert!(round(&[Value::String(input.to_string())]).is_err()),
        }
    }

    // Additional tests for min/max
    #[test]
    fn test_max_string_values() {
        assert_eq!(
            max(&[
                Value::String("10".to_string()),
                Value::String("5".to_string())
            ])
            .unwrap(),
            Value::Int(10)
        );
    }

    // Additional tests for int
    #[test]
    fn test_int_large_number() {
        assert_eq!(
            int(&[Value::Int(9007199254740992i64)]).unwrap(),
            Value::Int(9007199254740992i64)
        );
    }

    // Additional tests for float
    #[test]
    fn test_float_negative_string() {
        assert_eq!(
            float(&[Value::String("-3.14".to_string())]).unwrap(),
            Value::Float(-3.14)
        );
    }

    #[test]
    fn test_float_scientific_notation() {
        assert_eq!(
            float(&[Value::String("1e10".to_string())]).unwrap(),
            Value::Float(1e10)
        );
    }

    // Additional tests for ceil and floor

    #[test]
    fn test_pow_fractional() {
        assert!(
            (pow(&[Value::Float(4.0), Value::Float(0.5)])
                .unwrap()
                .as_float()
                .unwrap()
                - 2.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_pow_bool() {
        assert_eq!(
            pow(&[Value::Bool(true), Value::Bool(true)]).unwrap(),
            Value::Float(1.0)
        );
    }

    // Additional tests for trigonometric functions

    // Additional tests for logarithmic functions

    #[test]
    fn test_exp_negative() {
        assert!(
            (exp(&[Value::Float(-1.0)]).unwrap().as_float().unwrap()
                - 1.0 / std::f64::consts::E)
                .abs()
                < 1e-10
        );
    }

    // Test type errors for DateTime and Lambda
    #[test]
    fn test_datetime_type_errors() {
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        assert!(abs(&[dt.clone()]).is_err());
        assert!(round(&[dt.clone()]).is_err());
        assert!(int(&[dt.clone()]).is_err());
        assert!(float(&[dt.clone()]).is_err());
        assert!(ceil(&[dt.clone()]).is_err());
        assert!(floor(&[dt.clone()]).is_err());
        assert!(sqrt(&[dt.clone()]).is_err());
        assert!(pow(&[dt.clone(), Value::Int(2)]).is_err());
        assert!(sin(&[dt.clone()]).is_err());
        assert!(cos(&[dt.clone()]).is_err());
        assert!(tan(&[dt.clone()]).is_err());
        assert!(ln(&[dt.clone()]).is_err());
        assert!(log10(&[dt.clone()]).is_err());
        assert!(exp(&[dt.clone()]).is_err());
    }

    #[test]
    fn test_lambda_type_errors() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        assert!(abs(&[lambda.clone()]).is_err());
        assert!(round(&[lambda.clone()]).is_err());
        assert!(int(&[lambda.clone()]).is_err());
        assert!(float(&[lambda.clone()]).is_err());
        assert!(ceil(&[lambda.clone()]).is_err());
        assert!(floor(&[lambda.clone()]).is_err());
        assert!(sqrt(&[lambda.clone()]).is_err());
        assert!(pow(&[lambda.clone(), Value::Int(2)]).is_err());
        assert!(sin(&[lambda.clone()]).is_err());
        assert!(cos(&[lambda.clone()]).is_err());
        assert!(tan(&[lambda.clone()]).is_err());
        assert!(ln(&[lambda.clone()]).is_err());
        assert!(log10(&[lambda.clone()]).is_err());
        assert!(exp(&[lambda.clone()]).is_err());
    }

    // Test edge cases with special float values
    #[test]
    fn test_special_float_values() {
        // sqrt of infinity
        assert!(sqrt(&[Value::Float(f64::INFINITY)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_infinite());
        // sqrt of NaN
        assert!(sqrt(&[Value::Float(f64::NAN)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_nan());
        // exp of infinity
        assert!(exp(&[Value::Float(f64::INFINITY)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_infinite());
        // exp of large negative
        assert_eq!(
            exp(&[Value::Float(-1000.0)]).unwrap().as_float().unwrap(),
            0.0
        );
    }

    // Test max/min with DateTime and Lambda (should be skipped)
    #[test]
    fn test_min_max_skips_datetime_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });

        // DateTime and Lambda should be skipped
        assert_eq!(
            min(&[dt.clone(), Value::Int(5), lambda.clone()]).unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            max(&[dt.clone(), Value::Int(5), lambda.clone()]).unwrap(),
            Value::Int(5)
        );
    }

    // Additional tests to cover error handling branches

    #[test]
    fn test_min_with_list() {
        // List should be skipped in min
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(min(&[list, Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_max_with_list() {
        // List should be skipped in max
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(max(&[list, Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_int_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = int(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_float_invalid_string_error() {
        let result = float(&[Value::String("not-a-number".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot convert"));
    }

    #[test]
    fn test_pow_with_list_base_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = pow(&[list, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_pow_with_datetime_base_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = pow(&[dt, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_pow_with_lambda_base_error() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = pow(&[lambda, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert lambda"));
    }

    #[test]
    fn test_pow_with_list_exp_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = pow(&[Value::Int(2), list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_pow_with_datetime_exp_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = pow(&[Value::Int(2), dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sin_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = sin(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_sin_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = sin(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sin_with_lambda_error() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = sin(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert lambda"));
    }

    #[test]
    fn test_cos_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = cos(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_cos_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = cos(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_tan_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = tan(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_tan_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = tan(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_ln_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = ln(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_ln_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = ln(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_log10_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = log10(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_log10_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = log10(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_exp_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = exp(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_exp_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = exp(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sqrt_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = sqrt(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_sqrt_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = sqrt(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_int_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = int(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_float_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = float(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_float_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = float(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_ceil_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = ceil(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_ceil_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = ceil(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_floor_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = floor(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_floor_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = floor(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_abs_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = abs(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_abs_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = abs(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_round_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = round(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_round_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = round(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    // Tests moved from src/libs/expr/tests/functions.rs
    #[test]
    fn test_abs_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["-5".to_string()];
        assert_eq!(eval_expr("abs(@1)", &row, None).unwrap().to_string(), "5");

        let row: Vec<String> = vec!["-3.14".to_string()];
        assert_eq!(
            eval_expr("abs(@1)", &row, None).unwrap().to_string(),
            "3.14"
        );
    }

    #[test]
    fn test_round_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["3.7".to_string()];
        assert_eq!(eval_expr("round(@1)", &row, None).unwrap().to_string(), "4");

        let row: Vec<String> = vec!["3.2".to_string()];
        assert_eq!(eval_expr("round(@1)", &row, None).unwrap().to_string(), "3");
    }

    #[test]
    fn test_min_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec![];
        assert_eq!(
            eval_expr("min(3, 1, 2)", &row, None).unwrap().to_string(),
            "1"
        );
        assert_eq!(
            eval_expr("min(10, 5)", &row, None).unwrap().to_string(),
            "5"
        );
    }

    #[test]
    fn test_max_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec![];
        assert_eq!(
            eval_expr("max(3, 1, 2)", &row, None).unwrap().to_string(),
            "3"
        );
        assert_eq!(
            eval_expr("max(10, 5)", &row, None).unwrap().to_string(),
            "10"
        );
    }

    // Additional edge case tests for abs
    #[test]
    fn test_abs_float_string_integer() {
        // Float string that parses as integer (no decimal)
        assert_eq!(
            abs(&[Value::String("42.0".to_string())]).unwrap(),
            Value::Float(42.0)
        );
    }

    #[test]
    fn test_abs_zero_string() {
        assert_eq!(
            abs(&[Value::String("0".to_string())]).unwrap(),
            Value::Int(0)
        );
        assert_eq!(
            abs(&[Value::String("-0".to_string())]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_abs_large_float_string() {
        assert_eq!(
            abs(&[Value::String("-1.5e10".to_string())]).unwrap(),
            Value::Float(1.5e10)
        );
    }

    // Additional edge case tests for round
    #[test]
    fn test_round_exact_half() {
        assert_eq!(round(&[Value::Float(2.5)]).unwrap(), Value::Int(3));
        assert_eq!(round(&[Value::Float(3.5)]).unwrap(), Value::Int(4));
        // Rust's round() rounds half away from zero
        assert_eq!(round(&[Value::Float(-2.5)]).unwrap(), Value::Int(-3));
    }

    #[test]
    fn test_round_negative_edge_cases() {
        assert_eq!(round(&[Value::Float(-2.7)]).unwrap(), Value::Int(-3));
        assert_eq!(round(&[Value::Float(-2.3)]).unwrap(), Value::Int(-2));
    }

    #[test]
    fn test_round_zero() {
        assert_eq!(round(&[Value::Float(0.0)]).unwrap(), Value::Int(0));
        assert_eq!(round(&[Value::Float(-0.0)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_round_string_scientific() {
        assert_eq!(
            round(&[Value::String("1.5e2".to_string())]).unwrap(),
            Value::Int(150)
        );
    }

    // Additional edge case tests for min/max
    #[test]
    fn test_min_max_with_only_null() {
        assert_eq!(min(&[Value::Null, Value::Null]).unwrap(), Value::Null);
        assert_eq!(max(&[Value::Null, Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_min_max_with_only_lists() {
        let list = Value::List(vec![Value::Int(1)]);
        assert_eq!(
            min(&[list.clone(), Value::List(vec![])]).unwrap(),
            Value::Null
        );
        assert_eq!(
            max(&[list.clone(), Value::List(vec![])]).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn test_min_max_with_only_datetimes() {
        use chrono::Utc;
        let dt1 = Value::DateTime(Utc::now());
        let dt2 = Value::DateTime(Utc::now());
        assert_eq!(min(&[dt1.clone(), dt2.clone()]).unwrap(), Value::Null);
        assert_eq!(max(&[dt1.clone(), dt2.clone()]).unwrap(), Value::Null);
    }

    #[test]
    fn test_min_max_with_only_lambdas() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        assert_eq!(min(&[lambda.clone(), lambda.clone()]).unwrap(), Value::Null);
        assert_eq!(max(&[lambda.clone(), lambda.clone()]).unwrap(), Value::Null);
    }

    #[test]
    fn test_min_max_mixed_with_invalid_strings() {
        assert!(min(&[Value::String("abc".to_string()), Value::Int(1)]).is_err());
        assert!(max(&[Value::String("abc".to_string()), Value::Int(1)]).is_err());
    }

    #[test]
    fn test_min_max_empty_string() {
        assert!(min(&[Value::String("".to_string()), Value::Int(1)]).is_err());
        assert!(max(&[Value::String("".to_string()), Value::Int(1)]).is_err());
    }

    // Additional edge case tests for int
    #[test]
    fn test_int_scientific_notation() {
        assert_eq!(
            int(&[Value::String("1e5".to_string())]).unwrap(),
            Value::Int(100000)
        );
        assert_eq!(
            int(&[Value::String("1.5e2".to_string())]).unwrap(),
            Value::Int(150)
        );
    }

    #[test]
    fn test_int_negative_string_edge_case() {
        assert_eq!(
            int(&[Value::String("-42".to_string())]).unwrap(),
            Value::Int(-42)
        );
    }

    #[test]
    fn test_int_zero_string() {
        assert_eq!(
            int(&[Value::String("0".to_string())]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_int_empty_string() {
        assert!(int(&[Value::String("".to_string())]).is_err());
    }

    #[test]
    fn test_int_whitespace_string() {
        assert!(int(&[Value::String("  ".to_string())]).is_err());
        assert!(int(&[Value::String(" 42 ".to_string())]).is_err());
    }

    // Additional edge case tests for float
    #[test]
    fn test_float_scientific_notation_edge_cases() {
        assert_eq!(
            float(&[Value::String("1e-5".to_string())]).unwrap(),
            Value::Float(1e-5)
        );
        assert_eq!(
            float(&[Value::String("-1.5e10".to_string())]).unwrap(),
            Value::Float(-1.5e10)
        );
    }

    #[test]
    fn test_float_zero() {
        assert_eq!(
            float(&[Value::String("0.0".to_string())]).unwrap(),
            Value::Float(0.0)
        );
        assert_eq!(
            float(&[Value::String("-0.0".to_string())]).unwrap(),
            Value::Float(-0.0)
        );
    }

    #[test]
    fn test_float_empty_string() {
        assert!(float(&[Value::String("".to_string())]).is_err());
    }

    #[test]
    fn test_float_whitespace_string() {
        assert!(float(&[Value::String("  ".to_string())]).is_err());
    }

    #[test]
    fn test_float_infinity() {
        assert_eq!(
            float(&[Value::String("inf".to_string())]).unwrap(),
            Value::Float(f64::INFINITY)
        );
        assert_eq!(
            float(&[Value::String("-inf".to_string())]).unwrap(),
            Value::Float(f64::NEG_INFINITY)
        );
    }

    // Additional edge case tests for ceil
    #[test]
    fn test_ceil_scientific_notation() {
        assert_eq!(
            ceil(&[Value::String("1.1e2".to_string())]).unwrap(),
            Value::Int(110)
        );
    }

    #[test]
    fn test_ceil_zero() {
        assert_eq!(ceil(&[Value::Float(0.0)]).unwrap(), Value::Int(0));
        assert_eq!(ceil(&[Value::Float(-0.0)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_ceil_empty_string() {
        assert!(ceil(&[Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for floor
    #[test]
    fn test_floor_scientific_notation() {
        assert_eq!(
            floor(&[Value::String("1.9e2".to_string())]).unwrap(),
            Value::Int(190)
        );
    }

    #[test]
    fn test_floor_zero() {
        assert_eq!(floor(&[Value::Float(0.0)]).unwrap(), Value::Int(0));
        assert_eq!(floor(&[Value::Float(-0.0)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_floor_empty_string() {
        assert!(floor(&[Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for sqrt
    #[test]
    fn test_sqrt_scientific_notation() {
        assert!(
            (sqrt(&[Value::String("1e4".to_string())])
                .unwrap()
                .as_float()
                .unwrap()
                - 100.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_sqrt_very_small() {
        assert!(
            (sqrt(&[Value::Float(1e-10)]).unwrap().as_float().unwrap() - 1e-5).abs()
                < 1e-10
        );
    }

    #[test]
    fn test_sqrt_empty_string() {
        assert!(sqrt(&[Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for pow
    #[test]
    fn test_pow_one() {
        assert_eq!(
            pow(&[Value::Int(5), Value::Int(1)]).unwrap(),
            Value::Float(5.0)
        );
    }

    #[test]
    fn test_pow_negative_base_even_exponent() {
        assert_eq!(
            pow(&[Value::Int(-2), Value::Int(4)]).unwrap(),
            Value::Float(16.0)
        );
    }

    #[test]
    fn test_pow_negative_base_odd_exponent() {
        assert_eq!(
            pow(&[Value::Int(-2), Value::Int(3)]).unwrap(),
            Value::Float(-8.0)
        );
    }

    #[test]
    fn test_pow_scientific_notation() {
        assert!(
            (pow(&[Value::String("1e2".to_string()), Value::Int(2)])
                .unwrap()
                .as_float()
                .unwrap()
                - 10000.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_pow_empty_string() {
        assert!(pow(&[Value::String("".to_string()), Value::Int(2)]).is_err());
        assert!(pow(&[Value::Int(2), Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for trigonometric functions
    #[test]
    fn test_sin_negative() {
        assert!(
            (sin(&[Value::Float(-std::f64::consts::PI / 2.0)])
                .unwrap()
                .as_float()
                .unwrap()
                + 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_sin_empty_string() {
        assert!(sin(&[Value::String("".to_string())]).is_err());
    }

    #[test]
    fn test_cos_negative() {
        assert!(
            (cos(&[Value::Float(-std::f64::consts::PI)])
                .unwrap()
                .as_float()
                .unwrap()
                + 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_cos_empty_string() {
        assert!(cos(&[Value::String("".to_string())]).is_err());
    }

    #[test]
    fn test_tan_empty_string() {
        assert!(tan(&[Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for logarithmic functions
    #[test]
    fn test_ln_fraction() {
        assert!(
            (ln(&[Value::Float(0.5)]).unwrap().as_float().unwrap()
                - (-0.6931471805599453))
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_ln_empty_string() {
        assert!(ln(&[Value::String("".to_string())]).is_err());
    }

    #[test]
    fn test_log10_fraction() {
        assert!(
            (log10(&[Value::Float(0.1)]).unwrap().as_float().unwrap() - (-1.0)).abs()
                < 1e-10
        );
    }

    #[test]
    fn test_log10_empty_string() {
        assert!(log10(&[Value::String("".to_string())]).is_err());
    }

    // Additional edge case tests for exp
    #[test]
    fn test_exp_negative_edge_case() {
        assert!(
            (exp(&[Value::Float(-1.0)]).unwrap().as_float().unwrap()
                - 0.36787944117144233)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_exp_large() {
        assert!(exp(&[Value::Float(1000.0)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_infinite());
    }

    #[test]
    fn test_exp_empty_string() {
        assert!(exp(&[Value::String("".to_string())]).is_err());
    }

    // Additional integration tests
    #[test]
    fn test_int_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["3.14".to_string()];
        assert_eq!(eval_expr("int(@1)", &row, None).unwrap().to_string(), "3");

        let row: Vec<String> = vec!["42".to_string()];
        assert_eq!(eval_expr("int(@1)", &row, None).unwrap().to_string(), "42");
    }

    #[test]
    fn test_float_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["42".to_string()];
        assert_eq!(
            eval_expr("float(@1)", &row, None).unwrap().to_string(),
            "42"
        );
    }

    #[test]
    fn test_ceil_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["3.2".to_string()];
        assert_eq!(eval_expr("ceil(@1)", &row, None).unwrap().to_string(), "4");
    }

    #[test]
    fn test_floor_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["3.9".to_string()];
        assert_eq!(eval_expr("floor(@1)", &row, None).unwrap().to_string(), "3");
    }

    #[test]
    fn test_sqrt_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["16".to_string()];
        assert_eq!(eval_expr("sqrt(@1)", &row, None).unwrap().to_string(), "4");
    }

    #[test]
    fn test_pow_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec![];
        assert_eq!(eval_expr("pow(2, 3)", &row, None).unwrap().to_string(), "8");
    }
}
