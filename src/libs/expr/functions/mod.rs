use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use std::collections::HashMap;

mod datetime;
mod hash;
mod io;
mod list;
mod logical;
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
        self.register("sort", FunctionInfo::fixed(list::sort, 1));
        self.register("unique", FunctionInfo::fixed(list::unique, 1));
        self.register("slice", FunctionInfo::variadic(list::slice, 2));
        self.register("reduce", FunctionInfo::fixed(list::reduce, 3));
        self.register("map", FunctionInfo::fixed(list::map, 2));
        self.register("filter", FunctionInfo::fixed(list::filter, 2));

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
            &[
                Value::String("a".to_string()),
                Value::Int(1),
                Value::Bool(true),
            ],
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
            &[
                Value::String("hello world".to_string()),
                Value::Int(0),
                Value::Int(5),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_split() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "split",
            &[
                Value::String("a,b,c".to_string()),
                Value::String(",".to_string()),
            ],
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
            &[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_starts_with() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "starts_with",
            &[
                Value::String("hello".to_string()),
                Value::String("he".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_ends_with() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "ends_with",
            &[
                Value::String("hello".to_string()),
                Value::String("lo".to_string()),
            ],
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
        let result = registry.call(
            "default",
            &[Value::Null, Value::String("fallback".to_string())],
        );
        assert_eq!(result.unwrap(), Value::String("fallback".to_string()));

        // false returns fallback
        let result = registry.call("default", &[Value::Bool(false), Value::Int(0)]);
        assert_eq!(result.unwrap(), Value::Int(0));

        // non-null returns original
        let result = registry.call(
            "default",
            &[
                Value::String("value".to_string()),
                Value::String("fallback".to_string()),
            ],
        );
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
            &[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
                Value::String("rust".to_string()),
            ],
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
            &[
                Value::List(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                    Value::String("c".to_string()),
                ]),
                Value::String(",".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("a,b,c".to_string()));
    }

    #[test]
    fn test_first() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "first",
            &[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );
        assert_eq!(result.unwrap(), Value::Int(1));
    }

    #[test]
    fn test_last() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "last",
            &[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );
        assert_eq!(result.unwrap(), Value::Int(3));
    }

    #[test]
    fn test_sqrt() {
        let registry = FunctionRegistry::new();
        let result = registry.call("sqrt", &[Value::Float(16.0)]);
        assert_eq!(result.unwrap(), Value::Float(4.0));

        let result = registry.call("sqrt", &[Value::Int(9)]);
        assert_eq!(result.unwrap(), Value::Float(3.0));

        // Negative number error
        let result = registry.call("sqrt", &[Value::Float(-4.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pow() {
        let registry = FunctionRegistry::new();
        let result = registry.call("pow", &[Value::Float(2.0), Value::Float(3.0)]);
        assert_eq!(result.unwrap(), Value::Float(8.0));

        let result = registry.call("pow", &[Value::Int(2), Value::Int(10)]);
        assert_eq!(result.unwrap(), Value::Float(1024.0));
    }

    #[test]
    fn test_truncate() {
        let registry = FunctionRegistry::new();
        // "hello world" is 11 bytes, limit 8, end "..." is 3 bytes
        // So we take 8-3=5 bytes: "hello" + "..." = "hello..."
        let result = registry.call(
            "truncate",
            &[Value::String("hello world".to_string()), Value::Int(8)],
        );
        assert_eq!(result.unwrap(), Value::String("hello...".to_string()));

        // Custom ending: limit 8, end ">>" is 2 bytes
        // So we take 8-2=6 bytes: "hello " + ">>" = "hello >>"
        let result = registry.call(
            "truncate",
            &[
                Value::String("hello world".to_string()),
                Value::Int(8),
                Value::String(">>".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("hello >>".to_string()));

        // String shorter than limit - return as-is
        let result = registry.call(
            "truncate",
            &[Value::String("hi".to_string()), Value::Int(10)],
        );
        assert_eq!(result.unwrap(), Value::String("hi".to_string()));
    }

    #[test]
    fn test_reverse() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "reverse",
            &[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)])
        );

        // Empty list
        let result = registry.call("reverse", &[Value::List(vec![])]);
        assert_eq!(result.unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_wordcount() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "wordcount",
            &[Value::String("hello world foo bar".to_string())],
        );
        assert_eq!(result.unwrap(), Value::Int(4));

        let result = registry.call(
            "wordcount",
            &[Value::String("   multiple   spaces   ".to_string())],
        );
        assert_eq!(result.unwrap(), Value::Int(2));

        let result = registry.call("wordcount", &[Value::String("".to_string())]);
        assert_eq!(result.unwrap(), Value::Int(0));
    }

    #[test]
    fn test_char_len() {
        let registry = FunctionRegistry::new();
        let result = registry.call("char_len", &[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::Int(5));

        // UTF-8 characters
        let result = registry.call("char_len", &[Value::String("你好世界".to_string())]);
        assert_eq!(result.unwrap(), Value::Int(4));
    }

    #[test]
    fn test_nth() {
        let registry = FunctionRegistry::new();
        let list = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);

        let result = registry.call("nth", &[list.clone(), Value::Int(0)]);
        assert_eq!(result.unwrap(), Value::Int(10));

        let result = registry.call("nth", &[list.clone(), Value::Int(2)]);
        assert_eq!(result.unwrap(), Value::Int(30));

        // Out of bounds
        let result = registry.call("nth", &[list.clone(), Value::Int(5)]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_sort() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "sort",
            &[Value::List(vec![
                Value::Int(3),
                Value::Int(1),
                Value::Int(2),
            ])],
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );

        // Mixed types
        let result = registry.call(
            "sort",
            &[Value::List(vec![
                Value::Float(2.5),
                Value::Int(1),
                Value::Float(1.5),
            ])],
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(1), Value::Float(1.5), Value::Float(2.5)])
        );
    }

    #[test]
    fn test_unique() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "unique",
            &[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(1),
                Value::Int(3),
            ])],
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );

        // Strings
        let result = registry.call(
            "unique",
            &[Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("a".to_string()),
            ])],
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ])
        );
    }

    #[test]
    fn test_slice() {
        let registry = FunctionRegistry::new();
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);

        // slice(list, 1, 3) -> [2, 3]
        let result =
            registry.call("slice", &[list.clone(), Value::Int(1), Value::Int(3)]);
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(2), Value::Int(3)])
        );

        // slice(list, 2) -> [3, 4, 5]
        let result = registry.call("slice", &[list.clone(), Value::Int(2)]);
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(3), Value::Int(4), Value::Int(5)])
        );

        // Out of bounds handling
        let result = registry.call("slice", &[list.clone(), Value::Int(10)]);
        assert_eq!(result.unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_regex_match() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "regex_match",
            &[
                Value::String("hello world".to_string()),
                Value::String(r"hello.*".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Bool(true));

        let result = registry.call(
            "regex_match",
            &[
                Value::String("hello world".to_string()),
                Value::String(r"^foo".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_regex_extract() {
        let registry = FunctionRegistry::new();
        // Extract full match (group 0)
        let result = registry.call(
            "regex_extract",
            &[
                Value::String("hello 123 world".to_string()),
                Value::String(r"\d+".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("123".to_string()));

        // Extract specific group
        let result = registry.call(
            "regex_extract",
            &[
                Value::String("hello 123 world".to_string()),
                Value::String(r"(\d+)".to_string()),
                Value::Int(1),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("123".to_string()));

        // No match
        let result = registry.call(
            "regex_extract",
            &[
                Value::String("hello world".to_string()),
                Value::String(r"\d+".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_regex_replace() {
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "regex_replace",
            &[
                Value::String("hello 123 world 456".to_string()),
                Value::String(r"\d+".to_string()),
                Value::String("XXX".to_string()),
            ],
        );
        assert_eq!(
            result.unwrap(),
            Value::String("hello XXX world XXX".to_string())
        );

        // Replace with capture group reference
        let result = registry.call(
            "regex_replace",
            &[
                Value::String("hello world".to_string()),
                Value::String(r"(world)".to_string()),
                Value::String("$1!".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("hello world!".to_string()));
    }

    #[test]
    fn test_md5() {
        let registry = FunctionRegistry::new();
        let result = registry.call("md5", &[Value::String("hello".to_string())]);
        // MD5 of "hello" is 5d41402abc4b2a76b9719d911017c592
        assert_eq!(
            result.unwrap(),
            Value::String("5d41402abc4b2a76b9719d911017c592".to_string())
        );
    }

    #[test]
    fn test_sha256() {
        let registry = FunctionRegistry::new();
        let result = registry.call("sha256", &[Value::String("hello".to_string())]);
        // SHA256 of "hello"
        assert_eq!(
            result.unwrap(),
            Value::String(
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_base64() {
        let registry = FunctionRegistry::new();
        let result = registry.call("base64", &[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("aGVsbG8=".to_string()));

        // Empty string
        let result = registry.call("base64", &[Value::String("".to_string())]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_unbase64() {
        let registry = FunctionRegistry::new();
        let result = registry.call("unbase64", &[Value::String("aGVsbG8=".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));

        // Round-trip
        let encoded = registry
            .call("base64", &[Value::String("test 123".to_string())])
            .unwrap();
        let result = registry.call("unbase64", &[encoded]);
        assert_eq!(result.unwrap(), Value::String("test 123".to_string()));
    }

    #[test]
    fn test_now() {
        let registry = FunctionRegistry::new();
        let result = registry.call("now", &[]);
        assert!(result.is_ok());
        // Check it's a DateTime
        match result.unwrap() {
            Value::DateTime(_) => {}
            _ => panic!("now() should return DateTime"),
        }
    }

    #[test]
    fn test_strptime() {
        use chrono::Datelike;
        let registry = FunctionRegistry::new();
        let result = registry.call(
            "strptime",
            &[
                Value::String("2024-03-15 14:30:00".to_string()),
                Value::String("%Y-%m-%d %H:%M:%S".to_string()),
            ],
        );
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
        let registry = FunctionRegistry::new();
        // Parse and format
        let dt = registry
            .call(
                "strptime",
                &[
                    Value::String("2024-03-15 14:30:00".to_string()),
                    Value::String("%Y-%m-%d %H:%M:%S".to_string()),
                ],
            )
            .unwrap();

        let result =
            registry.call("strftime", &[dt, Value::String("%Y/%m/%d".to_string())]);
        assert_eq!(result.unwrap(), Value::String("2024/03/15".to_string()));

        // Format from string
        let result = registry.call(
            "strftime",
            &[
                Value::String("2024-03-15T14:30:00Z".to_string()),
                Value::String("%d-%m-%Y".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("15-03-2024".to_string()));
    }

    #[test]
    fn test_reduce() {
        let registry = FunctionRegistry::new();

        // Sum
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::Int(0),
                Value::String("+".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Int(6));

        // Product
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![Value::Int(2), Value::Int(3), Value::Int(4)]),
                Value::Int(1),
                Value::String("*".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Int(24));

        // String concat
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                    Value::String("c".to_string()),
                ]),
                Value::String("".to_string()),
                Value::String("+".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::String("abc".to_string()));

        // Empty list returns initial value
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![]),
                Value::Int(42),
                Value::String("+".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Int(42));

        // Min
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![Value::Int(5), Value::Int(2), Value::Int(8)]),
                Value::Int(10),
                Value::String("min".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Int(2));

        // Max
        let result = registry.call(
            "reduce",
            &[
                Value::List(vec![Value::Int(5), Value::Int(2), Value::Int(8)]),
                Value::Int(0),
                Value::String("max".to_string()),
            ],
        );
        assert_eq!(result.unwrap(), Value::Int(8));
    }

    // Trigonometric function tests
    #[test]
    fn test_sin() {
        let registry = FunctionRegistry::new();
        let result = registry.call("sin", &[Value::Float(0.0)]);
        assert!((result.unwrap().as_float().unwrap() - 0.0).abs() < 1e-10);

        let result = registry.call("sin", &[Value::Float(std::f64::consts::PI / 2.0)]);
        assert!((result.unwrap().as_float().unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cos() {
        let registry = FunctionRegistry::new();
        let result = registry.call("cos", &[Value::Float(0.0)]);
        assert!((result.unwrap().as_float().unwrap() - 1.0).abs() < 1e-10);

        let result = registry.call("cos", &[Value::Float(std::f64::consts::PI)]);
        assert!((result.unwrap().as_float().unwrap() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_tan() {
        let registry = FunctionRegistry::new();
        let result = registry.call("tan", &[Value::Float(0.0)]);
        assert!((result.unwrap().as_float().unwrap() - 0.0).abs() < 1e-10);

        let result = registry.call("tan", &[Value::Float(std::f64::consts::PI / 4.0)]);
        assert!((result.unwrap().as_float().unwrap() - 1.0).abs() < 1e-10);
    }

    // Logarithmic and exponential function tests
    #[test]
    fn test_ln() {
        let registry = FunctionRegistry::new();
        let result = registry.call("ln", &[Value::Float(1.0)]);
        assert!((result.unwrap().as_float().unwrap() - 0.0).abs() < 1e-10);

        let result = registry.call("ln", &[Value::Float(std::f64::consts::E)]);
        assert!((result.unwrap().as_float().unwrap() - 1.0).abs() < 1e-10);

        // Error on non-positive
        let result = registry.call("ln", &[Value::Float(-1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_log10() {
        let registry = FunctionRegistry::new();
        let result = registry.call("log10", &[Value::Float(1.0)]);
        assert!((result.unwrap().as_float().unwrap() - 0.0).abs() < 1e-10);

        let result = registry.call("log10", &[Value::Float(100.0)]);
        assert!((result.unwrap().as_float().unwrap() - 2.0).abs() < 1e-10);

        // Error on non-positive
        let result = registry.call("log10", &[Value::Float(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_exp() {
        let registry = FunctionRegistry::new();
        let result = registry.call("exp", &[Value::Float(0.0)]);
        assert!((result.unwrap().as_float().unwrap() - 1.0).abs() < 1e-10);

        let result = registry.call("exp", &[Value::Float(1.0)]);
        assert!(
            (result.unwrap().as_float().unwrap() - std::f64::consts::E).abs() < 1e-10
        );
    }
}
