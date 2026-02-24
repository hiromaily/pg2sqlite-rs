/// Schema stripping and name collision handling.
use std::collections::HashMap;

use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::{Ident, QualifiedName, SchemaModel, TableConstraint};

/// Strip schema prefixes from all identifiers.
/// When include_all_schemas is true and names collide, prefix with schema__table.
pub fn resolve_names(
    model: &mut SchemaModel,
    include_all_schemas: bool,
    warnings: &mut Vec<Warning>,
) {
    if !include_all_schemas {
        // Just strip schema prefixes
        strip_schemas(model);
        return;
    }

    // Detect collisions
    let mut name_counts: HashMap<String, Vec<(Option<String>, usize)>> = HashMap::new();
    for (i, table) in model.tables.iter().enumerate() {
        let schema = table.name.schema.as_ref().map(|s| s.normalized.clone());
        name_counts
            .entry(table.name.name.normalized.clone())
            .or_default()
            .push((schema, i));
    }

    // Build rename map for collisions
    let mut rename_map: HashMap<(Option<String>, String), String> = HashMap::new();
    for (name, entries) in &name_counts {
        if entries.len() > 1 {
            for (schema, _) in entries {
                if let Some(s) = schema {
                    let new_name = format!("{s}__{name}");
                    rename_map.insert((Some(s.clone()), name.clone()), new_name.clone());
                    warnings.push(
                        Warning::new(
                            warning::SCHEMA_PREFIXED,
                            Severity::Lossy,
                            format!(
                                "table '{s}.{name}' renamed to '{new_name}' to avoid collision"
                            ),
                        )
                        .with_object(&new_name),
                    );
                }
            }
        }
    }

    // Apply renames
    for table in &mut model.tables {
        let key = (
            table.name.schema.as_ref().map(|s| s.normalized.clone()),
            table.name.name.normalized.clone(),
        );
        if let Some(new_name) = rename_map.get(&key) {
            table.name = QualifiedName::new(Ident::new(new_name));
        } else {
            table.name.schema = None;
        }
    }

    // Rename FK references in constraints
    for table in &mut model.tables {
        for constraint in &mut table.constraints {
            if let TableConstraint::ForeignKey { ref_table, .. } = constraint {
                let key = (
                    ref_table.schema.as_ref().map(|s| s.normalized.clone()),
                    ref_table.name.normalized.clone(),
                );
                if let Some(new_name) = rename_map.get(&key) {
                    *ref_table = QualifiedName::new(Ident::new(new_name));
                } else {
                    ref_table.schema = None;
                }
            }
        }

        // Also rename column-level FK refs
        for col in &mut table.columns {
            if let Some(fk) = &mut col.references {
                let key = (
                    fk.table.schema.as_ref().map(|s| s.normalized.clone()),
                    fk.table.name.normalized.clone(),
                );
                if let Some(new_name) = rename_map.get(&key) {
                    fk.table = QualifiedName::new(Ident::new(new_name));
                } else {
                    fk.table.schema = None;
                }
            }
        }
    }

    // Rename index table references
    for index in &mut model.indexes {
        let key = (
            index.table.schema.as_ref().map(|s| s.normalized.clone()),
            index.table.name.normalized.clone(),
        );
        if let Some(new_name) = rename_map.get(&key) {
            index.table = QualifiedName::new(Ident::new(new_name));
        } else {
            index.table.schema = None;
        }
    }
}

/// Simple schema stripping (no collision handling).
fn strip_schemas(model: &mut SchemaModel) {
    for table in &mut model.tables {
        table.name.schema = None;
    }
    for index in &mut model.indexes {
        index.table.schema = None;
    }
    for table in &mut model.tables {
        for constraint in &mut table.constraints {
            if let TableConstraint::ForeignKey { ref_table, .. } = constraint {
                ref_table.schema = None;
            }
        }
        for col in &mut table.columns {
            if let Some(fk) = &mut col.references {
                fk.table.schema = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Column, PgType, Table};

    fn make_table(schema: Option<&str>, name: &str) -> Table {
        Table {
            name: match schema {
                Some(s) => QualifiedName::with_schema(Ident::new(s), Ident::new(name)),
                None => QualifiedName::new(Ident::new(name)),
            },
            columns: vec![Column {
                name: Ident::new("id"),
                pg_type: PgType::Integer,
                sqlite_type: None,
                not_null: false,
                default: None,
                is_primary_key: false,
                is_unique: false,
                autoincrement: false,
                references: None,
                check: None,
            }],
            constraints: vec![],
        }
    }

    #[test]
    fn test_strip_schemas_single() {
        let mut model = SchemaModel {
            tables: vec![make_table(Some("public"), "users")],
            ..Default::default()
        };
        let mut w = Vec::new();
        resolve_names(&mut model, false, &mut w);
        assert!(model.tables[0].name.schema.is_none());
    }

    #[test]
    fn test_collision_prefixing() {
        let mut model = SchemaModel {
            tables: vec![
                make_table(Some("public"), "users"),
                make_table(Some("other"), "users"),
            ],
            ..Default::default()
        };
        let mut w = Vec::new();
        resolve_names(&mut model, true, &mut w);

        let names: Vec<&str> = model
            .tables
            .iter()
            .map(|t| t.name.name.normalized.as_str())
            .collect();
        assert!(names.contains(&"public__users"));
        assert!(names.contains(&"other__users"));
        assert!(w.iter().any(|w| w.code == warning::SCHEMA_PREFIXED));
    }
}
