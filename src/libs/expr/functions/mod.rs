use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use std::collections::HashMap;

/// Function signature
pub type Function = fn(&[Value]) -> Result<Value, EvalError>;

/// Function metadata
#[derive(Clone)]
pub struct FunctionInfo {
    pub func: Function,
    pub min_arity: usize,
    pub max_arity: usize, // usize::MAX for variadic
}

impl FunctionInfo {
    pub fn new(func: Function, min_arity: usize, max_arity: usize) -> Self {
        Self {
            func,
            min_arity,
            max_arity,
        }
    }

    pub fn fixed(func: Function, arity: usize) -> Self {
        Self::new(func, arity, arity)
    }

    pub fn variadic(func: Function, min_arity: usize) -> Self {
        Self::new(func, min_arity, usize::MAX)
    }
}

/// Function registry
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, FunctionInfo>,
}

impl FunctionRegistry {
    /// Create a new registry with built-in functions
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtins();
        registry
    }

    /// Register a function
    pub fn register(&mut self, name: impl Into<String>, info: FunctionInfo) {
        self.functions.insert(name.into(), info);
    }

    /// Look up a function by name
    pub fn get(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    /// Check if a function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Call a function by name with arguments
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, EvalError> {
        match self.get(name) {
            Some(info) => {
                if args.len() < info.min_arity {
                    return Err(EvalError::WrongArity {
                        name: name.to_string(),
                        expected: info.min_arity,
                        got: args.len(),
                    });
                }
                if args.len() > info.max_arity && info.max_arity != usize::MAX {
                    return Err(EvalError::WrongArity {
                        name: name.to_string(),
                        expected: info.max_arity,
                        got: args.len(),
                    });
                }
                (info.func)(args)
            }
            None => Err(EvalError::UnknownFunction(name.to_string())),
        }
    }

    /// Register all built-in functions
    fn register_builtins(&mut self) {
        // String functions
        self.register("trim", FunctionInfo::fixed(string::trim, 1));
        self.register("upper", FunctionInfo::fixed(string::upper, 1));
        self.register("lower", FunctionInfo::fixed(string::lower, 1));
        self.register("len", FunctionInfo::fixed(string::len, 1));
        self.register("substr", FunctionInfo::fixed(string::substr, 3));
        self.register("split", FunctionInfo::fixed(string::split, 2));
        self.register("contains", FunctionInfo::fixed(string::contains, 2));
        self.register("starts_with", FunctionInfo::fixed(string::starts_with, 2));
        self.register("ends_with", FunctionInfo::fixed(string::ends_with, 2));

        // Numeric functions
        self.register("abs", FunctionInfo::fixed(numeric::abs, 1));
        self.register("round", FunctionInfo::fixed(numeric::round, 1));
        self.register("min", FunctionInfo::variadic(numeric::min, 1));
        self.register("max", FunctionInfo::variadic(numeric::max, 1));
        self.register("int", FunctionInfo::fixed(numeric::int, 1));
        self.register("float", FunctionInfo::fixed(numeric::float, 1));

        // Logical functions
        self.register("if", FunctionInfo::fixed(logical::if_fn, 3));
        self.register("default", FunctionInfo::fixed(logical::default_fn, 2));
    }
}

/// String functions
pub mod string {
    use super::*;

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
}

/// Numeric functions
pub mod numeric {
    use super::*;

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
}

/// Logical functions
pub mod logical {
    use super::*;

    pub fn if_fn(args: &[Value]) -> Result<Value, EvalError> {
        let condition = args[0].as_bool();
        if condition {
            Ok(args[1].clone())
        } else {
            Ok(args[2].clone())
        }
    }

    pub fn default_fn(args: &[Value]) -> Result<Value, EvalError> {
        if args[0].is_null() || args[0].as_bool() == false {
            Ok(args[1].clone())
        } else {
            Ok(args[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        let registry = FunctionRegistry::new();
        let result = registry.call("trim", &[Value::String("  hello  ".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_upper() {
        let registry = FunctionRegistry::new();
        let result = registry.call("upper", &[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_abs() {
        let registry = FunctionRegistry::new();
        let result = registry.call("abs", &[Value::Int(-5)]);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_min() {
        let registry = FunctionRegistry::new();
        let result =
            registry.call("min", &[Value::Int(3), Value::Int(1), Value::Int(2)]);
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_if() {
        let registry = FunctionRegistry::new();
        let result =
            registry.call("if", &[Value::Bool(true), Value::Int(1), Value::Int(0)]);
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_unknown_function() {
        let registry = FunctionRegistry::new();
        let result = registry.call("unknown", &[]);
        assert!(matches!(result, Err(EvalError::UnknownFunction(_))));
    }

    #[test]
    fn test_wrong_arity() {
        let registry = FunctionRegistry::new();
        let result = registry.call("trim", &[]);
        assert!(matches!(result, Err(EvalError::WrongArity { .. })));
    }
}
