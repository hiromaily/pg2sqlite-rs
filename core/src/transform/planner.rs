/// Planner: merge ALTER TABLE constraints and resolve SERIAL/IDENTITY/sequences.
use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::{Expr, PgType, SchemaModel, TableConstraint};

/// Plan and merge ALTER TABLE constraints into CREATE TABLE, resolve SERIAL/sequences.
pub fn plan(model: &mut SchemaModel, warnings: &mut Vec<Warning>) {
    merge_alter_constraints(model, warnings);
    resolve_serials(model, warnings);
    resolve_enums(model, warnings);
}

/// Merge ALTER TABLE ADD CONSTRAINT statements into the corresponding CREATE TABLE.
fn merge_alter_constraints(model: &mut SchemaModel, warnings: &mut Vec<Warning>) {
    let alters = std::mem::take(&mut model.alter_constraints);

    for alter in alters {
        let target_table = model.tables.iter_mut().find(|t| t.name == alter.table);

        match target_table {
            Some(table) => {
                table.constraints.push(alter.constraint);
            }
            None => {
                warnings.push(
                    Warning::new(
                        warning::ALTER_TARGET_MISSING,
                        Severity::Unsupported,
                        format!(
                            "ALTER TABLE target '{}' not found; constraint skipped",
                            alter.table.name.normalized
                        ),
                    )
                    .with_object(&alter.table.name.normalized),
                );
            }
        }
    }
}

/// Resolve SERIAL/BIGSERIAL/SMALLSERIAL columns:
/// - If column is single-column integer PK → mark as INTEGER PRIMARY KEY (rowid alias)
/// - Otherwise → map type to INTEGER, drop the DEFAULT, warn
fn resolve_serials(model: &mut SchemaModel, warnings: &mut Vec<Warning>) {
    // Collect sequence names for reference
    let _sequence_names: Vec<String> = model
        .sequences
        .iter()
        .map(|s| s.name.name.normalized.clone())
        .collect();

    for table in &mut model.tables {
        // Find if there's a table-level PK
        let table_pk_columns: Vec<String> = table
            .constraints
            .iter()
            .filter_map(|c| match c {
                TableConstraint::PrimaryKey { columns, .. } => Some(
                    columns
                        .iter()
                        .map(|c| c.normalized.clone())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect();

        for col in &mut table.columns {
            let is_serial = matches!(
                col.pg_type,
                PgType::Serial | PgType::BigSerial | PgType::SmallSerial
            );

            // Also check for nextval default (SERIAL sugar)
            let has_nextval = matches!(&col.default, Some(Expr::NextVal(_)));

            if !is_serial && !has_nextval {
                continue;
            }

            let obj = format!("{}.{}", table.name.name.normalized, col.name.normalized);

            // Is this column the sole PK?
            let is_sole_pk = col.is_primary_key
                || (table_pk_columns.len() == 1 && table_pk_columns[0] == col.name.normalized);

            if is_sole_pk {
                col.pg_type = PgType::Integer;
                col.is_primary_key = true;
                col.default = None;

                // Remove nextval default if present
                warnings.push(
                    Warning::new(
                        warning::SERIAL_TO_ROWID,
                        Severity::Lossy,
                        "SERIAL column mapped to INTEGER PRIMARY KEY (rowid alias)",
                    )
                    .with_object(&obj),
                );
            } else {
                col.pg_type = PgType::Integer;
                col.default = None;

                warnings.push(
                    Warning::new(
                        warning::SERIAL_NOT_PRIMARY_KEY,
                        Severity::Lossy,
                        "SERIAL column is not the sole primary key; mapped to INTEGER without auto-increment",
                    )
                    .with_object(&obj),
                );
            }
        }
    }

    // Warn about standalone sequences
    for seq in &model.sequences {
        warnings.push(
            Warning::new(
                warning::SEQUENCE_IGNORED,
                Severity::Info,
                format!(
                    "sequence '{}' ignored (absorbed into SERIAL handling or unused)",
                    seq.name.name.normalized
                ),
            )
            .with_object(&seq.name.name.normalized),
        );
    }
}

/// Resolve enum columns: replace PgType::Other with PgType::Enum where a matching enum exists.
fn resolve_enums(model: &mut SchemaModel, _warnings: &mut [Warning]) {
    let enum_names: std::collections::HashSet<String> = model
        .enums
        .iter()
        .map(|e| e.name.name.normalized.clone())
        .collect();

    for table in &mut model.tables {
        for col in &mut table.columns {
            if let PgType::Other { name } = &col.pg_type
                && enum_names.contains(name)
            {
                col.pg_type = PgType::Enum { name: name.clone() };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AlterConstraint, Column, FkAction, Ident, QualifiedName, Table};

    fn make_table(name: &str, columns: Vec<Column>, constraints: Vec<TableConstraint>) -> Table {
        Table {
            name: QualifiedName::new(Ident::new(name)),
            columns,
            constraints,
        }
    }

    fn make_column(name: &str, pg_type: PgType) -> Column {
        Column {
            name: Ident::new(name),
            pg_type,
            sqlite_type: None,
            not_null: false,
            default: None,
            is_primary_key: false,
            is_unique: false,
            references: None,
            check: None,
        }
    }

    #[test]
    fn test_merge_alter_constraints() {
        let mut model = SchemaModel {
            tables: vec![make_table(
                "orders",
                vec![
                    make_column("id", PgType::Integer),
                    make_column("user_id", PgType::Integer),
                ],
                vec![],
            )],
            alter_constraints: vec![AlterConstraint {
                table: QualifiedName::new(Ident::new("orders")),
                constraint: TableConstraint::ForeignKey {
                    name: Some(Ident::new("fk_user")),
                    columns: vec![Ident::new("user_id")],
                    ref_table: QualifiedName::new(Ident::new("users")),
                    ref_columns: vec![Ident::new("id")],
                    on_delete: Some(FkAction::Cascade),
                    on_update: None,
                    deferrable: false,
                },
            }],
            ..Default::default()
        };
        let mut w = Vec::new();
        plan(&mut model, &mut w);
        assert_eq!(model.tables[0].constraints.len(), 1);
    }

    #[test]
    fn test_alter_target_missing() {
        let mut model = SchemaModel {
            tables: vec![],
            alter_constraints: vec![AlterConstraint {
                table: QualifiedName::new(Ident::new("nonexistent")),
                constraint: TableConstraint::Check {
                    name: None,
                    expr: Expr::Raw("true".to_string()),
                },
            }],
            ..Default::default()
        };
        let mut w = Vec::new();
        plan(&mut model, &mut w);
        assert!(w.iter().any(|w| w.code == warning::ALTER_TARGET_MISSING));
    }

    #[test]
    fn test_serial_sole_pk() {
        let mut col = make_column("id", PgType::Serial);
        col.is_primary_key = true;
        let mut model = SchemaModel {
            tables: vec![make_table("users", vec![col], vec![])],
            ..Default::default()
        };
        let mut w = Vec::new();
        plan(&mut model, &mut w);
        assert_eq!(model.tables[0].columns[0].pg_type, PgType::Integer);
        assert!(model.tables[0].columns[0].is_primary_key);
        assert!(w.iter().any(|w| w.code == warning::SERIAL_TO_ROWID));
    }

    #[test]
    fn test_serial_not_pk() {
        let col = make_column("counter", PgType::Serial);
        let mut model = SchemaModel {
            tables: vec![make_table("t", vec![col], vec![])],
            ..Default::default()
        };
        let mut w = Vec::new();
        plan(&mut model, &mut w);
        assert_eq!(model.tables[0].columns[0].pg_type, PgType::Integer);
        assert!(w.iter().any(|w| w.code == warning::SERIAL_NOT_PRIMARY_KEY));
    }
}
