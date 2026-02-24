/// Core IR model types for representing parsed DDL schemas.
use super::expr::Expr;
use super::ident::{Ident, QualifiedName};
use super::types::{PgType, SqliteType};

/// The top-level schema model containing all parsed DDL objects.
#[derive(Debug, Clone, Default)]
pub struct SchemaModel {
    pub tables: Vec<Table>,
    pub indexes: Vec<Index>,
    pub sequences: Vec<Sequence>,
    pub enums: Vec<EnumDef>,
    pub domains: Vec<DomainDef>,
    pub alter_constraints: Vec<AlterConstraint>,
    pub identity_columns: Vec<AlterIdentity>,
}

/// A parsed CREATE TABLE statement.
#[derive(Debug, Clone)]
pub struct Table {
    pub name: QualifiedName,
    pub columns: Vec<Column>,
    pub constraints: Vec<TableConstraint>,
}

/// A column definition within a table.
#[derive(Debug, Clone)]
pub struct Column {
    pub name: Ident,
    pub pg_type: PgType,
    pub sqlite_type: Option<SqliteType>,
    pub not_null: bool,
    pub default: Option<Expr>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub autoincrement: bool,
    pub references: Option<ForeignKeyRef>,
    pub check: Option<Expr>,
}

/// Table-level constraint.
#[derive(Debug, Clone)]
pub enum TableConstraint {
    PrimaryKey {
        name: Option<Ident>,
        columns: Vec<Ident>,
    },
    Unique {
        name: Option<Ident>,
        columns: Vec<Ident>,
    },
    ForeignKey {
        name: Option<Ident>,
        columns: Vec<Ident>,
        ref_table: QualifiedName,
        ref_columns: Vec<Ident>,
        on_delete: Option<FkAction>,
        on_update: Option<FkAction>,
        deferrable: bool,
    },
    Check {
        name: Option<Ident>,
        expr: Expr,
    },
}

/// An ALTER TABLE ... ADD CONSTRAINT that needs merging.
#[derive(Debug, Clone)]
pub struct AlterConstraint {
    pub table: QualifiedName,
    pub constraint: TableConstraint,
}

/// An ALTER TABLE ... ALTER COLUMN ... ADD GENERATED AS IDENTITY.
#[derive(Debug, Clone)]
pub struct AlterIdentity {
    pub table: QualifiedName,
    pub column: Ident,
}

/// Foreign key reference from a column-level constraint.
#[derive(Debug, Clone)]
pub struct ForeignKeyRef {
    pub table: QualifiedName,
    pub column: Option<Ident>,
    pub on_delete: Option<FkAction>,
    pub on_update: Option<FkAction>,
}

/// Foreign key referential action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FkAction {
    Cascade,
    SetNull,
    SetDefault,
    Restrict,
    NoAction,
}

impl std::fmt::Display for FkAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FkAction::Cascade => write!(f, "CASCADE"),
            FkAction::SetNull => write!(f, "SET NULL"),
            FkAction::SetDefault => write!(f, "SET DEFAULT"),
            FkAction::Restrict => write!(f, "RESTRICT"),
            FkAction::NoAction => write!(f, "NO ACTION"),
        }
    }
}

/// A CREATE INDEX statement.
#[derive(Debug, Clone)]
pub struct Index {
    pub name: Ident,
    pub table: QualifiedName,
    pub columns: Vec<IndexColumn>,
    pub unique: bool,
    pub method: Option<IndexMethod>,
    pub where_clause: Option<Expr>,
}

/// A column or expression in an index.
#[derive(Debug, Clone)]
pub enum IndexColumn {
    Column(Ident),
    Expression(Expr),
}

/// Index access method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexMethod {
    Btree,
    Hash,
    Gin,
    Gist,
    SpGist,
    Brin,
}

impl std::fmt::Display for IndexMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexMethod::Btree => write!(f, "btree"),
            IndexMethod::Hash => write!(f, "hash"),
            IndexMethod::Gin => write!(f, "gin"),
            IndexMethod::Gist => write!(f, "gist"),
            IndexMethod::SpGist => write!(f, "spgist"),
            IndexMethod::Brin => write!(f, "brin"),
        }
    }
}

/// A CREATE SEQUENCE statement.
#[derive(Debug, Clone)]
pub struct Sequence {
    pub name: QualifiedName,
    pub owned_by: Option<(QualifiedName, Ident)>,
}

/// A CREATE TYPE ... AS ENUM statement.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: QualifiedName,
    pub values: Vec<String>,
}

/// A CREATE DOMAIN statement.
#[derive(Debug, Clone)]
pub struct DomainDef {
    pub name: QualifiedName,
    pub base_type: PgType,
    pub not_null: bool,
    pub default: Option<Expr>,
    pub check: Option<Expr>,
}
