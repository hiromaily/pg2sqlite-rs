/// Schema filtering and identifier normalization for parsed PG DDL.
use crate::ir::SchemaModel;

/// Options for schema normalization.
pub struct NormalizeOptions {
    /// Schema to include (default: "public").
    pub schema: Option<String>,
    /// If true, include all schemas (bypass schema filtering).
    pub include_all_schemas: bool,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            schema: Some("public".to_string()),
            include_all_schemas: false,
        }
    }
}

/// Filter and normalize the schema model based on options.
pub fn normalize(model: &mut SchemaModel, opts: &NormalizeOptions) {
    if opts.include_all_schemas {
        return;
    }

    let target_schema = opts.schema.as_deref().unwrap_or("public");

    // Filter tables by schema
    model.tables.retain(|t| {
        match &t.name.schema {
            Some(s) => s.normalized == target_schema,
            None => true, // Unqualified names are assumed to be in the target schema
        }
    });

    // Filter indexes by table schema
    model.indexes.retain(|idx| match &idx.table.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });

    // Filter sequences by schema
    model.sequences.retain(|seq| match &seq.name.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });

    // Filter enums by schema
    model.enums.retain(|e| match &e.name.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });

    // Filter domains by schema
    model.domains.retain(|d| match &d.name.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });

    // Filter alter constraints by table schema
    model.alter_constraints.retain(|ac| match &ac.table.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });

    // Filter identity columns by table schema
    model.identity_columns.retain(|ic| match &ic.table.schema {
        Some(s) => s.normalized == target_schema,
        None => true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg::parser;

    #[test]
    fn test_normalize_filters_schema() {
        let sql = r#"
            CREATE TABLE public.users (id INTEGER);
            CREATE TABLE other.accounts (id INTEGER);
        "#;
        let (mut model, _) = parser::parse(sql);
        assert_eq!(model.tables.len(), 2);

        normalize(&mut model, &NormalizeOptions::default());
        assert_eq!(model.tables.len(), 1);
        assert_eq!(model.tables[0].name.name.normalized, "users");
    }

    #[test]
    fn test_normalize_include_all_schemas() {
        let sql = r#"
            CREATE TABLE public.users (id INTEGER);
            CREATE TABLE other.accounts (id INTEGER);
        "#;
        let (mut model, _) = parser::parse(sql);
        normalize(
            &mut model,
            &NormalizeOptions {
                schema: None,
                include_all_schemas: true,
            },
        );
        assert_eq!(model.tables.len(), 2);
    }

    #[test]
    fn test_normalize_unqualified_passes() {
        let sql = "CREATE TABLE users (id INTEGER);";
        let (mut model, _) = parser::parse(sql);
        normalize(&mut model, &NormalizeOptions::default());
        assert_eq!(model.tables.len(), 1);
    }

    #[test]
    fn test_normalize_custom_schema() {
        let sql = r#"
            CREATE TABLE myschema.users (id INTEGER);
            CREATE TABLE public.accounts (id INTEGER);
        "#;
        let (mut model, _) = parser::parse(sql);
        normalize(
            &mut model,
            &NormalizeOptions {
                schema: Some("myschema".to_string()),
                include_all_schemas: false,
            },
        );
        assert_eq!(model.tables.len(), 1);
        assert_eq!(model.tables[0].name.name.normalized, "users");
    }
}
