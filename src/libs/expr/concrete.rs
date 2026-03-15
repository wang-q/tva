/// Concrete expression - optimized for execution
/// All names are resolved to indices, constants are pre-computed
use crate::libs::expr::functions::Function;
use crate::libs::expr::parser::ast::{BinaryOp, UnaryOp};
use crate::libs::expr::runtime::value::Value;

/// Concrete expression enum - optimized for execution
/// Unlike Expr, this has no string lookups at runtime
#[derive(Debug, Clone)]
pub enum ConcreteExpr {
    /// Column reference by index (0-based, resolved at compile time)
    Column(usize),
    /// Pre-computed constant value
    Constant(Value),
    /// Variable reference (bound by 'as', index into variables array)
    Variable(usize),
    /// Lambda parameter reference (index into lambda params)
    LambdaParam(usize),
    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<ConcreteExpr>,
    },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<ConcreteExpr>,
        right: Box<ConcreteExpr>,
    },
    /// Function call with resolved function pointer
    Call {
        func: Function,
        args: Vec<ConcreteExpr>,
    },
    /// Method call (resolved to function call with object as first arg)
    MethodCall {
        object: Box<ConcreteExpr>,
        func: Function,
        args: Vec<ConcreteExpr>,
    },
    /// Pipe expression
    Pipe {
        left: Box<ConcreteExpr>,
        right: Box<ConcretePipeRight>,
    },
    /// Variable binding
    Bind {
        expr: Box<ConcreteExpr>,
        var_index: usize,
    },
    /// Lambda expression with concrete body
    Lambda {
        params: Vec<String>, // Keep names for parameter matching
        body: Box<ConcreteExpr>,
    },
    /// Block of expressions
    Block(Vec<ConcreteExpr>),
}

/// Right-hand side of a pipe expression (concrete version)
#[derive(Debug, Clone)]
pub enum ConcretePipeRight {
    /// Function call with implicit first argument
    Call {
        func: Function,
        args: Vec<ConcreteExpr>,
    },
    /// Function call with placeholder
    CallWithPlaceholder {
        func: Function,
        placeholder_index: usize, // Which arg is the placeholder
        args: Vec<ConcreteExpr>,
    },
}

/// Compilation context for concretization
pub struct CompileContext<'a> {
    /// Headers for column name resolution
    pub headers: &'a [String],
    /// Variable name to index mapping
    pub variables: Vec<String>,
    /// Lambda parameter stack (for nested lambdas)
    pub lambda_params: Vec<Vec<String>>,
}

impl<'a> CompileContext<'a> {
    pub fn new(headers: &'a [String]) -> Self {
        Self {
            headers,
            variables: Vec::new(),
            lambda_params: Vec::new(),
        }
    }

    /// Get or create variable index
    pub fn get_or_create_var(&mut self, name: &str) -> usize {
        if let Some(idx) = self.variables.iter().position(|v| v == name) {
            idx
        } else {
            let idx = self.variables.len();
            self.variables.push(name.to_string());
            idx
        }
    }

    /// Get variable index if exists
    pub fn get_var(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| v == name)
    }

    /// Get lambda parameter index (searches from innermost scope)
    pub fn get_lambda_param(&self, name: &str) -> Option<usize> {
        for params in self.lambda_params.iter().rev() {
            if let Some(idx) = params.iter().position(|p| p == name) {
                return Some(idx);
            }
        }
        None
    }

    /// Push lambda scope
    pub fn push_lambda_scope(&mut self, params: Vec<String>) {
        self.lambda_params.push(params);
    }

    /// Pop lambda scope
    pub fn pop_lambda_scope(&mut self) {
        self.lambda_params.pop();
    }

    /// Resolve column name to index (0-based)
    pub fn resolve_column(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }
}
