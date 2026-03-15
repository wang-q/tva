use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use std::collections::HashMap;

mod datetime;
mod hash;
mod io;
mod list;
mod logical;
mod meta;
mod numeric;
mod regex;
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
        self.register("truncate", FunctionInfo::variadic(string::truncate, 2));
        self.register("wordcount", FunctionInfo::fixed(string::wordcount, 1));
        self.register("char_len", FunctionInfo::fixed(string::char_len, 1));

        // Numeric functions
        self.register("abs", FunctionInfo::fixed(numeric::abs, 1));
        self.register("round", FunctionInfo::fixed(numeric::round, 1));
        self.register("min", FunctionInfo::variadic(numeric::min, 1));
        self.register("max", FunctionInfo::variadic(numeric::max, 1));
        self.register("int", FunctionInfo::fixed(numeric::int, 1));
        self.register("float", FunctionInfo::fixed(numeric::float, 1));
        self.register("ceil", FunctionInfo::fixed(numeric::ceil, 1));
        self.register("floor", FunctionInfo::fixed(numeric::floor, 1));
        self.register("sqrt", FunctionInfo::fixed(numeric::sqrt, 1));
        self.register("pow", FunctionInfo::fixed(numeric::pow, 2));
        self.register("sin", FunctionInfo::fixed(numeric::sin, 1));
        self.register("cos", FunctionInfo::fixed(numeric::cos, 1));
        self.register("tan", FunctionInfo::fixed(numeric::tan, 1));
        self.register("ln", FunctionInfo::fixed(numeric::ln, 1));
        self.register("log10", FunctionInfo::fixed(numeric::log10, 1));
        self.register("exp", FunctionInfo::fixed(numeric::exp, 1));

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
        self.register("reverse", FunctionInfo::fixed(list::reverse, 1));
        self.register("nth", FunctionInfo::fixed(list::nth, 2));
        self.register("replace_nth", FunctionInfo::fixed(list::replace_nth, 3));
        self.register("sort", FunctionInfo::fixed(list::sort, 1));
        self.register("sort_by", FunctionInfo::fixed(list::sort_by, 2));
        self.register("unique", FunctionInfo::fixed(list::unique, 1));
        self.register("slice", FunctionInfo::variadic(list::slice, 2));
        self.register("reduce", FunctionInfo::fixed(list::reduce, 3));
        self.register("map", FunctionInfo::fixed(list::map, 2));
        self.register("filter", FunctionInfo::fixed(list::filter, 2));
        self.register("range", FunctionInfo::variadic(list::range, 1));

        // Regex functions
        self.register("regex_match", FunctionInfo::fixed(regex::regex_match, 2));
        self.register(
            "regex_extract",
            FunctionInfo::variadic(regex::regex_extract, 2),
        );
        self.register(
            "regex_replace",
            FunctionInfo::fixed(regex::regex_replace, 3),
        );

        // Hash functions
        self.register("md5", FunctionInfo::fixed(hash::md5, 1));
        self.register("sha256", FunctionInfo::fixed(hash::sha256, 1));
        self.register("base64", FunctionInfo::fixed(hash::base64_encode, 1));
        self.register("unbase64", FunctionInfo::fixed(hash::base64_decode, 1));

        // Datetime functions
        self.register("now", FunctionInfo::fixed(datetime::now, 0));
        self.register("strptime", FunctionInfo::fixed(datetime::strptime, 2));
        self.register("strftime", FunctionInfo::fixed(datetime::strftime, 2));

        // Meta functions
        self.register("type", FunctionInfo::fixed(meta::type_fn, 1));
        self.register("env", FunctionInfo::fixed(meta::env_fn, 1));
        self.register("cwd", FunctionInfo::fixed(meta::cwd_fn, 0));
        self.register("version", FunctionInfo::fixed(meta::version_fn, 0));
        self.register("platform", FunctionInfo::fixed(meta::platform_fn, 0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Error handling tests - registry level tests that should remain in mod.rs
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

    // FunctionRegistry tests
    #[test]
    fn test_registry_new() {
        let registry = FunctionRegistry::new();
        // Should have built-in functions registered
        assert!(registry.contains("trim"));
        assert!(registry.contains("upper"));
        assert!(registry.contains("len"));
    }

    #[test]
    fn test_registry_default() {
        let registry: FunctionRegistry = Default::default();
        // Default should not have any functions
        assert!(!registry.contains("trim"));
    }

    #[test]
    fn test_registry_get_existing() {
        let registry = FunctionRegistry::new();
        let info = registry.get("trim");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.min_arity, 1);
        assert_eq!(info.max_arity, 1);
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = FunctionRegistry::new();
        let info = registry.get("nonexistent");
        assert!(info.is_none());
    }

    #[test]
    fn test_registry_contains() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("trim"));
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_registry_register() {
        let mut registry = FunctionRegistry::default();

        // Define a simple test function
        fn test_fn(args: &[Value]) -> Result<Value, EvalError> {
            Ok(Value::Int(args.len() as i64))
        }

        registry.register("test_fn", FunctionInfo::fixed(test_fn, 2));

        assert!(registry.contains("test_fn"));
        let result = registry.call("test_fn", &[Value::Int(1), Value::Int(2)]);
        assert_eq!(result.unwrap(), Value::Int(2));
    }

    #[test]
    fn test_registry_call_fixed_arity() {
        let registry = FunctionRegistry::new();

        // trim takes exactly 1 argument
        let result = registry.call("trim", &[Value::String("  hello  ".to_string())]);
        assert!(result.is_ok());

        // Too few arguments
        let result = registry.call("trim", &[]);
        assert!(
            matches!(result, Err(EvalError::WrongArity { name, expected, got }) 
            if name == "trim" && expected == 1 && got == 0)
        );

        // Too many arguments
        let result = registry.call(
            "trim",
            &[
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ],
        );
        assert!(
            matches!(result, Err(EvalError::WrongArity { name, expected, got }) 
            if name == "trim" && expected == 1 && got == 2)
        );
    }

    #[test]
    fn test_registry_call_variadic() {
        let registry = FunctionRegistry::new();

        // min is variadic with min_arity = 1
        let result = registry.call("min", &[Value::Int(5)]);
        assert!(result.is_ok());

        let result =
            registry.call("min", &[Value::Int(5), Value::Int(3), Value::Int(8)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(3));

        // Too few arguments
        let result = registry.call("min", &[]);
        assert!(
            matches!(result, Err(EvalError::WrongArity { name, expected, got }) 
            if name == "min" && expected == 1 && got == 0)
        );
    }

    #[test]
    fn test_registry_call_zero_arity() {
        let registry = FunctionRegistry::new();

        // now takes 0 arguments
        let result = registry.call("now", &[]);
        assert!(result.is_ok());

        // Too many arguments
        let result = registry.call("now", &[Value::Int(1)]);
        assert!(
            matches!(result, Err(EvalError::WrongArity { name, expected, got }) 
            if name == "now" && expected == 0 && got == 1)
        );
    }

    #[test]
    fn test_function_info_new() {
        fn dummy(_: &[Value]) -> Result<Value, EvalError> {
            Ok(Value::Null)
        }

        let info = FunctionInfo::new(dummy, 2, 5);
        assert_eq!(info.min_arity, 2);
        assert_eq!(info.max_arity, 5);
    }

    #[test]
    fn test_function_info_fixed() {
        fn dummy(_: &[Value]) -> Result<Value, EvalError> {
            Ok(Value::Null)
        }

        let info = FunctionInfo::fixed(dummy, 3);
        assert_eq!(info.min_arity, 3);
        assert_eq!(info.max_arity, 3);
    }

    #[test]
    fn test_function_info_variadic() {
        fn dummy(_: &[Value]) -> Result<Value, EvalError> {
            Ok(Value::Null)
        }

        let info = FunctionInfo::variadic(dummy, 2);
        assert_eq!(info.min_arity, 2);
        assert_eq!(info.max_arity, usize::MAX);
    }

    // Test all function categories are registered
    #[test]
    fn test_string_functions_registered() {
        let registry = FunctionRegistry::new();
        let string_funcs = [
            "trim",
            "upper",
            "lower",
            "len",
            "substr",
            "split",
            "contains",
            "starts_with",
            "ends_with",
            "replace",
            "truncate",
            "wordcount",
            "char_len",
        ];
        for func in &string_funcs {
            assert!(
                registry.contains(func),
                "String function '{}' should be registered",
                func
            );
        }
    }

    #[test]
    fn test_numeric_functions_registered() {
        let registry = FunctionRegistry::new();
        let numeric_funcs = [
            "abs", "round", "min", "max", "int", "float", "ceil", "floor", "sqrt",
            "pow", "sin", "cos", "tan", "ln", "log10", "exp",
        ];
        for func in &numeric_funcs {
            assert!(
                registry.contains(func),
                "Numeric function '{}' should be registered",
                func
            );
        }
    }

    #[test]
    fn test_logical_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("if"));
        assert!(registry.contains("default"));
    }

    #[test]
    fn test_list_functions_registered() {
        let registry = FunctionRegistry::new();
        let list_funcs = [
            "join",
            "first",
            "last",
            "reverse",
            "nth",
            "replace_nth",
            "sort",
            "sort_by",
            "unique",
            "slice",
            "reduce",
            "map",
            "filter",
            "range",
        ];
        for func in &list_funcs {
            assert!(
                registry.contains(func),
                "List function '{}' should be registered",
                func
            );
        }
    }

    #[test]
    fn test_hash_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("md5"));
        assert!(registry.contains("sha256"));
        assert!(registry.contains("base64"));
        assert!(registry.contains("unbase64"));
    }

    #[test]
    fn test_datetime_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("now"));
        assert!(registry.contains("strptime"));
        assert!(registry.contains("strftime"));
    }

    #[test]
    fn test_meta_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("type"));
        assert!(registry.contains("env"));
        assert!(registry.contains("cwd"));
        assert!(registry.contains("version"));
        assert!(registry.contains("platform"));
    }

    #[test]
    fn test_regex_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("regex_match"));
        assert!(registry.contains("regex_extract"));
        assert!(registry.contains("regex_replace"));
    }

    #[test]
    fn test_io_functions_registered() {
        let registry = FunctionRegistry::new();
        assert!(registry.contains("print"));
        assert!(registry.contains("eprint"));
    }

    // Integration tests for function calls
    #[test]
    fn test_call_trim() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("trim", &[Value::String("  hello  ".to_string())])
            .unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_call_upper() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("upper", &[Value::String("hello".to_string())])
            .unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_call_len() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("len", &[Value::String("hello".to_string())])
            .unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_call_abs() {
        let registry = FunctionRegistry::new();
        let result = registry.call("abs", &[Value::Int(-42)]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_call_min() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("min", &[Value::Int(5), Value::Int(3), Value::Int(8)])
            .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_call_max() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("max", &[Value::Int(5), Value::Int(3), Value::Int(8)])
            .unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_call_join() {
        let registry = FunctionRegistry::new();
        let list = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]);
        let result = registry
            .call("join", &[list, Value::String(",".to_string())])
            .unwrap();
        assert_eq!(result, Value::String("a,b,c".to_string()));
    }

    #[test]
    fn test_call_type() {
        let registry = FunctionRegistry::new();
        let result = registry.call("type", &[Value::Int(42)]).unwrap();
        assert_eq!(result, Value::String("int".to_string()));
    }

    #[test]
    fn test_call_md5() {
        let registry = FunctionRegistry::new();
        let result = registry
            .call("md5", &[Value::String("hello".to_string())])
            .unwrap();
        assert_eq!(
            result,
            Value::String("5d41402abc4b2a76b9719d911017c592".to_string())
        );
    }
}
