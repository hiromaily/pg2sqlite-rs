//! Expression types for default values, CHECK constraints, and WHERE clauses.

/// An expression node in the IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal (e.g., `42`)
    IntegerLiteral(i64),
    /// Float literal (e.g., `3.14`)
    FloatLiteral(f64),
    /// String literal (e.g., `'hello'`)
    StringLiteral(String),
    /// Boolean literal (`true`/`false`)
    BooleanLiteral(bool),
    /// NULL literal
    Null,
    /// Column reference (e.g., `status`)
    ColumnRef(String),
    /// Function call (e.g., `now()`, `lower(col)`)
    FunctionCall { name: String, args: Vec<Expr> },
    /// Type cast (e.g., `value::integer`, `CAST(value AS integer)`)
    Cast {
        expr: std::boxed::Box<Expr>,
        type_name: String,
    },
    /// Binary operation (e.g., `a + b`, `a AND b`)
    BinaryOp {
        left: std::boxed::Box<Expr>,
        op: String,
        right: std::boxed::Box<Expr>,
    },
    /// Unary operation (e.g., `NOT a`, `-x`)
    UnaryOp {
        op: String,
        expr: std::boxed::Box<Expr>,
    },
    /// IS NULL / IS NOT NULL
    IsNull {
        expr: std::boxed::Box<Expr>,
        negated: bool,
    },
    /// IN list (e.g., `col IN ('a', 'b', 'c')`)
    InList {
        expr: std::boxed::Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    /// BETWEEN (e.g., `col BETWEEN 1 AND 10`)
    Between {
        expr: std::boxed::Box<Expr>,
        low: std::boxed::Box<Expr>,
        high: std::boxed::Box<Expr>,
        negated: bool,
    },
    /// Parenthesized expression
    Nested(std::boxed::Box<Expr>),
    /// nextval('sequence_name') — PG-specific, removed during transform
    NextVal(String),
    /// CURRENT_TIMESTAMP (SQLite built-in)
    CurrentTimestamp,
    /// Raw SQL string for expressions that can't be decomposed further
    Raw(String),
}

impl Expr {
    /// Render this expression as a SQL string.
    pub fn to_sql(&self) -> String {
        match self {
            Expr::IntegerLiteral(n) => n.to_string(),
            Expr::FloatLiteral(n) => n.to_string(),
            Expr::StringLiteral(s) => format!("'{}'", s.replace('\'', "''")),
            Expr::BooleanLiteral(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            Expr::Null => "NULL".to_string(),
            Expr::ColumnRef(name) => name.clone(),
            Expr::FunctionCall { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_sql()).collect();
                format!("{name}({})", args_str.join(", "))
            }
            Expr::Cast { expr, type_name } => {
                format!("CAST({} AS {type_name})", expr.to_sql())
            }
            Expr::BinaryOp { left, op, right } => {
                format!("{} {op} {}", left.to_sql(), right.to_sql())
            }
            Expr::UnaryOp { op, expr } => {
                format!("{op} {}", expr.to_sql())
            }
            Expr::IsNull { expr, negated } => {
                if *negated {
                    format!("{} IS NOT NULL", expr.to_sql())
                } else {
                    format!("{} IS NULL", expr.to_sql())
                }
            }
            Expr::InList {
                expr,
                list,
                negated,
            } => {
                let items: Vec<String> = list.iter().map(|e| e.to_sql()).collect();
                let not = if *negated { "NOT " } else { "" };
                format!("{} {not}IN ({})", expr.to_sql(), items.join(", "))
            }
            Expr::Between {
                expr,
                low,
                high,
                negated,
            } => {
                let not = if *negated { "NOT " } else { "" };
                format!(
                    "{} {not}BETWEEN {} AND {}",
                    expr.to_sql(),
                    low.to_sql(),
                    high.to_sql()
                )
            }
            Expr::Nested(inner) => format!("({})", inner.to_sql()),
            Expr::NextVal(seq) => format!("nextval('{seq}')"),
            Expr::CurrentTimestamp => "CURRENT_TIMESTAMP".to_string(),
            Expr::Raw(sql) => sql.clone(),
        }
    }
}
