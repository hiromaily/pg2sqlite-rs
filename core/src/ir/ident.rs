//! Identifier types for PostgreSQL and SQLite DDL objects.

/// SQLite reserved keywords that require quoting.
const SQLITE_RESERVED: &[&str] = &[
    "abort",
    "action",
    "add",
    "after",
    "all",
    "alter",
    "always",
    "analyze",
    "and",
    "as",
    "asc",
    "attach",
    "autoincrement",
    "before",
    "begin",
    "between",
    "by",
    "cascade",
    "case",
    "cast",
    "check",
    "collate",
    "column",
    "commit",
    "conflict",
    "constraint",
    "create",
    "cross",
    "current",
    "current_date",
    "current_time",
    "current_timestamp",
    "database",
    "default",
    "deferrable",
    "deferred",
    "delete",
    "desc",
    "detach",
    "distinct",
    "do",
    "drop",
    "each",
    "else",
    "end",
    "escape",
    "except",
    "exclude",
    "exclusive",
    "exists",
    "explain",
    "fail",
    "filter",
    "first",
    "following",
    "for",
    "foreign",
    "from",
    "full",
    "generated",
    "glob",
    "group",
    "groups",
    "having",
    "if",
    "ignore",
    "immediate",
    "in",
    "index",
    "indexed",
    "initially",
    "inner",
    "insert",
    "instead",
    "intersect",
    "into",
    "is",
    "isnull",
    "join",
    "key",
    "last",
    "left",
    "like",
    "limit",
    "match",
    "materialized",
    "natural",
    "no",
    "not",
    "nothing",
    "notnull",
    "null",
    "nulls",
    "of",
    "offset",
    "on",
    "or",
    "order",
    "others",
    "outer",
    "over",
    "partition",
    "plan",
    "pragma",
    "preceding",
    "primary",
    "query",
    "raise",
    "range",
    "recursive",
    "references",
    "regexp",
    "reindex",
    "release",
    "rename",
    "replace",
    "restrict",
    "returning",
    "right",
    "rollback",
    "row",
    "rows",
    "savepoint",
    "select",
    "set",
    "table",
    "temp",
    "temporary",
    "then",
    "ties",
    "to",
    "transaction",
    "trigger",
    "unbounded",
    "union",
    "unique",
    "update",
    "using",
    "vacuum",
    "values",
    "view",
    "virtual",
    "when",
    "where",
    "window",
    "with",
    "without",
];

/// An identifier with both raw (original) and normalized (lowercase) forms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    /// Original form as written in source DDL.
    pub raw: String,
    /// Normalized form (lowercased for unquoted identifiers).
    pub normalized: String,
}

impl Ident {
    /// Create an identifier from an unquoted name (normalizes to lowercase).
    pub fn new(name: &str) -> Self {
        Self {
            raw: name.to_string(),
            normalized: name.to_lowercase(),
        }
    }

    /// Create an identifier from a quoted name (preserves case).
    pub fn quoted(name: &str) -> Self {
        Self {
            raw: name.to_string(),
            normalized: name.to_string(),
        }
    }

    /// Check if this identifier needs quoting in SQLite output.
    pub fn needs_quotes(&self) -> bool {
        let n = &self.normalized;

        // Empty identifiers need quotes
        if n.is_empty() {
            return true;
        }

        // Starts with digit
        if n.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }

        // Contains uppercase, spaces, hyphens, or other special chars
        if n.chars()
            .any(|c| c.is_ascii_uppercase() || c == ' ' || c == '-')
        {
            return true;
        }

        // Contains non-alphanumeric/underscore chars
        if n.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
            return true;
        }

        // Is a SQLite reserved keyword
        if SQLITE_RESERVED.contains(&n.as_str()) {
            return true;
        }

        false
    }

    /// Render the identifier for SQLite output, quoting if necessary.
    pub fn to_sql(&self) -> String {
        if self.needs_quotes() {
            format!("\"{}\"", self.normalized.replace('"', "\"\""))
        } else {
            self.normalized.clone()
        }
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.normalized)
    }
}

/// A schema-qualified name (e.g., `public.users`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub schema: Option<Ident>,
    pub name: Ident,
}

impl QualifiedName {
    pub fn new(name: Ident) -> Self {
        Self { schema: None, name }
    }

    pub fn with_schema(schema: Ident, name: Ident) -> Self {
        Self {
            schema: Some(schema),
            name,
        }
    }

    /// Get the table name for SQLite output (no schema prefix).
    pub fn to_sql(&self) -> String {
        self.name.to_sql()
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(schema) = &self.schema {
            write!(f, "{}.{}", schema, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident_new_normalizes() {
        let id = Ident::new("MyTable");
        assert_eq!(id.normalized, "mytable");
        assert_eq!(id.raw, "MyTable");
    }

    #[test]
    fn test_ident_quoted_preserves() {
        let id = Ident::quoted("MyTable");
        assert_eq!(id.normalized, "MyTable");
    }

    #[test]
    fn test_needs_quotes_reserved() {
        assert!(Ident::new("select").needs_quotes());
        assert!(Ident::new("table").needs_quotes());
    }

    #[test]
    fn test_needs_quotes_special_chars() {
        assert!(Ident::quoted("My-Column").needs_quotes());
        assert!(Ident::quoted("Has Space").needs_quotes());
    }

    #[test]
    fn test_no_quotes_simple() {
        assert!(!Ident::new("users").needs_quotes());
        assert!(!Ident::new("user_id").needs_quotes());
    }

    #[test]
    fn test_to_sql() {
        assert_eq!(Ident::new("users").to_sql(), "users");
        assert_eq!(Ident::new("select").to_sql(), "\"select\"");
        assert_eq!(Ident::quoted("MyTable").to_sql(), "\"MyTable\"");
    }

    #[test]
    fn test_starts_with_digit() {
        assert!(Ident::new("1col").needs_quotes());
    }
}
