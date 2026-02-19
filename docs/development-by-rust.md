Below is a **Rust-oriented module design proposal** for a **PostgreSQL 16 DDL → SQLite3 DDL** schema converter, aligned with the rule tables you already requested. It’s structured so an AI agent can implement it with clear boundaries and testability.

---

# Rust Module Design Proposal (PG16 Schema → SQLite3 Schema Converter)

## 1. Crate Layout (Recommended)

```
pg2sqlite-schema/
  Cargo.toml
  src/
    main.rs
    lib.rs

    cli/
      mod.rs
      args.rs

    input/
      mod.rs
      reader.rs

    pg/
      mod.rs
      lexer.rs        (optional, only if not using external parser)
      parser.rs       (wraps external SQL parser or custom parsing)
      ast.rs          (PG AST types or re-exports)
      normalize.rs    (identifier normalization + schema filtering)
      diagnostics.rs  (PG parse warnings)

    ir/
      mod.rs
      model.rs        (SchemaModel, Table, Column, Index, etc.)
      expr.rs         (Expr IR for defaults/check/where)
      ident.rs        (QualifiedName, Identifier, quoting rules)
      span.rs         (Span/Location for diagnostics)

    transform/
      mod.rs
      planner.rs      (collects ALTERs, resolves sequences, merges constraints)
      type_map.rs     (PG type -> SQLite type/affinity mapping)
      expr_map.rs     (default/check/where expression conversion)
      constraint.rs   (PK/UNIQUE/FK/CHECK mapping)
      index.rs        (index conversion rules, compatibility checks)
      name_resolve.rs (schema stripping, collision strategy)
      topo.rs         (dependency ordering for FK)

    sqlite/
      mod.rs
      ir.rs           (SQLite-specific IR (optional) or reuse shared IR)
      render.rs       (DDL renderer)
      pragma.rs       (PRAGMA foreign_keys, optional settings)

    diagnostics/
      mod.rs
      warning.rs      (Warning, Severity, codes)
      reporter.rs     (stderr/json file output, strict mode behavior)

    tests/
      mod.rs
      fixtures/
        *.sql
      golden/
        *.out.sql
```

You can also collapse `sqlite/ir.rs` into the shared IR if you keep it generic enough. A two-IR approach (PG-IR → SQLite-IR) tends to be cleaner.

---

## 2. Core Data Flow

### 2.1 Pipeline

1. **Read input** (`input::reader`)
2. **Parse PG DDL** (`pg::parser`) → PG AST (or direct PG IR)
3. **Normalize & Filter** (`pg::normalize`) → PG IR
4. **Plan & Merge** (`transform::planner`)

   * merge `ALTER TABLE ... ADD CONSTRAINT` into owning table
   * resolve sequences / serial / identity
5. **Transform** (multiple passes in `transform::*`) → SQLite IR
6. **Order statements** (`transform::topo`)
7. **Render SQLite DDL** (`sqlite::render`)
8. **Emit diagnostics** (`diagnostics::*`)

---

## 3. Key Public APIs

### 3.1 Library Entry Point

Expose a library API so CLI is thin:

```rust
pub struct ConvertOptions {
    pub schema: Option<String>,
    pub include_all_schemas: bool,
    pub enable_foreign_keys: bool,
    pub strict: bool,
    pub collision_strategy: CollisionStrategy,
    pub sqlite_features: SqliteFeatures,
}

pub struct ConvertResult {
    pub sqlite_sql: String,
    pub warnings: Vec<Warning>,
}

pub fn convert_pg_ddl_to_sqlite(input: &str, opts: &ConvertOptions) -> Result<ConvertResult, ConvertError>;
```

### 3.2 Feature Flags / Compatibility

```rust
pub struct SqliteFeatures {
    pub allow_partial_indexes: bool,
    pub allow_expression_indexes: bool,
    pub emulate_enum_check: bool,
    pub emulate_uuid_default: bool,
    pub emit_autoincrement: bool,
}
```

---

## 4. IR Design (Rust Structs)

### 4.1 Shared “Schema IR” (Language-agnostic)

In `ir/model.rs`:

* `SchemaModel { tables, indexes, sequences, warnings }`
* `Table { name: QualifiedName, columns: Vec<Column>, constraints: Vec<TableConstraint> }`
* `Column { name: Identifier, ty: TypeRef, not_null, default: Option<Expr>, constraints: Vec<ColumnConstraint> }`
* `Index { name, table, unique, keys: Vec<IndexKey>, where_clause: Option<Expr> }`

`ir/expr.rs`:

* `Expr` should be minimal but expressive:

  * literals (`Null`, `Bool`, `Int`, `Float`, `String`)
  * identifiers
  * unary/binary operators
  * function calls (name + args)
  * parenthesized/grouping
  * cast (PG) node optionally (so you can strip it)

`ir/ident.rs`:

* `Identifier { raw: String, normalized: String, was_quoted: bool }`
* `QualifiedName { schema: Option<Identifier>, name: Identifier }`
* `fn needs_quotes(&Identifier) -> bool`

### 4.2 SQLite IR (Optional but Recommended)

In `sqlite/ir.rs`, define SQLite-ready types (already mapped):

* `SqliteColumnType: enum { Integer, Text, Real, Numeric, Blob, Custom(String) }`
* constraints already in SQLite syntax
* expressions normalized to SQLite function/operator set

This reduces “accidental PG-isms” in the renderer.

---

## 5. Parsing Strategy Module (`pg::parser`)

### 5.1 Options

* **Preferred**: Wrap an existing SQL parser crate that supports Postgres dialect.
* If the parser returns a generic AST, write adapters:

  * `ast_to_ir.rs` (PG AST → PG IR)

### 5.2 Parser Output Requirements

The parser stage must identify:

* `CREATE TABLE` (columns + inline constraints + table constraints)
* `ALTER TABLE ... ADD CONSTRAINT`
* `CREATE INDEX`
* `CREATE TYPE ... AS ENUM`
* `CREATE DOMAIN`
* `CREATE SEQUENCE`
* ignore: `COMMENT`, `SET`, `SELECT set_config`

If parsing fails:

* return `ConvertError::ParseError` with span/line/col if possible.

---

## 6. Normalization (`pg::normalize`)

Responsibilities:

* Apply schema filtering:

  * `--schema public` default
  * `--include-all-schemas` bypass
* Normalize identifiers:

  * handle quoting rules and case folding
* Collect named objects in symbol tables:

  * tables by qualified name
  * enum types by qualified name
  * domains by qualified name
  * sequences by qualified name

Outputs:

* `SchemaModel` (PG IR) + symbol tables.

---

## 7. Transformation Passes (`transform::*`)

### 7.1 Planner Pass (`transform::planner`)

Goal: restructure PG IR into a form that SQLite can express.

Tasks:

* Merge `ALTER TABLE ... ADD CONSTRAINT` into the target table IR.
* Convert `SERIAL/BIGSERIAL` and `GENERATED AS IDENTITY`:

  * detect “single-column integer primary key” → mark column as `RowIdAliasCandidate`.
* Resolve enums/domains:

  * attach enum value list to columns (for optional CHECK emulation)
  * flatten domain constraints into column/table constraints
* Identify unsupported constructs early and create warnings/errors.

### 7.2 Type Mapping (`transform::type_map`)

Input: PG `TypeRef` (+ domain/enum metadata)
Output: SQLite type + optional added constraints (e.g., enum CHECK)

Key function:

```rust
pub fn map_type(pg_type: &TypeRef, ctx: &TypeContext, opts: &ConvertOptions) -> (SqliteColumnType, Vec<Warning>, Vec<ExtraConstraint>);
```

### 7.3 Expression Mapping (`transform::expr_map`)

Converts PG `Expr` → SQLite `Expr` or rejects it.

Rules:

* map `true/false` → `1/0`
* map `now()` → `CURRENT_TIMESTAMP`
* strip `::type` casts where safe
* unsupported functions → warn + drop expression (depending on context)
* used by:

  * column DEFAULT
  * CHECK constraints
  * partial index WHERE clause
  * expression index keys

API:

```rust
pub fn map_expr(expr: &Expr, mode: ExprMode, opts: &ConvertOptions) -> ExprMappingResult;
```

Where `ExprMode` influences strictness:

* `DefaultExpr`, `CheckExpr`, `IndexWhere`, `IndexExpr`

### 7.4 Constraint Mapping (`transform::constraint`)

Handles:

* PK selection logic (rowid alias vs composite)
* UNIQUE constraints
* FK constraints (action mapping)
* CHECK constraints (depends on expr_map)

Outputs: SQLite-ready constraints and warnings.

### 7.5 Index Conversion (`transform::index`)

Rules:

* ignore index methods; warn if non-btree
* decide skip/emit for:

  * partial index (requires sqlite feature + expr_map ok)
  * expression index (requires sqlite feature + expr_map ok)

### 7.6 Name Resolution & Collision (`transform::name_resolve`)

SQLite has no schemas. Implement:

* schema stripping
* collision handling:

  * `CollisionStrategy::PrefixSchema` => `schema__table`
  * `CollisionStrategy::Error` => fail on collision in strict
  * `CollisionStrategy::WarnAndMangle` => add numeric suffix

Maintain a mapping table:

* original qualified name → sqlite identifier
  Used by FK reference rewriting and index table rewriting.

### 7.7 Dependency Ordering (`transform::topo`)

If foreign keys enabled:

* build graph table → referenced table
* topological sort
* warn on cycles and fall back to alphabetical

---

## 8. Rendering (`sqlite::render`)

### 8.1 Renderer Responsibilities

* Quote identifiers consistently
* Emit:

  * optional `PRAGMA foreign_keys = ON;`
  * `CREATE TABLE` statements
  * `CREATE INDEX` statements
* Deterministic formatting:

  * stable ordering
  * consistent indentation

API:

```rust
pub fn render_schema(model: &SqliteSchemaModel, opts: &ConvertOptions) -> String;
```

Keep renderer “dumb”: no PG logic; all decisions made in transform passes.

---

## 9. Diagnostics System (`diagnostics::*`)

### 9.1 Data Model

```rust
pub enum Severity { Info, Warning, Lossy, Unsupported, Error }

pub struct Warning {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub object: Option<String>,   // table/column/index identifier
    pub span: Option<Span>,
}
```

### 9.2 Strict Mode Policy

In `diagnostics::reporter`:

* If `opts.strict`:

  * treat `Lossy` and above as errors (configurable)
  * return `ConvertError::StrictViolation(Vec<Warning>)`

---

## 10. Error Types (`ConvertError`)

Recommend structured errors:

* `IoError`
* `ParseError { message, span }`
* `InvalidDdl { message, span }`
* `StrictViolation { warnings }`
* `Internal { message }`

---

## 11. Testing Strategy (Rust)

### 11.1 Unit Tests

* `type_map` mapping table tests
* `expr_map` for defaults (`now()`, boolean, casts)
* identifier quoting (`needs_quotes`)
* FK action mapping

### 11.2 Golden Tests

`tests/golden/*.sql` → expected `*.out.sql`

* stable ordering checks
* include ALTER constraints merging cases

### 11.3 Snapshot Tests (optional)

Use snapshot testing for rendered SQL strings.

---

## 12. Implementation Notes (Practical)

* Keep transformation passes pure (input → output + warnings).
* Store symbol tables in a `Context` struct passed to transform passes.
* Prefer `Arc<str>` or interning for identifiers if performance matters, but not required for MVP.
* Make “skipping” explicit: return `Option<T>` plus warning rather than silently dropping.
