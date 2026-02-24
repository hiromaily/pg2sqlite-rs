# sqlparser-rs Reference for AI Agents

This document describes how `sqlparser-rs` is used in pg2sqlite-rs, and provides guidance for upgrading the dependency.

- **Repository**: <https://github.com/apache/datafusion-sqlparser-rs>
- **Crate**: <https://crates.io/crates/sqlparser>
- **Current version**: `0.61` (as of 2026-02-24)

---

## 1. Role in pg2sqlite-rs

`sqlparser-rs` is the **only external parser** used to convert raw PostgreSQL DDL text into an AST. All sqlparser interaction is isolated in a single file:

```
core/src/pg/parser.rs   ← sole consumer of sqlparser APIs
```

No other module imports from `sqlparser`. The parser converts the sqlparser AST into pg2sqlite-rs's own IR (`SchemaModel`), which flows through the rest of the pipeline without any sqlparser dependency.

## 2. Dependency Configuration

```toml
# core/Cargo.toml
sqlparser = { version = "0.61", features = ["std", "visitor", "recursive-protection"] }
```

| Feature | Purpose |
|---------|---------|
| `std` | Standard library support (required) |
| `visitor` | Visitor pattern API (available but currently unused) |
| `recursive-protection` | Guards against stack overflow on deeply nested expressions |

## 3. Parser Invocation

```rust
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

let dialect = PostgreSqlDialect {};
let statements = Parser::parse_sql(&dialect, &input)?;
```

**Pre-processing**: Before parsing, `strip_identity_options()` removes `AS IDENTITY (...)` blocks from `pg_dump` output because sqlparser cannot parse the inner sequence options.

## 4. Imported Types (18 total)

```rust
use sqlparser::ast::{
    self, AlterColumnOperation, AlterTableOperation, Array, ArrayElemTypeDef,
    BinaryOperator, ColumnDef, ColumnOption, CreateIndex, DataType,
    Expr as SqlExpr, ObjectName, ObjectNamePart, ReferentialAction,
    Statement, TableConstraint as SqlConstraint,
    UserDefinedTypeRepresentation, ValueWithSpan,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
```

## 5. AST Variants Matched

### 5.1 Statement Variants

| Variant | Handler | IR Output |
|---------|---------|-----------|
| `CreateTable` | `parse_create_table()` | `Table` (columns + constraints) |
| `CreateIndex` | `parse_create_index()` | `Index` |
| `CreateSequence` | Direct mapping | `Sequence` |
| `AlterTable` | `parse_alter_table_op()` | `AlterConstraint` / `AlterIdentity` |
| `CreateType` | Direct matching | `EnumDef` (enum representation only) |
| All others | Silently skipped | — |

### 5.2 DataType Variants (40+)

**Integer**: `SmallInt`, `Int2`, `Integer`, `Int`, `Int4`, `BigInt`, `Int8`

**Numeric**: `Numeric(precision, scale)`, `Decimal(precision, scale)`, `Real`, `Float4`, `Double`, `DoublePrecision`, `Float8`

**Character**: `Text`, `Varchar(length)`, `CharacterVarying(length)`, `Char(length)`, `Character(length)`

**Boolean / Date-Time**: `Boolean`, `Date`, `Time(TimezoneInfo)`, `Timestamp(TimezoneInfo)`, `Interval`

**Binary / UUID / JSON**: `Bytea`, `Blob`, `Uuid`, `JSON`, `JSONB`

**Array**: `Array(ArrayElemTypeDef::SquareBracket(...))`, `Array(ArrayElemTypeDef::AngleBracket(...))`

**Custom types** (matched by name string): `serial`, `bigserial`, `smallserial`, `inet`, `cidr`, `macaddr`, geometric types, range types, etc.

### 5.3 Expression Variants (16)

`Value`, `Identifier`, `CompoundIdentifier`, `Function`, `Cast`, `BinaryOp`, `UnaryOp`, `IsNull`, `IsNotNull`, `InList`, `Between`, `Nested`, `AnyOp`, `Array`

Special: `nextval('seq')` is detected and converted to `Expr::NextVal`.

### 5.4 Other Matched Variants

- **ColumnOption**: `NotNull`, `Null`, `Default`, `PrimaryKey`, `Unique`, `ForeignKey`, `Check`
- **TableConstraint**: `PrimaryKey`, `Unique`, `ForeignKey`, `Check`
- **ReferentialAction**: `Cascade`, `SetNull`, `SetDefault`, `Restrict`, `NoAction`
- **AlterTableOperation**: `AddConstraint`, `AlterColumn(AddGenerated)`
- **IndexType**: `BTree`, `Hash`, `GIN`, `GiST`, `SPGiST`, `BRIN`
- **Value**: `Number`, `SingleQuotedString`, `Boolean`, `Null`

## 6. PostgreSqlDialect Capabilities

The `PostgreSqlDialect` in sqlparser supports:

- **String literals**: `U&'...'`, `E'...'`, numeric underscores
- **Operators**: Custom operators, `<<`/`>>`, `!`, `NOTNULL`
- **Functions**: Named arguments (`:`, `=`, `=>`), `FILTER` in aggregates
- **Data types**: Geometric types, array brackets, `INTERVAL` qualifiers
- **DDL**: `CREATE TYPE` (all forms), `CREATE OPERATOR`, `PARTITION OF`, `ALTER TABLE`, index operator classes
- **Other**: `TABLESAMPLE`, `LISTEN/NOTIFY`, `COMMENT ON`

---

## 7. Upgrade Guide

### 7.1 Versioning Policy

sqlparser-rs treats **any AST change as a breaking change**, incrementing the minor version (e.g., 0.61 → 0.62). Breaking changes are typically:

- New fields added to AST structs (breaks struct construction)
- New enum variants (breaks exhaustive `match`)
- Struct/enum moves (e.g., nested structs moved to module level)

There is **no formal migration guide**. Changes are documented per-release in the `changelog/` directory.

### 7.2 Upgrade Steps

```bash
# 1. Check the latest version
cargo search sqlparser

# 2. Review changelog for breaking changes
#    Check: https://github.com/apache/datafusion-sqlparser-rs/tree/main/changelog
#    Or fetch the specific version changelog:
#    https://github.com/apache/datafusion-sqlparser-rs/blob/main/changelog/0.XX.0.md

# 3. Update version in core/Cargo.toml
#    sqlparser = { version = "0.XX", features = ["std", "visitor", "recursive-protection"] }

# 4. Run cargo check to identify compilation errors
cargo check -p pg2sqlite-core

# 5. Fix compilation errors (typically in core/src/pg/parser.rs)
#    Common fixes:
#    - Add wildcard arms to match statements for new enum variants
#    - Update struct field accesses for renamed/moved fields
#    - Adjust import paths for moved types

# 6. Run full CI to verify
make ci
```

### 7.3 Common Fix Patterns

**New enum variants added** (most common):

```rust
// Before: exhaustive match
match data_type {
    DataType::Integer => ...,
    DataType::Text => ...,
    _ => handle_unknown(),
}
// Usually no fix needed if you have a wildcard arm.
// If not, add one.
```

**Struct fields added**:

```rust
// Error: missing field `new_field` in pattern
// Fix: add `..` to pattern or explicitly handle the new field
Statement::CreateTable { name, columns, .. } => { ... }
```

**Types moved to different module path**:

```rust
// Error: unresolved import `sqlparser::ast::SomeType`
// Fix: update import path per changelog
use sqlparser::ast::new_module::SomeType;
```

### 7.4 What to Check After Upgrade

1. **All tests pass**: `make test`
2. **Golden test outputs unchanged**: If output changes, verify the new output is correct and update golden files
3. **New PostgreSQL DDL support**: Check if new `Statement` variants or `DataType` variants are relevant for pg2sqlite-rs and should be handled (not just ignored)
4. **Pre-processing still needed**: Check if `strip_identity_options()` is still necessary (sqlparser may add native support for `AS IDENTITY` options in a future version)

### 7.5 Recent Version History

| Version | Date | Key PostgreSQL Additions |
|---------|------|--------------------------|
| 0.61.0 | Feb 2026 | `PARTITION OF`, operator classes in indexes, `ANALYZE`, `ALTER OPERATOR` |
| 0.60.0 | Dec 2025 | Complete `CREATE TYPE`, PostgreSQL operator DDL, constraint unification |

### 7.6 Checking for New Releases

```bash
# Via cargo
cargo search sqlparser

# Via GitHub releases
gh api repos/apache/datafusion-sqlparser-rs/releases --jq '.[0:5] | .[] | "\(.tag_name) \(.published_at)"'

# Via crates.io
curl -s https://crates.io/api/v1/crates/sqlparser | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['crate']['newest_version'])"
```
