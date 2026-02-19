//! pg2sqlc-core: PostgreSQL 16 DDL to SQLite3 DDL converter library.

pub mod diagnostics;
pub mod ir;
pub mod pg;
pub mod sqlite;
pub mod transform;

use std::path::PathBuf;

use diagnostics::warning::Warning;
use diagnostics::{StrictViolation, check_strict};

/// Options for the DDL conversion.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Schema to filter by (default: "public").
    pub schema: Option<String>,
    /// If true, include all schemas (bypass schema filtering).
    pub include_all_schemas: bool,
    /// If true, emit `PRAGMA foreign_keys = ON;` and include FK constraints.
    pub enable_foreign_keys: bool,
    /// If true, fail on lossy conversions.
    pub strict: bool,
    /// Path for warning output (None = stderr).
    pub emit_warnings: Option<PathBuf>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            schema: Some("public".to_string()),
            include_all_schemas: false,
            enable_foreign_keys: false,
            strict: false,
            emit_warnings: None,
        }
    }
}

/// Result of a successful conversion.
#[derive(Debug)]
pub struct ConvertResult {
    /// The generated SQLite DDL text.
    pub sqlite_sql: String,
    /// Warnings emitted during conversion.
    pub warnings: Vec<Warning>,
}

/// Errors that can occur during conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("Strict mode violation:\n{0}")]
    StrictViolation(#[from] StrictViolation),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convert PostgreSQL DDL text to SQLite DDL.
///
/// This is the primary public API for the library.
pub fn convert_pg_ddl_to_sqlite(
    input: &str,
    opts: &ConvertOptions,
) -> Result<ConvertResult, ConvertError> {
    let mut warnings = Vec::new();

    // 1. Parse PG DDL → IR
    let (mut model, parse_warnings) = pg::parser::parse(input);
    warnings.extend(parse_warnings);

    // 2. Normalize (schema filtering)
    let normalize_opts = pg::normalize::NormalizeOptions {
        schema: opts.schema.clone(),
        include_all_schemas: opts.include_all_schemas,
    };
    pg::normalize::normalize(&mut model, &normalize_opts);

    // 3. Plan (merge ALTERs, resolve SERIAL/sequences)
    transform::planner::plan(&mut model, &mut warnings);

    // 4. Transform types
    for table in &mut model.tables {
        for col in &mut table.columns {
            let obj = format!("{}.{}", table.name.name.normalized, col.name.normalized);
            col.sqlite_type = Some(transform::type_map::map_type(
                &col.pg_type,
                &obj,
                &mut warnings,
            ));

            // Transform default expressions
            if let Some(default) = &col.default {
                col.default = transform::expr_map::map_expr(default, &obj, &mut warnings);
            }
        }
    }

    // 5. Transform constraints
    transform::constraint::transform_constraints(
        &mut model,
        opts.enable_foreign_keys,
        &mut warnings,
    );

    // 6. Transform indexes
    transform::index::transform_indexes(&mut model, &mut warnings);

    // 7. Resolve names (schema stripping, collision handling)
    transform::name_resolve::resolve_names(&mut model, opts.include_all_schemas, &mut warnings);

    // 8. Topological sort (if FK enabled)
    if opts.enable_foreign_keys {
        transform::topo::topological_sort(&mut model.tables);
    } else {
        // Alphabetical order when FKs disabled
        model
            .tables
            .sort_by(|a, b| a.name.name.normalized.cmp(&b.name.name.normalized));
    }

    // 9. Render SQLite DDL
    let sqlite_sql = sqlite::render::render(&model, opts.enable_foreign_keys);

    // 10. Check strict mode
    if opts.strict {
        check_strict(&warnings)?;
    }

    Ok(ConvertResult {
        sqlite_sql,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let input = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);";
        let result = convert_pg_ddl_to_sqlite(input, &ConvertOptions::default()).unwrap();
        assert!(result.sqlite_sql.contains("CREATE TABLE users"));
        assert!(result.sqlite_sql.contains("id INTEGER PRIMARY KEY"));
        assert!(result.sqlite_sql.contains("name TEXT NOT NULL"));
    }

    #[test]
    fn test_schema_filtering() {
        let input = r#"
            CREATE TABLE public.users (id INTEGER);
            CREATE TABLE other.accounts (id INTEGER);
        "#;
        let result = convert_pg_ddl_to_sqlite(input, &ConvertOptions::default()).unwrap();
        assert!(result.sqlite_sql.contains("users"));
        assert!(!result.sqlite_sql.contains("accounts"));
    }

    #[test]
    fn test_fk_with_pragma() {
        let input = r#"
            CREATE TABLE users (id INTEGER PRIMARY KEY);
            CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));
        "#;
        let opts = ConvertOptions {
            enable_foreign_keys: true,
            ..Default::default()
        };
        let result = convert_pg_ddl_to_sqlite(input, &opts).unwrap();
        assert!(result.sqlite_sql.contains("PRAGMA foreign_keys = ON;"));
        assert!(result.sqlite_sql.contains("REFERENCES users(id)"));
    }

    #[test]
    fn test_fk_ordering() {
        let input = r#"
            CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));
            CREATE TABLE users (id INTEGER PRIMARY KEY);
        "#;
        let opts = ConvertOptions {
            enable_foreign_keys: true,
            ..Default::default()
        };
        let result = convert_pg_ddl_to_sqlite(input, &opts).unwrap();
        let users_pos = result.sqlite_sql.find("CREATE TABLE users").unwrap();
        let orders_pos = result.sqlite_sql.find("CREATE TABLE orders").unwrap();
        assert!(
            users_pos < orders_pos,
            "users should come before orders due to FK dependency"
        );
    }

    #[test]
    fn test_strict_mode_fails_on_lossy() {
        let input = "CREATE TABLE t (active BOOLEAN DEFAULT true);";
        let opts = ConvertOptions {
            strict: true,
            ..Default::default()
        };
        let result = convert_pg_ddl_to_sqlite(input, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_boolean_conversion() {
        let input = "CREATE TABLE t (active BOOLEAN DEFAULT true);";
        let result = convert_pg_ddl_to_sqlite(input, &ConvertOptions::default()).unwrap();
        assert!(result.sqlite_sql.contains("INTEGER"));
        assert!(result.sqlite_sql.contains("DEFAULT 1"));
    }

    #[test]
    fn test_timestamp_default_now() {
        let input = "CREATE TABLE t (created_at TIMESTAMP DEFAULT now());";
        let result = convert_pg_ddl_to_sqlite(input, &ConvertOptions::default()).unwrap();
        assert!(result.sqlite_sql.contains("TEXT"));
        assert!(result.sqlite_sql.contains("CURRENT_TIMESTAMP"));
    }

    #[test]
    fn test_varchar_length_dropped() {
        let input = "CREATE TABLE t (name VARCHAR(255) NOT NULL);";
        let result = convert_pg_ddl_to_sqlite(input, &ConvertOptions::default()).unwrap();
        assert!(result.sqlite_sql.contains("TEXT NOT NULL"));
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.code == "VARCHAR_LENGTH_IGNORED")
        );
    }

    #[test]
    fn test_alter_table_constraint_merged() {
        let input = r#"
            CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER);
            ALTER TABLE orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
        "#;
        let opts = ConvertOptions {
            enable_foreign_keys: true,
            ..Default::default()
        };
        let result = convert_pg_ddl_to_sqlite(input, &opts).unwrap();
        assert!(
            result
                .sqlite_sql
                .contains("FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE")
        );
    }

    #[test]
    fn test_include_all_schemas() {
        let input = r#"
            CREATE TABLE public.users (id INTEGER);
            CREATE TABLE other.accounts (id INTEGER);
        "#;
        let opts = ConvertOptions {
            include_all_schemas: true,
            ..Default::default()
        };
        let result = convert_pg_ddl_to_sqlite(input, &opts).unwrap();
        assert!(result.sqlite_sql.contains("users"));
        assert!(result.sqlite_sql.contains("accounts"));
    }
}
