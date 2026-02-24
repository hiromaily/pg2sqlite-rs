/// Constraint transformation: PK, UNIQUE, FK, CHECK.
use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::{PgType, SchemaModel, SqliteType, Table, TableConstraint};
use crate::transform::expr_map;

/// Transform constraints on all tables in the schema model.
pub fn transform_constraints(
    model: &mut SchemaModel,
    enable_foreign_keys: bool,
    warnings: &mut Vec<Warning>,
) {
    for table in &mut model.tables {
        transform_table_constraints(table, enable_foreign_keys, warnings);
    }
}

fn transform_table_constraints(
    table: &mut Table,
    enable_foreign_keys: bool,
    warnings: &mut Vec<Warning>,
) {
    let table_name = table.name.name.normalized.clone();

    // Handle single-column integer PK: promote to column-level INTEGER PRIMARY KEY
    handle_integer_pk(table);

    // Transform CHECK constraint expressions
    let mut kept_constraints = Vec::new();
    for constraint in &table.constraints {
        match constraint {
            TableConstraint::PrimaryKey { .. } | TableConstraint::Unique { .. } => {
                kept_constraints.push(constraint.clone());
            }
            TableConstraint::ForeignKey { deferrable, .. } => {
                if !enable_foreign_keys {
                    continue;
                }
                if *deferrable {
                    warnings.push(
                        Warning::new(
                            warning::DEFERRABLE_IGNORED,
                            Severity::Lossy,
                            "DEFERRABLE modifier dropped from foreign key",
                        )
                        .with_object(&table_name),
                    );
                }
                // Clone without deferrable
                let mut c = constraint.clone();
                if let TableConstraint::ForeignKey { deferrable, .. } = &mut c {
                    *deferrable = false;
                }
                kept_constraints.push(c);
            }
            TableConstraint::Check { name, expr } => {
                let obj = format!("{table_name}.CHECK");
                match expr_map::map_expr(expr, &obj, warnings) {
                    Some(mapped) => {
                        kept_constraints.push(TableConstraint::Check {
                            name: name.clone(),
                            expr: mapped,
                        });
                    }
                    None => {
                        warnings.push(
                            Warning::new(
                                warning::CHECK_EXPRESSION_UNSUPPORTED,
                                Severity::Unsupported,
                                "CHECK constraint expression uses unsupported PG features; dropped",
                            )
                            .with_object(&table_name),
                        );
                    }
                }
            }
        }
    }
    table.constraints = kept_constraints;

    // Transform column-level CHECK constraints
    for col in &mut table.columns {
        if let Some(check) = &col.check {
            let obj = format!("{}.{}", table_name, col.name.normalized);
            match expr_map::map_expr(check, &obj, warnings) {
                Some(mapped) => col.check = Some(mapped),
                None => col.check = None,
            }
        }

        // Drop column-level FK refs if foreign keys disabled
        if !enable_foreign_keys {
            col.references = None;
        }
    }
}

/// If a table has a single-column integer PK as a table-level constraint,
/// and the column is an integer type, promote it to column-level.
fn handle_integer_pk(table: &mut Table) {
    let pk_constraint = table.constraints.iter().position(
        |c| matches!(c, TableConstraint::PrimaryKey { columns, .. } if columns.len() == 1),
    );

    if let Some(pk_idx) = pk_constraint
        && let TableConstraint::PrimaryKey { columns, .. } = &table.constraints[pk_idx]
    {
        let pk_col_name = columns[0].normalized.clone();

        // Check if the column is an integer type
        let col = table
            .columns
            .iter_mut()
            .find(|c| c.name.normalized == pk_col_name);
        if let Some(col) = col {
            // Skip if already resolved as autoincrement by identity resolution
            if col.autoincrement {
                return;
            }

            let is_integer = matches!(col.sqlite_type, Some(SqliteType::Integer))
                || matches!(
                    col.pg_type,
                    PgType::Integer
                        | PgType::BigInt
                        | PgType::SmallInt
                        | PgType::Serial
                        | PgType::BigSerial
                        | PgType::SmallSerial
                );

            if is_integer {
                col.is_primary_key = true;
                table.constraints.remove(pk_idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Column, FkAction, Ident, QualifiedName};

    fn make_column(name: &str, pg_type: PgType) -> Column {
        Column {
            name: Ident::new(name),
            pg_type,
            sqlite_type: None,
            not_null: false,
            default: None,
            is_primary_key: false,
            is_unique: false,
            autoincrement: false,
            references: None,
            check: None,
        }
    }

    fn make_table(name: &str, columns: Vec<Column>, constraints: Vec<TableConstraint>) -> Table {
        Table {
            name: QualifiedName::new(Ident::new(name)),
            columns,
            constraints,
        }
    }

    #[test]
    fn test_fk_dropped_when_disabled() {
        let mut model = SchemaModel {
            tables: vec![make_table(
                "orders",
                vec![make_column("id", PgType::Integer)],
                vec![TableConstraint::ForeignKey {
                    name: None,
                    columns: vec![Ident::new("user_id")],
                    ref_table: QualifiedName::new(Ident::new("users")),
                    ref_columns: vec![Ident::new("id")],
                    on_delete: Some(FkAction::Cascade),
                    on_update: None,
                    deferrable: false,
                }],
            )],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_constraints(&mut model, false, &mut w);
        assert!(model.tables[0].constraints.is_empty());
    }

    #[test]
    fn test_fk_kept_when_enabled() {
        let mut model = SchemaModel {
            tables: vec![make_table(
                "orders",
                vec![make_column("id", PgType::Integer)],
                vec![TableConstraint::ForeignKey {
                    name: None,
                    columns: vec![Ident::new("user_id")],
                    ref_table: QualifiedName::new(Ident::new("users")),
                    ref_columns: vec![Ident::new("id")],
                    on_delete: Some(FkAction::Cascade),
                    on_update: None,
                    deferrable: false,
                }],
            )],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_constraints(&mut model, true, &mut w);
        assert_eq!(model.tables[0].constraints.len(), 1);
    }

    #[test]
    fn test_single_integer_pk_promoted() {
        let mut model = SchemaModel {
            tables: vec![make_table(
                "t",
                vec![make_column("id", PgType::Integer)],
                vec![TableConstraint::PrimaryKey {
                    name: None,
                    columns: vec![Ident::new("id")],
                }],
            )],
            ..Default::default()
        };
        let mut w = Vec::new();
        transform_constraints(&mut model, false, &mut w);
        assert!(model.tables[0].columns[0].is_primary_key);
        // Table-level PK should be removed
        assert!(model.tables[0].constraints.is_empty());
    }
}
