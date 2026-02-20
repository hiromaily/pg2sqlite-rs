/// PostgreSQL expression → SQLite expression conversion.
use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::Expr;

/// Convert a PG expression to a SQLite-compatible expression.
/// Returns None if the expression should be dropped entirely.
pub fn map_expr(expr: &Expr, object: &str, warnings: &mut Vec<Warning>) -> Option<Expr> {
    match expr {
        // Literals pass through
        Expr::IntegerLiteral(_) | Expr::FloatLiteral(_) | Expr::StringLiteral(_) | Expr::Null => {
            Some(expr.clone())
        }

        // Boolean → 0/1
        Expr::BooleanLiteral(b) => Some(Expr::IntegerLiteral(if *b { 1 } else { 0 })),

        // Column references pass through
        Expr::ColumnRef(_) => Some(expr.clone()),

        // CURRENT_TIMESTAMP passes through
        Expr::CurrentTimestamp => Some(Expr::CurrentTimestamp),

        // nextval('seq') → removed
        Expr::NextVal(seq) => {
            warnings.push(
                Warning::new(
                    warning::NEXTVAL_REMOVED,
                    Severity::Lossy,
                    format!("nextval('{seq}') default removed"),
                )
                .with_object(object),
            );
            None
        }

        // Function calls
        Expr::FunctionCall { name, args } => map_function_call(name, args, object, warnings),

        // Cast → strip the cast, keep the inner expression
        Expr::Cast {
            expr: inner,
            type_name,
        } => {
            warnings.push(
                Warning::new(
                    warning::CAST_REMOVED,
                    Severity::Info,
                    format!("cast to {type_name} removed"),
                )
                .with_object(object),
            );
            map_expr(inner, object, warnings)
        }

        // Binary operations — recursively convert both sides
        Expr::BinaryOp { left, op, right } => {
            let left = map_expr(left, object, warnings)?;
            let right = map_expr(right, object, warnings)?;
            Some(Expr::BinaryOp {
                left: std::boxed::Box::new(left),
                op: op.clone(),
                right: std::boxed::Box::new(right),
            })
        }

        // Unary operations
        Expr::UnaryOp { op, expr: inner } => {
            let mapped = map_expr(inner, object, warnings)?;
            Some(Expr::UnaryOp {
                op: op.clone(),
                expr: std::boxed::Box::new(mapped),
            })
        }

        // IS NULL / IS NOT NULL
        Expr::IsNull {
            expr: inner,
            negated,
        } => {
            let mapped = map_expr(inner, object, warnings)?;
            Some(Expr::IsNull {
                expr: std::boxed::Box::new(mapped),
                negated: *negated,
            })
        }

        // IN list
        Expr::InList {
            expr: inner,
            list,
            negated,
        } => {
            let mapped_expr = map_expr(inner, object, warnings)?;
            let mapped_list: Vec<Expr> = list
                .iter()
                .filter_map(|e| map_expr(e, object, warnings))
                .collect();
            Some(Expr::InList {
                expr: std::boxed::Box::new(mapped_expr),
                list: mapped_list,
                negated: *negated,
            })
        }

        // BETWEEN
        Expr::Between {
            expr: inner,
            low,
            high,
            negated,
        } => {
            let mapped = map_expr(inner, object, warnings)?;
            let mapped_low = map_expr(low, object, warnings)?;
            let mapped_high = map_expr(high, object, warnings)?;
            Some(Expr::Between {
                expr: std::boxed::Box::new(mapped),
                low: std::boxed::Box::new(mapped_low),
                high: std::boxed::Box::new(mapped_high),
                negated: *negated,
            })
        }

        // Nested expressions
        Expr::Nested(inner) => {
            let mapped = map_expr(inner, object, warnings)?;
            Some(Expr::Nested(std::boxed::Box::new(mapped)))
        }

        // Raw SQL — pass through (best effort)
        Expr::Raw(_) => Some(expr.clone()),
    }
}

fn map_function_call(
    name: &str,
    args: &[Expr],
    object: &str,
    warnings: &mut Vec<Warning>,
) -> Option<Expr> {
    match name {
        // now() → CURRENT_TIMESTAMP
        "now" => Some(Expr::CurrentTimestamp),

        // lower(), upper(), length(), abs(), max(), min() — SQLite-compatible
        "lower" | "upper" | "length" | "abs" | "max" | "min" | "coalesce" | "nullif" | "typeof"
        | "trim" | "ltrim" | "rtrim" | "replace" | "substr" | "instr" | "hex" | "quote"
        | "round" | "random" | "unicode" | "zeroblob" | "total" | "sum" | "avg" | "count"
        | "group_concat" => {
            let mapped_args: Vec<Expr> = args
                .iter()
                .filter_map(|a| map_expr(a, object, warnings))
                .collect();
            Some(Expr::FunctionCall {
                name: name.to_string(),
                args: mapped_args,
            })
        }

        // PG-specific functions → drop with warning
        _ => {
            warnings.push(
                Warning::new(
                    warning::DEFAULT_UNSUPPORTED,
                    Severity::Unsupported,
                    format!("unsupported function '{name}()' in expression"),
                )
                .with_object(object),
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_passthrough() {
        let mut w = Vec::new();
        let expr = Expr::IntegerLiteral(42);
        assert_eq!(
            map_expr(&expr, "t.c", &mut w),
            Some(Expr::IntegerLiteral(42))
        );
        assert!(w.is_empty());
    }

    #[test]
    fn test_boolean_to_integer() {
        let mut w = Vec::new();
        assert_eq!(
            map_expr(&Expr::BooleanLiteral(true), "t.c", &mut w),
            Some(Expr::IntegerLiteral(1))
        );
        assert_eq!(
            map_expr(&Expr::BooleanLiteral(false), "t.c", &mut w),
            Some(Expr::IntegerLiteral(0))
        );
    }

    #[test]
    fn test_now_to_current_timestamp() {
        let mut w = Vec::new();
        let expr = Expr::FunctionCall {
            name: "now".to_string(),
            args: vec![],
        };
        assert_eq!(map_expr(&expr, "t.c", &mut w), Some(Expr::CurrentTimestamp));
    }

    #[test]
    fn test_nextval_removed() {
        let mut w = Vec::new();
        let expr = Expr::NextVal("users_id_seq".to_string());
        assert_eq!(map_expr(&expr, "t.id", &mut w), None);
        assert_eq!(w[0].code, warning::NEXTVAL_REMOVED);
    }

    #[test]
    fn test_cast_stripped() {
        let mut w = Vec::new();
        let expr = Expr::Cast {
            expr: std::boxed::Box::new(Expr::IntegerLiteral(42)),
            type_name: "integer".to_string(),
        };
        assert_eq!(
            map_expr(&expr, "t.c", &mut w),
            Some(Expr::IntegerLiteral(42))
        );
        assert_eq!(w[0].code, warning::CAST_REMOVED);
    }

    #[test]
    fn test_unsupported_function() {
        let mut w = Vec::new();
        let expr = Expr::FunctionCall {
            name: "gen_random_uuid".to_string(),
            args: vec![],
        };
        assert_eq!(map_expr(&expr, "t.c", &mut w), None);
        assert_eq!(w[0].code, warning::DEFAULT_UNSUPPORTED);
    }

    #[test]
    fn test_compatible_function_passthrough() {
        let mut w = Vec::new();
        let expr = Expr::FunctionCall {
            name: "lower".to_string(),
            args: vec![Expr::ColumnRef("name".to_string())],
        };
        let result = map_expr(&expr, "t.c", &mut w);
        assert!(result.is_some());
        assert!(w.is_empty());
    }
}
