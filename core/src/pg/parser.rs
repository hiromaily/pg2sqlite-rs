/// PostgreSQL DDL parser using sqlparser-rs.
///
/// Converts sqlparser AST into our internal representation (IR).
use sqlparser::ast::{
    self, AlterTableOperation, Array, ArrayElemTypeDef, BinaryOperator, ColumnDef, ColumnOption,
    CreateIndex, DataType, Expr as SqlExpr, ObjectName, ReferentialAction, Statement,
    TableConstraint as SqlConstraint, UserDefinedTypeRepresentation,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::{
    AlterConstraint, Column, EnumDef, Expr, FkAction, ForeignKeyRef, Ident, Index, IndexColumn,
    IndexMethod, PgType, QualifiedName, SchemaModel, Sequence, Table, TableConstraint,
};

/// Parse PostgreSQL DDL text into an IR SchemaModel.
pub fn parse(input: &str) -> (SchemaModel, Vec<Warning>) {
    let dialect = PostgreSqlDialect {};
    let mut model = SchemaModel::default();
    let mut warnings = Vec::new();

    let statements = match Parser::parse_sql(&dialect, input) {
        Ok(stmts) => stmts,
        Err(e) => {
            warnings.push(Warning::new(
                warning::PARSE_SKIPPED,
                Severity::Error,
                format!("Failed to parse DDL: {e}"),
            ));
            return (model, warnings);
        }
    };

    for stmt in statements {
        match stmt {
            Statement::CreateTable(ct) => {
                if let Some(table) = parse_create_table(&ct, &mut warnings) {
                    model.tables.push(table);
                }
            }
            Statement::CreateIndex(ci) => {
                if let Some(idx) = parse_create_index(&ci, &mut warnings) {
                    model.indexes.push(idx);
                }
            }
            Statement::CreateSequence { name, .. } => {
                model.sequences.push(Sequence {
                    name: convert_object_name(&name),
                    owned_by: None,
                });
            }
            Statement::AlterTable {
                name, operations, ..
            } => {
                let table_name = convert_object_name(&name);
                for op in operations {
                    if let Some(constraint) = parse_alter_table_op(&table_name, &op, &mut warnings)
                    {
                        model.alter_constraints.push(constraint);
                    }
                }
            }
            Statement::CreateType {
                name,
                representation: UserDefinedTypeRepresentation::Enum { labels },
                ..
            } => {
                let values: Vec<String> = labels.into_iter().map(|v| v.to_string()).collect();
                model.enums.push(EnumDef {
                    name: convert_object_name(&name),
                    values,
                });
            }
            // Skip non-DDL statements silently
            _ => {}
        }
    }

    (model, warnings)
}

fn parse_create_table(ct: &ast::CreateTable, warnings: &mut [Warning]) -> Option<Table> {
    let name = convert_object_name(&ct.name);
    let mut columns = Vec::new();
    let mut constraints = Vec::new();

    for element in &ct.columns {
        columns.push(parse_column(element));
    }

    for constraint in &ct.constraints {
        if let Some(tc) = parse_table_constraint(constraint, warnings) {
            constraints.push(tc);
        }
    }

    Some(Table {
        name,
        columns,
        constraints,
    })
}

fn parse_column(col_def: &ColumnDef) -> Column {
    let name = Ident::new(&col_def.name.value);
    let pg_type = convert_data_type(&col_def.data_type);
    let mut not_null = false;
    let mut default = None;
    let mut is_primary_key = false;
    let mut is_unique = false;
    let mut references = None;
    let mut check = None;

    for opt in &col_def.options {
        match &opt.option {
            ColumnOption::NotNull => not_null = true,
            ColumnOption::Null => not_null = false,
            ColumnOption::Default(expr) => {
                default = Some(convert_sql_expr(expr));
            }
            ColumnOption::Unique { is_primary, .. } => {
                if *is_primary {
                    is_primary_key = true;
                } else {
                    is_unique = true;
                }
            }
            ColumnOption::ForeignKey {
                foreign_table,
                referred_columns,
                on_delete,
                on_update,
                ..
            } => {
                let ref_col = referred_columns.first().map(|c| Ident::new(&c.value));
                references = Some(ForeignKeyRef {
                    table: convert_object_name(foreign_table),
                    column: ref_col,
                    on_delete: on_delete.as_ref().and_then(convert_referential_action),
                    on_update: on_update.as_ref().and_then(convert_referential_action),
                });
            }
            ColumnOption::Check(expr) => {
                check = Some(convert_sql_expr(expr));
            }
            _ => {}
        }
    }

    Column {
        name,
        pg_type,
        sqlite_type: None,
        not_null,
        default,
        is_primary_key,
        is_unique,
        references,
        check,
    }
}

fn parse_table_constraint(
    constraint: &SqlConstraint,
    _warnings: &mut [Warning],
) -> Option<TableConstraint> {
    match constraint {
        SqlConstraint::PrimaryKey { columns, name, .. } => {
            let cols: Vec<Ident> = columns.iter().map(|c| Ident::new(&c.value)).collect();
            Some(TableConstraint::PrimaryKey {
                name: name.as_ref().map(|n| Ident::new(&n.value)),
                columns: cols,
            })
        }
        SqlConstraint::Unique { columns, name, .. } => {
            let cols: Vec<Ident> = columns.iter().map(|c| Ident::new(&c.value)).collect();
            Some(TableConstraint::Unique {
                name: name.as_ref().map(|n| Ident::new(&n.value)),
                columns: cols,
            })
        }
        // Note:
        // The parser hardcodes deferrable: false for foreign key constraints.
        // PostgreSQL supports DEFERRABLE characteristics which should be extracted from the sqlparser AST
        // to ensure accurate transformation and warning emission later in the pipeline.
        SqlConstraint::ForeignKey {
            name,
            columns,
            foreign_table,
            referred_columns,
            on_delete,
            on_update,
            ..
        } => Some(TableConstraint::ForeignKey {
            name: name.as_ref().map(|n| Ident::new(&n.value)),
            columns: columns.iter().map(|c| Ident::new(&c.value)).collect(),
            ref_table: convert_object_name(foreign_table),
            ref_columns: referred_columns
                .iter()
                .map(|c| Ident::new(&c.value))
                .collect(),
            on_delete: on_delete.as_ref().and_then(convert_referential_action),
            on_update: on_update.as_ref().and_then(convert_referential_action),
            deferrable: false,
        }),
        SqlConstraint::Check { name, expr } => Some(TableConstraint::Check {
            name: name.as_ref().map(|n| Ident::new(&n.value)),
            expr: convert_sql_expr(expr),
        }),
        _ => None,
    }
}

fn parse_create_index(ci: &CreateIndex, _warnings: &mut [Warning]) -> Option<Index> {
    let index_name = ci.name.as_ref()?;
    let name = Ident::new(&index_name.to_string());
    let table = convert_object_name(&ci.table_name);

    let mut columns = Vec::new();
    for col in &ci.columns {
        let col_name = col.expr.to_string();
        // Check if this looks like a function call / expression
        if col_name.contains('(') {
            columns.push(IndexColumn::Expression(Expr::Raw(col_name)));
        } else {
            columns.push(IndexColumn::Column(Ident::new(&col_name)));
        }
    }

    let method = ci
        .using
        .as_ref()
        .and_then(|m| match m.value.to_lowercase().as_str() {
            "btree" => Some(IndexMethod::Btree),
            "hash" => Some(IndexMethod::Hash),
            "gin" => Some(IndexMethod::Gin),
            "gist" => Some(IndexMethod::Gist),
            "spgist" => Some(IndexMethod::SpGist),
            "brin" => Some(IndexMethod::Brin),
            _ => None,
        });

    let where_clause = ci.predicate.as_ref().map(convert_sql_expr);

    Some(Index {
        name,
        table,
        columns,
        unique: ci.unique,
        method,
        where_clause,
    })
}

fn parse_alter_table_op(
    table: &QualifiedName,
    op: &AlterTableOperation,
    warnings: &mut [Warning],
) -> Option<AlterConstraint> {
    match op {
        AlterTableOperation::AddConstraint(constraint) => {
            parse_table_constraint(constraint, warnings).map(|c| AlterConstraint {
                table: table.clone(),
                constraint: c,
            })
        }
        _ => None,
    }
}

/// Convert sqlparser ObjectName to our QualifiedName.
fn convert_object_name(name: &ObjectName) -> QualifiedName {
    let parts: Vec<&str> = name.0.iter().map(|ident| ident.value.as_str()).collect();
    match parts.len() {
        1 => QualifiedName::new(Ident::new(parts[0])),
        2 => QualifiedName::with_schema(Ident::new(parts[0]), Ident::new(parts[1])),
        _ => {
            // Take the last two parts as schema.table
            let len = parts.len();
            QualifiedName::with_schema(Ident::new(parts[len - 2]), Ident::new(parts[len - 1]))
        }
    }
}

/// Convert sqlparser DataType to our PgType.
fn convert_data_type(dt: &DataType) -> PgType {
    match dt {
        DataType::SmallInt(_) | DataType::Int2(_) => PgType::SmallInt,
        DataType::Integer(_) | DataType::Int(_) | DataType::Int4(_) => PgType::Integer,
        DataType::BigInt(_) | DataType::Int8(_) => PgType::BigInt,
        DataType::Real | DataType::Float4 => PgType::Real,
        DataType::Double | DataType::DoublePrecision | DataType::Float8 => PgType::DoublePrecision,
        DataType::Numeric(info) | DataType::Decimal(info) => {
            let (precision, scale) = extract_numeric_info(info);
            PgType::Numeric { precision, scale }
        }
        DataType::Boolean => PgType::Boolean,
        DataType::Text => PgType::Text,
        DataType::Varchar(len) | DataType::CharacterVarying(len) => PgType::Varchar {
            length: extract_char_length(len),
        },
        DataType::Char(len) | DataType::Character(len) => PgType::Char {
            length: extract_char_length(len),
        },
        DataType::Date => PgType::Date,
        DataType::Time(_, tz) => PgType::Time {
            with_tz: matches!(tz, ast::TimezoneInfo::WithTimeZone),
        },
        DataType::Timestamp(_, tz) => PgType::Timestamp {
            with_tz: matches!(tz, ast::TimezoneInfo::WithTimeZone),
        },
        DataType::Interval => PgType::Interval,
        DataType::Bytea => PgType::Bytea,
        DataType::Uuid => PgType::Uuid,
        DataType::JSON => PgType::Json,
        DataType::JSONB => PgType::Jsonb,
        DataType::Blob(_) => PgType::Bytea,
        DataType::Array(
            ArrayElemTypeDef::SquareBracket(inner, _) | ArrayElemTypeDef::AngleBracket(inner),
        ) => PgType::Array {
            element: Box::new(convert_data_type(inner)),
        },
        DataType::Array(_) => PgType::Other {
            name: dt.to_string(),
        },
        DataType::Custom(name, _) => {
            // Use the last part of the name to handle schema-qualified types (e.g., pg_catalog.serial)
            let type_name = name
                .0
                .last()
                .map(|id| id.value.to_lowercase())
                .unwrap_or_default();
            match type_name.as_str() {
                "serial" => PgType::Serial,
                "bigserial" => PgType::BigSerial,
                "smallserial" => PgType::SmallSerial,
                "inet" => PgType::Inet,
                "cidr" => PgType::Cidr,
                "macaddr" | "macaddr8" => PgType::MacAddr,
                "money" => PgType::Money,
                "xml" => PgType::Xml,
                "point" => PgType::Point,
                "line" => PgType::Line,
                "lseg" => PgType::Lseg,
                "box" => PgType::Box,
                "path" => PgType::Path,
                "polygon" => PgType::Polygon,
                "circle" => PgType::Circle,
                "int4range" => PgType::Int4Range,
                "int8range" => PgType::Int8Range,
                "numrange" => PgType::NumRange,
                "tsrange" => PgType::TsRange,
                "tstzrange" => PgType::TsTzRange,
                "daterange" => PgType::DateRange,
                _ => PgType::Other { name: type_name },
            }
        }
        _ => PgType::Other {
            name: dt.to_string(),
        },
    }
}

/// Convert sqlparser Expr to our Expr.
fn convert_sql_expr(expr: &SqlExpr) -> Expr {
    match expr {
        SqlExpr::Value(val) => convert_value(val),
        SqlExpr::Identifier(ident) => Expr::ColumnRef(ident.value.clone()),
        SqlExpr::CompoundIdentifier(idents) => {
            let name: Vec<&str> = idents.iter().map(|i| i.value.as_str()).collect();
            Expr::ColumnRef(name.join("."))
        }
        SqlExpr::Function(func) => {
            let func_name = func.name.to_string().to_lowercase();
            let args: Vec<Expr> = match &func.args {
                ast::FunctionArguments::List(arg_list) => arg_list
                    .args
                    .iter()
                    .filter_map(|arg| match arg {
                        ast::FunctionArg::Unnamed(ast::FunctionArgExpr::Expr(e)) => {
                            Some(convert_sql_expr(e))
                        }
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };

            // Detect nextval('sequence_name')
            if func_name == "nextval"
                && let Some(Expr::StringLiteral(seq)) = args.first()
            {
                return Expr::NextVal(seq.clone());
            }

            Expr::FunctionCall {
                name: func_name,
                args,
            }
        }
        SqlExpr::Cast {
            expr, data_type, ..
        } => Expr::Cast {
            expr: Box::new(convert_sql_expr(expr)),
            type_name: data_type.to_string(),
        },
        SqlExpr::BinaryOp { left, op, right } => Expr::BinaryOp {
            left: Box::new(convert_sql_expr(left)),
            op: op.to_string(),
            right: Box::new(convert_sql_expr(right)),
        },
        SqlExpr::UnaryOp { op, expr } => Expr::UnaryOp {
            op: op.to_string(),
            expr: Box::new(convert_sql_expr(expr)),
        },
        SqlExpr::IsNull(expr) => Expr::IsNull {
            expr: Box::new(convert_sql_expr(expr)),
            negated: false,
        },
        SqlExpr::IsNotNull(expr) => Expr::IsNull {
            expr: Box::new(convert_sql_expr(expr)),
            negated: true,
        },
        SqlExpr::InList {
            expr,
            list,
            negated,
        } => Expr::InList {
            expr: Box::new(convert_sql_expr(expr)),
            list: list.iter().map(convert_sql_expr).collect(),
            negated: *negated,
        },
        SqlExpr::Between {
            expr,
            low,
            high,
            negated,
        } => Expr::Between {
            expr: Box::new(convert_sql_expr(expr)),
            low: Box::new(convert_sql_expr(low)),
            high: Box::new(convert_sql_expr(high)),
            negated: *negated,
        },
        SqlExpr::Nested(inner) => Expr::Nested(Box::new(convert_sql_expr(inner))),
        // col = ANY(ARRAY['a', 'b']) → col IN ('a', 'b')
        // Only convert when the right-hand side is an ARRAY literal.
        // Non-array forms (e.g., subqueries) fall through to Raw to avoid
        // producing a semantically incorrect single-element InList.
        SqlExpr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => match extract_array_elements(right) {
            Some(list) => {
                let left_expr = convert_sql_expr(left);
                Expr::InList {
                    expr: Box::new(left_expr),
                    list,
                    negated: false,
                }
            }
            None => Expr::Raw(expr.to_string()),
        },
        // Note: `expr != ANY(ARRAY[...])` is NOT equivalent to `NOT IN (...)`.
        // `!= ANY` is true if expr differs from *at least one* element,
        // whereas `NOT IN` requires expr to differ from *all* elements.
        // The correct equivalent of `NOT IN` is `!= ALL(...)`, not `!= ANY(...)`.
        // We let `!= ANY` fall through to the Raw fallback below.
        //
        // Fallback: render back to SQL string
        _ => Expr::Raw(expr.to_string()),
    }
}

/// Extract elements from an ARRAY literal expression.
/// Returns `None` for non-array expressions (e.g., subqueries) so callers
/// can fall back to `Expr::Raw` instead of producing incorrect results.
fn extract_array_elements(expr: &SqlExpr) -> Option<Vec<Expr>> {
    match expr {
        SqlExpr::Array(Array { elem, .. }) => Some(elem.iter().map(convert_sql_expr).collect()),
        _ => None,
    }
}

fn convert_value(val: &ast::Value) -> Expr {
    match val {
        ast::Value::Number(n, _) => {
            if let Ok(i) = n.parse::<i64>() {
                Expr::IntegerLiteral(i)
            } else if let Ok(f) = n.parse::<f64>() {
                Expr::FloatLiteral(f)
            } else {
                Expr::Raw(n.clone())
            }
        }
        ast::Value::SingleQuotedString(s) => Expr::StringLiteral(s.clone()),
        ast::Value::Boolean(b) => Expr::BooleanLiteral(*b),
        ast::Value::Null => Expr::Null,
        _ => Expr::Raw(val.to_string()),
    }
}

fn convert_referential_action(action: &ReferentialAction) -> Option<FkAction> {
    match action {
        ReferentialAction::Cascade => Some(FkAction::Cascade),
        ReferentialAction::SetNull => Some(FkAction::SetNull),
        ReferentialAction::SetDefault => Some(FkAction::SetDefault),
        ReferentialAction::Restrict => Some(FkAction::Restrict),
        ReferentialAction::NoAction => Some(FkAction::NoAction),
    }
}

fn extract_numeric_info(info: &ast::ExactNumberInfo) -> (Option<u32>, Option<u32>) {
    match info {
        ast::ExactNumberInfo::PrecisionAndScale(p, s) => (Some(*p as u32), Some(*s as u32)),
        ast::ExactNumberInfo::Precision(p) => (Some(*p as u32), None),
        ast::ExactNumberInfo::None => (None, None),
    }
}

fn extract_char_length(len: &Option<ast::CharacterLength>) -> Option<u32> {
    len.as_ref().map(|cl| match cl {
        ast::CharacterLength::IntegerLength { length, .. } => *length as u32,
        ast::CharacterLength::Max => u32::MAX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_table() {
        let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);";
        let (model, warnings) = parse(sql);
        assert!(warnings.is_empty());
        assert_eq!(model.tables.len(), 1);
        let table = &model.tables[0];
        assert_eq!(table.name.name.normalized, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.columns[0].is_primary_key);
        assert!(table.columns[1].not_null);
    }

    #[test]
    fn test_parse_schema_qualified_table() {
        let sql = "CREATE TABLE public.users (id INTEGER);";
        let (model, _) = parse(sql);
        let table = &model.tables[0];
        assert_eq!(table.name.schema.as_ref().unwrap().normalized, "public");
        assert_eq!(table.name.name.normalized, "users");
    }

    #[test]
    fn test_parse_create_index() {
        let sql = "CREATE INDEX idx_name ON users (name);";
        let (model, _) = parse(sql);
        assert_eq!(model.indexes.len(), 1);
        assert_eq!(model.indexes[0].name.normalized, "idx_name");
        assert!(!model.indexes[0].unique);
    }

    #[test]
    fn test_parse_unique_index() {
        let sql = "CREATE UNIQUE INDEX idx_email ON users (email);";
        let (model, _) = parse(sql);
        assert!(model.indexes[0].unique);
    }

    #[test]
    fn test_parse_alter_table_add_constraint() {
        let sql = r#"
            CREATE TABLE orders (id INTEGER, user_id INTEGER);
            ALTER TABLE orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users (id);
        "#;
        let (model, _) = parse(sql);
        assert_eq!(model.tables.len(), 1);
        assert_eq!(model.alter_constraints.len(), 1);
    }

    #[test]
    fn test_parse_create_type_enum() {
        let sql = "CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');";
        let (model, _) = parse(sql);
        assert_eq!(model.enums.len(), 1);
        assert_eq!(model.enums[0].values.len(), 3);
    }

    #[test]
    fn test_parse_column_default() {
        let sql = "CREATE TABLE t (created_at TIMESTAMP DEFAULT now());";
        let (model, _) = parse(sql);
        let col = &model.tables[0].columns[0];
        assert!(col.default.is_some());
    }

    #[test]
    fn test_non_ddl_ignored() {
        let sql = "SELECT 1; CREATE TABLE t (id INTEGER);";
        let (model, warnings) = parse(sql);
        assert_eq!(model.tables.len(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_foreign_key_with_actions() {
        let sql = r#"
            CREATE TABLE orders (
                id INTEGER PRIMARY KEY,
                user_id INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE SET NULL
            );
        "#;
        let (model, _) = parse(sql);
        let col = &model.tables[0].columns[1];
        let fk = col.references.as_ref().unwrap();
        assert_eq!(fk.on_delete, Some(FkAction::Cascade));
        assert_eq!(fk.on_update, Some(FkAction::SetNull));
    }

    #[test]
    fn test_parse_check_constraint() {
        let sql = "CREATE TABLE t (age INTEGER CHECK (age >= 0));";
        let (model, _) = parse(sql);
        assert!(model.tables[0].columns[0].check.is_some());
    }

    #[test]
    fn test_parse_any_array_to_in_list() {
        let sql = r#"CREATE TABLE t (
            status TEXT NOT NULL,
            CONSTRAINT status_check CHECK ((status = ANY (ARRAY['active'::text, 'inactive'::text])))
        );"#;
        let (model, _) = parse(sql);
        let table = &model.tables[0];
        assert_eq!(table.constraints.len(), 1);
        if let TableConstraint::Check { name, expr } = &table.constraints[0] {
            assert_eq!(name.as_ref().unwrap().normalized, "status_check");
            // Should be Nested(InList { ... })
            if let Expr::Nested(inner) = expr {
                if let Expr::InList {
                    expr: col,
                    list,
                    negated,
                } = inner.as_ref()
                {
                    assert!(!negated);
                    assert!(matches!(col.as_ref(), Expr::ColumnRef(name) if name == "status"));
                    assert_eq!(list.len(), 2);
                    // Casts should be preserved at parse level (stripped during transform)
                    assert!(
                        matches!(&list[0], Expr::Cast { expr, .. } if matches!(expr.as_ref(), Expr::StringLiteral(s) if s == "active"))
                    );
                } else {
                    panic!("Expected InList, got: {inner:?}");
                }
            } else {
                panic!("Expected Nested, got: {expr:?}");
            }
        } else {
            panic!("Expected Check constraint");
        }
    }
}
