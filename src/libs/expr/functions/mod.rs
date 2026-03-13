use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use std::collections::HashMap;

mod io;
mod list;
mod logical;
mod numeric;
mod string;

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
        self.register("replace", FunctionInfo::fixed(string::replace, 3));

        // Numeric functions
        self.register("abs", FunctionInfo::fixed(numeric::abs, 1));
        self.register("round", FunctionInfo::fixed(numeric::round, 1));
        self.register("min", FunctionInfo::variadic(numeric::min, 1));
        self.register("max", FunctionInfo::variadic(numeric::max, 1));
        self.register("int", FunctionInfo::fixed(numeric::int, 1));
        self.register("float", FunctionInfo::fixed(numeric::float, 1));
        self.register("ceil", FunctionInfo::fixed(numeric::ceil, 1));
        self.register("floor", FunctionInfo::fixed(numeric::floor, 1));

        // Logical functions
        self.register("if", FunctionInfo::fixed(logical::if_fn, 3));
        self.register("default", FunctionInfo::fixed(logical::default_fn, 2));

        // IO functions
        self.register("print", FunctionInfo::variadic(io::print, 1));
        self.register("eprint", FunctionInfo::variadic(io::eprint, 1));

        // List functions
        self.register("join", FunctionInfo::fixed(list::join, 2));
        self.register("first", FunctionInfo::fixed(list::first, 1));
        self.register("last", FunctionInfo::fixed(list::last, 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // IO function tests
    #[test]
    fn test_print() {
        let registry = FunctionRegistry::new();
        let result = registry.call("print", &[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_print_multiple_args() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "print",
            &[Value::String("a".to_string()), Value::Int(1), Value::Bool(true)],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eprint() {
        let registry = FunctionRegistry::new();
        let result = registry.call("eprint", &[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    // String function tests
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
    fn test_lower() {
        let registry = FunctionRegistry::new();
        let result = registry.call("lower", &[Value::String("HELLO".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_len() {
        let registry = FunctionRegistry::new();
        let result = registry.call("len", &[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_substr() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "substr",
            &[Value::String("hello world".to_string()), Value::Int(0), Value::Int(5)],
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_split() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "split",
            &[Value::String("a,b,c".to_string()), Value::String(",".to_string())],
        );
        match result.unwrap() {
            Value::List(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::String("a".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_contains() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "contains",
            &[Value::String("hello world".to_string()), Value::String("world".to_string())],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_starts_with() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "starts_with",
            &[Value::String("hello".to_string()), Value::String("he".to_string())],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_ends_with() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "ends_with",
            &[Value::String("hello".to_string()), Value::String("lo".to_string())],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // Numeric function tests
    #[test]
    fn test_abs() {
        let registry = FunctionRegistry::new();
        let result = registry.call("abs", &[Value::Int(-5)]);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_round() {
        let registry = FunctionRegistry::new();
        let result = registry.call("round", &[Value::Float(3.7)]);
        assert_eq!(result.unwrap(), Value::Int(4));
    }

    #[test]
    fn test_min() {
        let registry = FunctionRegistry::new();
        let result =
            registry.call("min", &[Value::Int(3), Value::Int(1), Value::Int(2)]);
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_max() {
        let registry = FunctionRegistry::new();
        let result =
            registry.call("max", &[Value::Int(3), Value::Int(5), Value::Int(2)]);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_int() {
        let registry = FunctionRegistry::new();
        let result = registry.call("int", &[Value::Float(3.7)]);
        assert_eq!(result.unwrap(), Value::Int(3));

        let result = registry.call("int", &[Value::String("42".to_string())]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_float() {
        let registry = FunctionRegistry::new();
        let result = registry.call("float", &[Value::Int(42)]);
        assert_eq!(result.unwrap(), Value::Float(42.0));

        let result = registry.call("float", &[Value::String("3.14".to_string())]);
        assert_eq!(result.unwrap(), Value::Float(3.14));
    }

    // Logical function tests
    #[test]
    fn test_if() {
        let registry = FunctionRegistry::new();
        let result =
            registry.call("if", &[Value::Bool(true), Value::Int(1), Value::Int(0)]);
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_default() {
        let registry = FunctionRegistry::new();
        
        // null returns fallback
        let result = registry.call("default", &[Value::Null, Value::String("fallback".to_string())]);
        assert_eq!(result.unwrap(), Value::String("fallback".to_string()));
        
        // false returns fallback
        let result = registry.call("default", &[Value::Bool(false), Value::Int(0)]);
        assert_eq!(result.unwrap(), Value::Int(0));
        
        // non-null returns original
        let result = registry.call("default", &[Value::String("value".to_string()), Value::String("fallback".to_string())]);
        assert_eq!(result.unwrap(), Value::String("value".to_string()));
    }

    // Error handling tests
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

    // Edge case tests
    #[test]
    fn test_trim_null() {
        let registry = FunctionRegistry::new();
        let result = registry.call("trim", &[Value::Null]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_len_null() {
        let registry = FunctionRegistry::new();
        let result = registry.call("len", &[Value::Null]);
        assert_eq!(result.unwrap(), Value::Int(0));
    }

    #[test]
    fn test_abs_null() {
        let registry = FunctionRegistry::new();
        let result = registry.call("abs", &[Value::Null]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_min_empty() {
        let registry = FunctionRegistry::new();
        let result = registry.call("min", &[Value::Null]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_max_empty() {
        let registry = FunctionRegistry::new();
        let result = registry.call("max", &[Value::Null]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    // New function tests
    #[test]
    fn test_replace() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "replace",
            &[Value::String("hello world".to_string()), Value::String("world".to_string()), Value::String("rust".to_string())],
        );
        assert_eq!(result.unwrap(), Value::String("hello rust".to_string()));
    }

    #[test]
    fn test_ceil() {
        let registry = FunctionRegistry::new();
        let result = registry.call("ceil", &[Value::Float(3.2)]);
        assert_eq!(result.unwrap(), Value::Int(4));

        let result = registry.call("ceil", &[Value::Float(-3.7)]);
        assert_eq!(result.unwrap(), Value::Int(-3));
    }

    #[test]
    fn test_floor() {
        let registry = FunctionRegistry::new();
        let result = registry.call("floor", &[Value::Float(3.7)]);
        assert_eq!(result.unwrap(), Value::Int(3));

        let result = registry.call("floor", &[Value::Float(-3.2)]);
        assert_eq!(result.unwrap(), Value::Int(-4));
    }

    #[test]
    fn test_join() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "join",
            &[Value::List(vec![Value::String("a".to_string()), Value::String("b".to_string()), Value::String("c".to_string())]), Value::String(",".to_string())],
        );
        assert_eq!(result.unwrap(), Value::String("a,b,c".to_string()));
    }

    #[test]
    fn test_first() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "first",
            &[Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
        );
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_last() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "last",
            &[Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
        );
        assert_eq!(result.unwrap(), Value::Int(3));
    }
}
