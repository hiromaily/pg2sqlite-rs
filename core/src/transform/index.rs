/// Index conversion with method filtering and expression handling.
use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::{Index, IndexColumn, IndexMethod, SchemaModel};
use crate::transform::expr_map;

/// Transform indexes in the schema model.
pub fn transform_indexes(model: &mut SchemaModel, warnings: &mut Vec<Warning>) {
    let mut kept = Vec::new();

    for index in &model.indexes {
        if let Some(idx) = transform_index(index, warnings) {
            kept.push(idx);
        }
    }

    model.indexes = kept;
}

fn transform_index(index: &Index, warnings: &mut Vec<Warning>) -> Option<Index> {
    let obj = index.name.normalized.clone();

    // Warn about non-btree methods
    if let Some(method) = &index.method
        && *method != IndexMethod::Btree
    {
        warnings.push(
            Warning::new(
                warning::INDEX_METHOD_IGNORED,
                Severity::Info,
                format!("index method '{method}' ignored; SQLite only supports btree"),
            )
            .with_object(&obj),
        );
    }

    // Transform WHERE clause
    let where_clause = if let Some(where_expr) = &index.where_clause {
        match expr_map::map_expr(where_expr, &obj, warnings) {
            Some(mapped) => Some(mapped),
            None => {
                warnings.push(
                    Warning::new(
                        warning::PARTIAL_INDEX_UNSUPPORTED,
                        Severity::Unsupported,
                        "partial index WHERE clause uses unsupported PG features; index skipped",
                    )
                    .with_object(&obj),
                );
                return None;
            }
        }
    } else {
        None
    };

    // Transform expression columns
    let mut columns = Vec::new();
    for col in &index.columns {
        match col {
            IndexColumn::Column(ident) => {
                columns.push(IndexColumn::Column(ident.clone()));
            }
            IndexColumn::Expression(expr) => match expr_map::map_expr(expr, &obj, warnings) {
                Some(mapped) => {
                    columns.push(IndexColumn::Expression(mapped));
                }
                None => {
                    warnings.push(
                        Warning::new(
                            warning::EXPRESSION_INDEX_UNSUPPORTED,
                            Severity::Unsupported,
                            "expression index uses unsupported PG features; index skipped",
                        )
                        .with_object(&obj),
                    );
                    return None;
                }
            },
        }
    }

    Some(Index {
        name: index.name.clone(),
        table: index.table.clone(),
        columns,
        unique: index.unique,
        method: None, // Method always stripped for SQLite
        where_clause,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Expr, Ident, QualifiedName};

    fn make_index(name: &str, table: &str, cols: Vec<&str>) -> Index {
        Index {
            name: Ident::new(name),
            table: QualifiedName::new(Ident::new(table)),
            columns: cols
                .into_iter()
                .map(|c| IndexColumn::Column(Ident::new(c)))
                .collect(),
            unique: false,
            method: None,
            where_clause: None,
        }
    }

    #[test]
    fn test_simple_index_passthrough() {
        let mut model = SchemaModel {
            indexes: vec![make_index("idx_name", "users", vec!["name"])],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_indexes(&mut model, &mut w);
        assert_eq!(model.indexes.len(), 1);
        assert!(w.is_empty());
    }

    #[test]
    fn test_gin_method_warned() {
        let mut idx = make_index("idx_data", "t", vec!["data"]);
        idx.method = Some(IndexMethod::Gin);
        let mut model = SchemaModel {
            indexes: vec![idx],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_indexes(&mut model, &mut w);
        assert_eq!(model.indexes.len(), 1);
        assert!(w.iter().any(|w| w.code == warning::INDEX_METHOD_IGNORED));
        assert!(model.indexes[0].method.is_none());
    }

    #[test]
    fn test_partial_index_with_compatible_where() {
        let mut idx = make_index("idx_active", "users", vec!["email"]);
        idx.where_clause = Some(Expr::IsNull {
            expr: Box::new(Expr::ColumnRef("deleted_at".to_string())),
            negated: false,
        });
        let mut model = SchemaModel {
            indexes: vec![idx],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_indexes(&mut model, &mut w);
        assert_eq!(model.indexes.len(), 1);
        assert!(model.indexes[0].where_clause.is_some());
    }
}
