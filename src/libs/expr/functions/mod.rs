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
}
