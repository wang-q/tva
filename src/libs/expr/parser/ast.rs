/// AST for expression parsing
/// Supports: column refs, variables, literals, arithmetic, comparison, logical ops, pipes

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Column reference: @1, @name
    ColumnRef(ColumnRef),
    /// Variable reference: @var_name (bound by 'as')
    Variable(String),
    /// Integer literal: 123
    Int(i64),
    /// Float literal: 3.14
    Float(f64),
    /// String literal: "hello"
    String(String),
    /// Boolean literal: true, false
    Bool(bool),
    /// Null literal
    Null,
    /// List literal: [1, 2, 3]
    List(Vec<Expr>),
    /// Unary operation: -x, !x
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// Binary operation: @1 + @2
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Function call: func(arg1, arg2)
    Call { name: String, args: Vec<Expr> },
    /// Method call: expr.method(arg1, arg2) - syntactic sugar for method(expr, arg1, arg2)
    MethodCall {
        object: Box<Expr>,
        name: String,
        args: Vec<Expr>,
    },
    /// Pipe expression: expr | func() or expr | func(_, arg)
    /// The first argument is implicitly the left side of the pipe
    Pipe {
        left: Box<Expr>,
        right: Box<PipeRight>,
    },
    /// Variable binding: expr as @name
    Bind { expr: Box<Expr>, name: String },
    /// Multiple expressions separated by semicolons
    Block(Vec<Expr>),
}

/// Right-hand side of a pipe expression
#[derive(Debug, Clone, PartialEq)]
pub enum PipeRight {
    /// Function call with implicit first argument: func()
    Call { name: String, args: Vec<Expr> },
    /// Function call with placeholder: func(_, arg2)
    /// The placeholder _ is replaced with the pipe's left value
    CallWithPlaceholder { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRef {
    /// Index-based: @1, @2
    Index(usize),
    /// Name-based: @col_name
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, // -
    Not, // !
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // **
    // String concatenation
    Concat, // ++
    // Comparison
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=
    // Logical
    And, // && / and
    Or,  // || / or
}

impl Expr {
    /// Create a column reference by index (1-based)
    pub fn col_idx(idx: usize) -> Self {
        Expr::ColumnRef(ColumnRef::Index(idx))
    }

    /// Create a column reference by name
    pub fn col_name(name: impl Into<String>) -> Self {
        Expr::ColumnRef(ColumnRef::Name(name.into()))
    }

    /// Create an integer literal
    pub fn int(n: i64) -> Self {
        Expr::Int(n)
    }

    /// Create a float literal
    pub fn float(n: f64) -> Self {
        Expr::Float(n)
    }

    /// Create a string literal
    pub fn string(s: impl Into<String>) -> Self {
        Expr::String(s.into())
    }

    /// Create a boolean literal
    pub fn bool(b: bool) -> Self {
        Expr::Bool(b)
    }

    /// Create a null literal
    pub fn null() -> Self {
        Expr::Null
    }

    /// Create a unary expression
    pub fn unary(op: UnaryOp, expr: Expr) -> Self {
        Expr::Unary {
            op,
            expr: Box::new(expr),
        }
    }

    /// Create a binary expression
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Self {
        Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Create a function call
    pub fn call(name: impl Into<String>, args: Vec<Expr>) -> Self {
        Expr::Call {
            name: name.into(),
            args,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_creation() {
        let expr = Expr::binary(BinaryOp::Add, Expr::col_idx(1), Expr::int(10));

        match expr {
            Expr::Binary { op, left, right } => {
                assert_eq!(op, BinaryOp::Add);
                assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(1))));
                assert!(matches!(*right, Expr::Int(10)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_unary_expr() {
        let expr = Expr::unary(UnaryOp::Neg, Expr::int(5));
        assert!(matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Neg,
                ..
            }
        ));
    }

    #[test]
    fn test_comparison_expr() {
        let expr = Expr::binary(BinaryOp::Gt, Expr::col_idx(1), Expr::float(3.14));
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));
    }
}
