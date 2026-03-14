/// AST for expression parsing
/// Supports: column refs, variables, literals, arithmetic, comparison, logical ops, pipes

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Column reference: @1, @name
    ColumnRef(ColumnRef),
    /// Variable reference: @var_name (bound by 'as')
    Variable(String),
    /// Lambda parameter reference (resolved during lambda evaluation)
    LambdaParam(String),
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
    /// Lambda expression: x => x + 1 or (x, y) => x + y
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
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
    // Comparison (numeric)
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=
    // Comparison (string)
    StrEq, // eq
    StrNe, // ne
    StrLt, // lt
    StrLe, // le
    StrGt, // gt
    StrGe, // ge
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

    /// Get the last expression from a Block, or return self if not a Block
    pub fn last_expr(&self) -> &Expr {
        match self {
            Expr::Block(exprs) => exprs.last().unwrap_or(self),
            _ => self,
        }
    }

    /// Format the expression as a string for display (e.g., as column header)
    pub fn format(&self) -> String {
        match self {
            Expr::ColumnRef(ColumnRef::Index(idx)) => format!("@{}", idx),
            Expr::ColumnRef(ColumnRef::Name(name)) => format!("@{}", name),
            Expr::Variable(name) => format!("@{}", name),
            Expr::LambdaParam(name) => name.clone(),
            Expr::Int(n) => n.to_string(),
            Expr::Float(n) => n.to_string(),
            Expr::String(s) => format!("\"{}\"", s),
            Expr::Bool(b) => b.to_string(),
            Expr::Null => "null".to_string(),
            Expr::List(items) => {
                let items_str: Vec<String> = items.iter().map(|e| e.format()).collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::Unary { op, expr } => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "not ",
                };
                format!("{}{}", op_str, expr.format())
            }
            Expr::Binary { op, left, right } => {
                let op_str = match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Div => " / ",
                    BinaryOp::Mod => " % ",
                    BinaryOp::Pow => " ** ",
                    BinaryOp::Concat => " ++ ",
                    BinaryOp::Eq => " == ",
                    BinaryOp::Ne => " != ",
                    BinaryOp::Lt => " < ",
                    BinaryOp::Le => " <= ",
                    BinaryOp::Gt => " > ",
                    BinaryOp::Ge => " >= ",
                    BinaryOp::StrEq => " eq ",
                    BinaryOp::StrNe => " ne ",
                    BinaryOp::StrLt => " lt ",
                    BinaryOp::StrLe => " le ",
                    BinaryOp::StrGt => " gt ",
                    BinaryOp::StrGe => " ge ",
                    BinaryOp::And => " and ",
                    BinaryOp::Or => " or ",
                };
                format!("{}{}{}", left.format(), op_str, right.format())
            }
            Expr::Call { name, args } => {
                let args_str: Vec<String> = args.iter().map(|e| e.format()).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            Expr::MethodCall { object, name, args } => {
                let args_str: Vec<String> = args.iter().map(|e| e.format()).collect();
                if args_str.is_empty() {
                    format!("{}.{}()", object.format(), name)
                } else {
                    format!("{}.{}({})", object.format(), name, args_str.join(", "))
                }
            }
            Expr::Pipe { left, right } => {
                let right_str = match right.as_ref() {
                    PipeRight::Call { name, args } => {
                        let args_str: Vec<String> =
                            args.iter().map(|e| e.format()).collect();
                        if args_str.is_empty() {
                            format!("{}()", name)
                        } else {
                            format!("{}({})", name, args_str.join(", "))
                        }
                    }
                    PipeRight::CallWithPlaceholder { name, args } => {
                        let args_str: Vec<String> =
                            args.iter().map(|e| e.format()).collect();
                        format!("{}(_, {})", name, args_str.join(", "))
                    }
                };
                format!("{} | {}", left.format(), right_str)
            }
            Expr::Bind { expr, name } => {
                format!("{} as @{}", expr.format(), name)
            }
            Expr::Lambda { params, body } => {
                if params.len() == 1 {
                    format!("{} => {}", params[0], body.format())
                } else {
                    format!("({}) => {}", params.join(", "), body.format())
                }
            }
            Expr::Block(_) => {
                // Block should not be formatted directly, use last_expr() first
                self.last_expr().format()
            }
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

    #[test]
    fn test_last_expr_single() {
        let expr = Expr::col_name("price");
        assert!(
            matches!(expr.last_expr(), Expr::ColumnRef(ColumnRef::Name(n)) if n == "price")
        );
    }

    #[test]
    fn test_last_expr_block() {
        let exprs = vec![
            Expr::col_name("price"),
            Expr::binary(BinaryOp::Mul, Expr::col_name("qty"), Expr::int(2)),
        ];
        let block = Expr::Block(exprs);
        // last_expr should return the last expression in the block
        assert!(matches!(
            block.last_expr(),
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn test_format_column_ref() {
        assert_eq!(Expr::col_idx(1).format(), "@1");
        assert_eq!(Expr::col_name("price").format(), "@price");
    }

    #[test]
    fn test_format_variable() {
        let var = Expr::Variable("total".to_string());
        assert_eq!(var.format(), "@total");
    }

    #[test]
    fn test_format_literals() {
        assert_eq!(Expr::int(42).format(), "42");
        assert_eq!(Expr::float(3.14).format(), "3.14");
        assert_eq!(Expr::string("hello").format(), "\"hello\"");
        assert_eq!(Expr::bool(true).format(), "true");
        assert_eq!(Expr::null().format(), "null");
    }

    #[test]
    fn test_format_list() {
        let list = Expr::List(vec![Expr::int(1), Expr::int(2), Expr::int(3)]);
        assert_eq!(list.format(), "[1, 2, 3]");
    }

    #[test]
    fn test_format_unary() {
        let neg = Expr::unary(UnaryOp::Neg, Expr::int(5));
        assert_eq!(neg.format(), "-5");

        let not = Expr::unary(UnaryOp::Not, Expr::bool(true));
        assert_eq!(not.format(), "not true");
    }

    #[test]
    fn test_format_binary() {
        let add = Expr::binary(BinaryOp::Add, Expr::int(1), Expr::int(2));
        assert_eq!(add.format(), "1 + 2");

        let mul = Expr::binary(BinaryOp::Mul, Expr::col_name("price"), Expr::int(2));
        assert_eq!(mul.format(), "@price * 2");

        let concat = Expr::binary(
            BinaryOp::Concat,
            Expr::string("hello"),
            Expr::string("world"),
        );
        assert_eq!(concat.format(), "\"hello\" ++ \"world\"");
    }

    #[test]
    fn test_format_call() {
        let call = Expr::call("upper", vec![Expr::col_name("name")]);
        assert_eq!(call.format(), "upper(@name)");

        let call_multi = Expr::call(
            "substr",
            vec![Expr::col_name("name"), Expr::int(0), Expr::int(5)],
        );
        assert_eq!(call_multi.format(), "substr(@name, 0, 5)");
    }

    #[test]
    fn test_format_method_call() {
        let method = Expr::MethodCall {
            object: Box::new(Expr::col_name("name")),
            name: "upper".to_string(),
            args: vec![],
        };
        assert_eq!(method.format(), "@name.upper()");

        let method_with_args = Expr::MethodCall {
            object: Box::new(Expr::col_name("name")),
            name: "substr".to_string(),
            args: vec![Expr::int(0), Expr::int(5)],
        };
        assert_eq!(method_with_args.format(), "@name.substr(0, 5)");
    }

    #[test]
    fn test_format_pipe() {
        let pipe = Expr::Pipe {
            left: Box::new(Expr::col_name("name")),
            right: Box::new(PipeRight::Call {
                name: "upper".to_string(),
                args: vec![],
            }),
        };
        assert_eq!(pipe.format(), "@name | upper()");

        let pipe_with_placeholder = Expr::Pipe {
            left: Box::new(Expr::col_name("name")),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "substr".to_string(),
                args: vec![Expr::int(0), Expr::int(5)],
            }),
        };
        assert_eq!(pipe_with_placeholder.format(), "@name | substr(_, 0, 5)");
    }

    #[test]
    fn test_format_bind() {
        let bind = Expr::Bind {
            expr: Box::new(Expr::binary(
                BinaryOp::Mul,
                Expr::col_name("price"),
                Expr::int(2),
            )),
            name: "total".to_string(),
        };
        assert_eq!(bind.format(), "@price * 2 as @total");
    }

    #[test]
    fn test_format_lambda() {
        let lambda_single = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::binary(
                BinaryOp::Mul,
                Expr::LambdaParam("x".to_string()),
                Expr::int(2),
            )),
        };
        assert_eq!(lambda_single.format(), "x => x * 2");

        let lambda_multi = Expr::Lambda {
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Expr::binary(
                BinaryOp::Add,
                Expr::LambdaParam("x".to_string()),
                Expr::LambdaParam("y".to_string()),
            )),
        };
        assert_eq!(lambda_multi.format(), "(x, y) => x + y");
    }

    #[test]
    fn test_format_block() {
        let exprs = vec![
            Expr::col_name("price"),
            Expr::binary(BinaryOp::Mul, Expr::col_name("qty"), Expr::int(2)),
        ];
        let block = Expr::Block(exprs);
        // Block should format as its last expression
        assert_eq!(block.format(), "@qty * 2");
    }
}
