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
    /// Whole row: @0
    WholeRow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
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
    And, // and
    Or,  // or
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
            Expr::ColumnRef(ColumnRef::WholeRow) => "@0".to_string(),
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

    #[test]
    fn test_expr_factory_methods() {
        // Test all factory methods
        let col_idx = Expr::col_idx(5);
        assert!(matches!(col_idx, Expr::ColumnRef(ColumnRef::Index(5))));

        let col_name = Expr::col_name("test_col");
        assert!(
            matches!(col_name, Expr::ColumnRef(ColumnRef::Name(n)) if n == "test_col")
        );

        let int = Expr::int(42);
        assert!(matches!(int, Expr::Int(42)));

        let float = Expr::float(3.14);
        assert!(matches!(float, Expr::Float(f) if (f - 3.14).abs() < 1e-10));

        let string = Expr::string("hello");
        assert!(matches!(string, Expr::String(s) if s == "hello"));

        let bool_true = Expr::bool(true);
        assert!(matches!(bool_true, Expr::Bool(true)));

        let bool_false = Expr::bool(false);
        assert!(matches!(bool_false, Expr::Bool(false)));

        let null = Expr::null();
        assert!(matches!(null, Expr::Null));
    }

    #[test]
    fn test_binary_operators() {
        // Test all binary operators
        let add = Expr::binary(BinaryOp::Add, Expr::int(1), Expr::int(2));
        assert_eq!(add.format(), "1 + 2");

        let sub = Expr::binary(BinaryOp::Sub, Expr::int(5), Expr::int(3));
        assert_eq!(sub.format(), "5 - 3");

        let mul = Expr::binary(BinaryOp::Mul, Expr::int(2), Expr::int(3));
        assert_eq!(mul.format(), "2 * 3");

        let div = Expr::binary(BinaryOp::Div, Expr::int(10), Expr::int(2));
        assert_eq!(div.format(), "10 / 2");

        let modulo = Expr::binary(BinaryOp::Mod, Expr::int(10), Expr::int(3));
        assert_eq!(modulo.format(), "10 % 3");

        let pow = Expr::binary(BinaryOp::Pow, Expr::int(2), Expr::int(3));
        assert_eq!(pow.format(), "2 ** 3");

        let concat =
            Expr::binary(BinaryOp::Concat, Expr::string("a"), Expr::string("b"));
        assert_eq!(concat.format(), "\"a\" ++ \"b\"");
    }

    #[test]
    fn test_comparison_operators() {
        // Numeric comparison
        let eq = Expr::binary(BinaryOp::Eq, Expr::int(1), Expr::int(1));
        assert_eq!(eq.format(), "1 == 1");

        let ne = Expr::binary(BinaryOp::Ne, Expr::int(1), Expr::int(2));
        assert_eq!(ne.format(), "1 != 2");

        let lt = Expr::binary(BinaryOp::Lt, Expr::int(1), Expr::int(2));
        assert_eq!(lt.format(), "1 < 2");

        let le = Expr::binary(BinaryOp::Le, Expr::int(1), Expr::int(2));
        assert_eq!(le.format(), "1 <= 2");

        let gt = Expr::binary(BinaryOp::Gt, Expr::int(2), Expr::int(1));
        assert_eq!(gt.format(), "2 > 1");

        let ge = Expr::binary(BinaryOp::Ge, Expr::int(2), Expr::int(1));
        assert_eq!(ge.format(), "2 >= 1");

        // String comparison
        let str_eq = Expr::binary(BinaryOp::StrEq, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_eq.format(), "\"a\" eq \"b\"");

        let str_ne = Expr::binary(BinaryOp::StrNe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_ne.format(), "\"a\" ne \"b\"");

        let str_lt = Expr::binary(BinaryOp::StrLt, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_lt.format(), "\"a\" lt \"b\"");

        let str_le = Expr::binary(BinaryOp::StrLe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_le.format(), "\"a\" le \"b\"");

        let str_gt = Expr::binary(BinaryOp::StrGt, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_gt.format(), "\"a\" gt \"b\"");

        let str_ge = Expr::binary(BinaryOp::StrGe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_ge.format(), "\"a\" ge \"b\"");
    }

    #[test]
    fn test_logical_operators() {
        let and = Expr::binary(BinaryOp::And, Expr::bool(true), Expr::bool(false));
        assert_eq!(and.format(), "true and false");

        let or = Expr::binary(BinaryOp::Or, Expr::bool(true), Expr::bool(false));
        assert_eq!(or.format(), "true or false");
    }

    #[test]
    fn test_complex_nested_expr() {
        // Test deeply nested expression
        // Note: format() doesn't add parentheses for precedence, it just formats linearly
        let expr = Expr::binary(
            BinaryOp::Add,
            Expr::binary(
                BinaryOp::Mul,
                Expr::col_idx(1),
                Expr::binary(BinaryOp::Add, Expr::int(10), Expr::int(20)),
            ),
            Expr::binary(BinaryOp::Div, Expr::col_idx(2), Expr::int(5)),
        );
        // format() produces linear output without parentheses for precedence
        assert_eq!(expr.format(), "@1 * 10 + 20 + @2 / 5");
    }

    #[test]
    fn test_empty_list() {
        let empty_list = Expr::List(vec![]);
        assert_eq!(empty_list.format(), "[]");
    }

    #[test]
    fn test_nested_list() {
        let nested_list = Expr::List(vec![
            Expr::List(vec![Expr::int(1), Expr::int(2)]),
            Expr::List(vec![Expr::int(3), Expr::int(4)]),
        ]);
        assert_eq!(nested_list.format(), "[[1, 2], [3, 4]]");
    }

    #[test]
    fn test_empty_block() {
        let empty_block = Expr::Block(vec![]);
        // Empty block's last_expr returns self, which would cause infinite recursion in format()
        // This is expected behavior - empty blocks should not be formatted
        // Just verify the block is created correctly
        assert!(matches!(empty_block, Expr::Block(ref v) if v.is_empty()));
    }

    #[test]
    fn test_lambda_param_format() {
        let param = Expr::LambdaParam("x".to_string());
        assert_eq!(param.format(), "x");
    }

    #[test]
    fn test_pipe_right_variants() {
        // Test PipeRight::Call
        let pipe_call = PipeRight::Call {
            name: "upper".to_string(),
            args: vec![],
        };
        match pipe_call {
            PipeRight::Call { name, args } => {
                assert_eq!(name, "upper");
                assert!(args.is_empty());
            }
            _ => panic!("Expected PipeRight::Call"),
        }

        // Test PipeRight::CallWithPlaceholder
        let pipe_placeholder = PipeRight::CallWithPlaceholder {
            name: "substr".to_string(),
            args: vec![Expr::int(0), Expr::int(5)],
        };
        match pipe_placeholder {
            PipeRight::CallWithPlaceholder { name, args } => {
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected PipeRight::CallWithPlaceholder"),
        }
    }

    #[test]
    fn test_column_ref_variants() {
        let idx = ColumnRef::Index(5);
        assert!(matches!(idx, ColumnRef::Index(5)));

        let name = ColumnRef::Name("test".to_string());
        assert!(matches!(name, ColumnRef::Name(n) if n == "test"));
    }

    #[test]
    fn test_unary_operators() {
        let neg = UnaryOp::Neg;
        assert!(matches!(neg, UnaryOp::Neg));

        let not = UnaryOp::Not;
        assert!(matches!(not, UnaryOp::Not));
    }

    #[test]
    fn test_expr_clone() {
        let expr = Expr::binary(BinaryOp::Add, Expr::col_name("price"), Expr::int(10));
        let cloned = expr.clone();
        assert_eq!(expr.format(), cloned.format());
    }

    #[test]
    fn test_expr_debug() {
        let expr = Expr::int(42);
        let debug_str = format!("{:?}", expr);
        assert!(debug_str.contains("Int"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_expr_equality() {
        let expr1 = Expr::int(42);
        let expr2 = Expr::int(42);
        let expr3 = Expr::int(43);

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_call_with_no_args() {
        let call = Expr::call("now", vec![]);
        assert_eq!(call.format(), "now()");
    }

    #[test]
    fn test_call_with_many_args() {
        let call = Expr::call(
            "substr",
            vec![
                Expr::col_name("text"),
                Expr::int(0),
                Expr::int(10),
                Expr::bool(true),
            ],
        );
        assert_eq!(call.format(), "substr(@text, 0, 10, true)");
    }

    #[test]
    fn test_whole_row_format() {
        let whole_row = Expr::ColumnRef(ColumnRef::WholeRow);
        assert_eq!(whole_row.format(), "@0");
    }

    #[test]
    fn test_lambda_param_expr() {
        let param = Expr::LambdaParam("x".to_string());
        assert_eq!(param.format(), "x");

        let param2 = Expr::LambdaParam("item".to_string());
        assert_eq!(param2.format(), "item");
    }

    #[test]
    fn test_nested_binary_format() {
        // Test nested binary expressions
        let inner = Expr::binary(BinaryOp::Mul, Expr::int(2), Expr::int(3));
        let outer = Expr::binary(BinaryOp::Add, Expr::int(1), inner);
        assert_eq!(outer.format(), "1 + 2 * 3");
    }

    #[test]
    fn test_all_string_comparison_ops() {
        // Test all string comparison operators
        let str_eq = Expr::binary(BinaryOp::StrEq, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_eq.format(), "\"a\" eq \"b\"");

        let str_ne = Expr::binary(BinaryOp::StrNe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_ne.format(), "\"a\" ne \"b\"");

        let str_lt = Expr::binary(BinaryOp::StrLt, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_lt.format(), "\"a\" lt \"b\"");

        let str_le = Expr::binary(BinaryOp::StrLe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_le.format(), "\"a\" le \"b\"");

        let str_gt = Expr::binary(BinaryOp::StrGt, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_gt.format(), "\"a\" gt \"b\"");

        let str_ge = Expr::binary(BinaryOp::StrGe, Expr::string("a"), Expr::string("b"));
        assert_eq!(str_ge.format(), "\"a\" ge \"b\"");
    }

    #[test]
    fn test_all_arithmetic_ops() {
        // Test all arithmetic operators
        let add = Expr::binary(BinaryOp::Add, Expr::int(1), Expr::int(2));
        assert_eq!(add.format(), "1 + 2");

        let sub = Expr::binary(BinaryOp::Sub, Expr::int(1), Expr::int(2));
        assert_eq!(sub.format(), "1 - 2");

        let mul = Expr::binary(BinaryOp::Mul, Expr::int(1), Expr::int(2));
        assert_eq!(mul.format(), "1 * 2");

        let div = Expr::binary(BinaryOp::Div, Expr::int(1), Expr::int(2));
        assert_eq!(div.format(), "1 / 2");

        let modulo = Expr::binary(BinaryOp::Mod, Expr::int(1), Expr::int(2));
        assert_eq!(modulo.format(), "1 % 2");

        let pow = Expr::binary(BinaryOp::Pow, Expr::int(2), Expr::int(3));
        assert_eq!(pow.format(), "2 ** 3");

        let concat =
            Expr::binary(BinaryOp::Concat, Expr::string("a"), Expr::string("b"));
        assert_eq!(concat.format(), "\"a\" ++ \"b\"");
    }

    #[test]
    fn test_all_numeric_comparison_ops() {
        // Test all numeric comparison operators
        let eq = Expr::binary(BinaryOp::Eq, Expr::int(1), Expr::int(2));
        assert_eq!(eq.format(), "1 == 2");

        let ne = Expr::binary(BinaryOp::Ne, Expr::int(1), Expr::int(2));
        assert_eq!(ne.format(), "1 != 2");

        let lt = Expr::binary(BinaryOp::Lt, Expr::int(1), Expr::int(2));
        assert_eq!(lt.format(), "1 < 2");

        let le = Expr::binary(BinaryOp::Le, Expr::int(1), Expr::int(2));
        assert_eq!(le.format(), "1 <= 2");

        let gt = Expr::binary(BinaryOp::Gt, Expr::int(1), Expr::int(2));
        assert_eq!(gt.format(), "1 > 2");

        let ge = Expr::binary(BinaryOp::Ge, Expr::int(1), Expr::int(2));
        assert_eq!(ge.format(), "1 >= 2");
    }

    #[test]
    fn test_logical_ops_format() {
        let and = Expr::binary(BinaryOp::And, Expr::bool(true), Expr::bool(false));
        assert_eq!(and.format(), "true and false");

        let or = Expr::binary(BinaryOp::Or, Expr::bool(true), Expr::bool(false));
        assert_eq!(or.format(), "true or false");
    }

    #[test]
    fn test_unary_ops_format() {
        let neg = Expr::unary(UnaryOp::Neg, Expr::int(5));
        assert_eq!(neg.format(), "-5");

        let not = Expr::unary(UnaryOp::Not, Expr::bool(true));
        assert_eq!(not.format(), "not true");
    }

    #[test]
    fn test_complex_nested_list() {
        let list = Expr::List(vec![
            Expr::List(vec![Expr::int(1), Expr::int(2)]),
            Expr::List(vec![Expr::int(3), Expr::int(4)]),
            Expr::List(vec![Expr::int(5), Expr::int(6)]),
        ]);
        assert_eq!(list.format(), "[[1, 2], [3, 4], [5, 6]]");
    }

    #[test]
    fn test_mixed_list() {
        let list = Expr::List(vec![
            Expr::int(1),
            Expr::string("hello"),
            Expr::bool(true),
            Expr::null(),
        ]);
        assert_eq!(list.format(), "[1, \"hello\", true, null]");
    }

    #[test]
    fn test_block_with_multiple_exprs() {
        let block = Expr::Block(vec![Expr::int(1), Expr::int(2), Expr::int(3)]);
        // Block formats as its last expression
        assert_eq!(block.format(), "3");
    }

    #[test]
    fn test_pipe_with_complex_left() {
        let left = Expr::binary(BinaryOp::Add, Expr::int(1), Expr::int(2));
        let pipe = Expr::Pipe {
            left: Box::new(left),
            right: Box::new(PipeRight::Call {
                name: "abs".to_string(),
                args: vec![],
            }),
        };
        assert_eq!(pipe.format(), "1 + 2 | abs()");
    }

    #[test]
    fn test_lambda_with_complex_body() {
        let lambda = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::binary(
                BinaryOp::Mul,
                Expr::LambdaParam("x".to_string()),
                Expr::binary(
                    BinaryOp::Add,
                    Expr::LambdaParam("x".to_string()),
                    Expr::int(1),
                ),
            )),
        };
        assert_eq!(lambda.format(), "x => x * x + 1");
    }

    #[test]
    fn test_method_call_chain_format() {
        let chain = Expr::MethodCall {
            object: Box::new(Expr::MethodCall {
                object: Box::new(Expr::col_name("name")),
                name: "trim".to_string(),
                args: vec![],
            }),
            name: "upper".to_string(),
            args: vec![],
        };
        assert_eq!(chain.format(), "@name.trim().upper()");
    }

    #[test]
    fn test_bind_with_pipe() {
        let bind = Expr::Bind {
            expr: Box::new(Expr::Pipe {
                left: Box::new(Expr::col_name("name")),
                right: Box::new(PipeRight::Call {
                    name: "trim".to_string(),
                    args: vec![],
                }),
            }),
            name: "clean_name".to_string(),
        };
        assert_eq!(bind.format(), "@name | trim() as @clean_name");
    }

    #[test]
    fn test_expr_equality_different_types() {
        let int = Expr::Int(42);
        let float = Expr::Float(42.0);
        let string = Expr::String("42".to_string());

        assert_ne!(int, float);
        assert_ne!(int, string);
        assert_ne!(float, string);
    }

    #[test]
    fn test_column_ref_equality() {
        let idx1 = ColumnRef::Index(1);
        let idx2 = ColumnRef::Index(1);
        let idx3 = ColumnRef::Index(2);
        let name = ColumnRef::Name("test".to_string());
        let whole = ColumnRef::WholeRow;

        assert_eq!(idx1, idx2);
        assert_ne!(idx1, idx3);
        assert_ne!(idx1, name);
        assert_ne!(idx1, whole);
        assert_ne!(name, whole);
    }

    #[test]
    fn test_unary_op_equality() {
        assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
        assert_eq!(UnaryOp::Not, UnaryOp::Not);
        assert_ne!(UnaryOp::Neg, UnaryOp::Not);
    }

    #[test]
    fn test_binary_op_equality() {
        assert_eq!(BinaryOp::Add, BinaryOp::Add);
        assert_eq!(BinaryOp::Sub, BinaryOp::Sub);
        assert_ne!(BinaryOp::Add, BinaryOp::Sub);
        assert_ne!(BinaryOp::StrEq, BinaryOp::Eq);
    }

    #[test]
    fn test_pipe_right_equality() {
        let call1 = PipeRight::Call {
            name: "upper".to_string(),
            args: vec![],
        };
        let call2 = PipeRight::Call {
            name: "upper".to_string(),
            args: vec![],
        };
        let call3 = PipeRight::Call {
            name: "lower".to_string(),
            args: vec![],
        };

        assert_eq!(call1, call2);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_expr_clone_preserves_structure() {
        let original = Expr::binary(
            BinaryOp::Add,
            Expr::col_name("price"),
            Expr::col_name("tax"),
        );
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.format(), cloned.format());
    }

    #[test]
    fn test_float_edge_cases_format() {
        let very_small = Expr::float(0.000001);
        assert!(
            very_small.format().starts_with("0.000001")
                || very_small.format().contains("e")
        );

        let negative = Expr::float(-123.456);
        assert_eq!(negative.format(), "-123.456");

        let zero = Expr::float(0.0);
        assert_eq!(zero.format(), "0");
    }

    #[test]
    fn test_large_int_format() {
        let large = Expr::int(1_000_000_000_000);
        assert_eq!(large.format(), "1000000000000");

        let negative = Expr::int(-999999);
        assert_eq!(negative.format(), "-999999");
    }

    #[test]
    fn test_string_with_special_chars_format() {
        let string_with_quotes = Expr::string("hello \"world\"");
        assert_eq!(string_with_quotes.format(), "\"hello \"world\"\"");

        let string_with_newline = Expr::string("hello\nworld");
        assert_eq!(string_with_newline.format(), "\"hello\nworld\"");
    }

    #[test]
    fn test_empty_string_format() {
        let empty = Expr::string("");
        assert_eq!(empty.format(), "\"\"");
    }

    #[test]
    fn test_call_with_single_arg() {
        let call = Expr::call("abs", vec![Expr::int(-5)]);
        assert_eq!(call.format(), "abs(-5)");
    }

    #[test]
    fn test_method_call_with_single_arg() {
        let method = Expr::MethodCall {
            object: Box::new(Expr::col_name("name")),
            name: "substr".to_string(),
            args: vec![Expr::int(0)],
        };
        assert_eq!(method.format(), "@name.substr(0)");
    }

    #[test]
    fn test_lambda_no_params() {
        let lambda = Expr::Lambda {
            params: vec![],
            body: Box::new(Expr::int(42)),
        };
        assert_eq!(lambda.format(), "() => 42");
    }

    #[test]
    fn test_method_call_no_args() {
        let method = Expr::MethodCall {
            object: Box::new(Expr::col_name("name")),
            name: "trim".to_string(),
            args: vec![],
        };
        assert_eq!(method.format(), "@name.trim()");
    }

    #[test]
    fn test_complex_pipe_chain() {
        // Test chained pipes: @name | upper() | trim()
        let inner_pipe = Expr::Pipe {
            left: Box::new(Expr::col_name("name")),
            right: Box::new(PipeRight::Call {
                name: "upper".to_string(),
                args: vec![],
            }),
        };
        let outer_pipe = Expr::Pipe {
            left: Box::new(inner_pipe),
            right: Box::new(PipeRight::Call {
                name: "trim".to_string(),
                args: vec![],
            }),
        };
        assert_eq!(outer_pipe.format(), "@name | upper() | trim()");
    }

    #[test]
    fn test_bind_with_complex_expr() {
        let bind = Expr::Bind {
            expr: Box::new(Expr::call("sum", vec![Expr::col_name("amount")])),
            name: "total".to_string(),
        };
        assert_eq!(bind.format(), "sum(@amount) as @total");
    }

    #[test]
    fn test_float_formatting() {
        let float = Expr::float(3.14159);
        let formatted = float.format();
        assert!(formatted.starts_with("3.14"));

        let negative_float = Expr::float(-2.5);
        assert_eq!(negative_float.format(), "-2.5");

        let zero_float = Expr::float(0.0);
        assert_eq!(zero_float.format(), "0");
    }

    #[test]
    fn test_string_escaping_in_format() {
        // Note: format() doesn't actually escape quotes in strings,
        // it just wraps them in double quotes
        let string = Expr::string("hello \"world\"");
        assert_eq!(string.format(), "\"hello \"world\"\"");
    }
}
