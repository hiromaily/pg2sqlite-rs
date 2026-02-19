## Context

The project starts from a copied pg2sqlite-rs workspace (`core/` library + `cli/` binary). The pg2sqlite codebase provides reusable infrastructure — clap CLI, thiserror errors, output formatters, Makefile — but all domain logic (linting rules, YAML parsing, config) is replaced.

Three specification documents define the conversion requirements:

- `docs/postgres-to-sqlite.md` — base spec covering pipeline, IR, CLI, and all conversion rules
- `docs/postgres-to-sqlite-rule.md` — detailed mapping tables for types, constraints, defaults, indexes with warning codes
- `docs/development-by-rust.md` — Rust module design with crate layout, data flow, IR structs, and public APIs

The converter is an offline, text-to-text tool. No database connections. Input is PostgreSQL 16 DDL text; output is valid SQLite3 DDL.

## Goals / Non-Goals

**Goals:**

- Parse PostgreSQL DDL (CREATE TABLE, ALTER TABLE ADD CONSTRAINT, CREATE INDEX, CREATE SEQUENCE, CREATE TYPE ENUM, CREATE DOMAIN)
- Transform to SQLite-compatible DDL with type mapping, constraint merging, and expression conversion
- Produce deterministic output (stable ordering for diffs)
- Emit structured warnings for every lossy conversion
- Library-first design: `core` crate exposes `convert_pg_ddl_to_sqlite()` as the primary API; CLI is a thin wrapper

**Non-Goals:**

- Data migration (COPY/INSERT)
- View, function, trigger, materialized view conversion
- Live database connectivity
- Full semantic equivalence (partial indexes and expression indexes are best-effort)

## Decisions

### 1. SQL Parser: `sqlparser` crate with PostgreSQL dialect

Use `sqlparser-rs` (the `sqlparser` crate) configured for the PostgreSQL dialect to parse DDL into an AST.

**Why over alternatives:**

- `pg_query` (libpg_query bindings): Requires C dependency, complicates builds and cross-compilation. More accurate but overkill for DDL-only parsing.
- Custom parser: High effort, error-prone, hard to maintain as PostgreSQL evolves.
- `sqlparser` is pure Rust, well-maintained, supports PostgreSQL dialect, and handles the DDL statements we need. Its AST is generic across dialects, so we write adapters (`pg::parser`) to extract our IR.

**Trade-off:** `sqlparser` may not parse every PostgreSQL-specific syntax perfectly (e.g., some advanced `ALTER TABLE` forms). Mitigation: unrecognized statements are skipped with a warning rather than causing a hard error.

### 2. Two-IR Architecture: PG IR → SQLite IR

Maintain separate intermediate representations rather than transforming in-place.

- **PG IR** (`ir/model.rs`): Captures parsed PostgreSQL schema objects with their original types, constraints, defaults
- **SQLite IR** (reuses same structs with mapped types): After transformation, types are SQLite affinities, expressions are SQLite-compatible, constraints are merged

**Why:** Keeps parsing and rendering completely decoupled. Transform passes are pure functions (input IR + options → output IR + warnings). Easier to test each pass independently.

**Alternative considered:** Single IR with "source" and "target" flags. Rejected because it conflates PG-specific data (enum definitions, domain metadata, sequence references) with SQLite output, making the code harder to reason about.

### 3. Module Layout

```
core/src/
  lib.rs              → pub fn convert_pg_ddl_to_sqlite(), ConvertOptions, ConvertResult, ConvertError
  pg/
    mod.rs
    parser.rs          → wraps sqlparser, extracts PG DDL into IR
    normalize.rs       → schema filtering, identifier case folding
  ir/
    mod.rs
    model.rs           → SchemaModel, Table, Column, Index, Sequence, Warning
    expr.rs            → Expr enum (literals, operators, function calls, casts)
    types.rs           → PgType, SqliteType enums
    ident.rs           → Identifier, QualifiedName, needs_quotes()
  transform/
    mod.rs
    planner.rs         → merge ALTER TABLE constraints, resolve SERIAL/IDENTITY/sequences
    type_map.rs        → PgType → SqliteType mapping with warnings
    expr_map.rs        → PG Expr → SQLite Expr (defaults, CHECK, WHERE)
    constraint.rs      → PK/UNIQUE/FK/CHECK transformation
    index.rs           → index conversion, method filtering
    name_resolve.rs    → schema stripping, collision handling
    topo.rs            → topological sort for FK dependencies
  sqlite/
    mod.rs
    render.rs          → DDL text generation from transformed IR
  diagnostics/
    mod.rs
    warning.rs         → Warning struct, Severity enum, warning codes
    reporter.rs        → output formatting (stderr, file), strict mode

cli/src/
  main.rs             → clap CLI, file I/O, delegates to core::convert_pg_ddl_to_sqlite()
```

### 4. Pipeline Execution Order

```
Input DDL text
  → pg::parser::parse()           — sqlparser AST → PG IR (SchemaModel)
  → pg::normalize::normalize()    — schema filtering, identifier normalization
  → transform::planner::plan()    — merge ALTERs, resolve SERIAL/sequences, attach enum metadata
  → transform (type_map, expr_map, constraint, index, name_resolve) — PG IR → SQLite IR
  → transform::topo::order()      — topological sort if FK enabled
  → sqlite::render::render()      — SQLite IR → DDL text
  → diagnostics::reporter         — emit warnings
```

Each pass is a pure function taking IR + options and returning transformed IR + warnings. Warnings are accumulated in a `Vec<Warning>` threaded through the pipeline.

### 5. SERIAL/IDENTITY Handling

SERIAL columns generate a sequence + DEFAULT nextval('seq'). The planner pass detects this pattern:

- If the column is a single-column integer primary key → emit as `INTEGER PRIMARY KEY` (SQLite rowid alias, auto-increment without AUTOINCREMENT keyword)
- Otherwise → map type to INTEGER, drop the DEFAULT, warn `SERIAL_NOT_PRIMARY_KEY`

GENERATED ALWAYS/BY DEFAULT AS IDENTITY follows the same logic.

### 6. ALTER TABLE Constraint Merging

PostgreSQL commonly uses `ALTER TABLE ... ADD CONSTRAINT` after CREATE TABLE. SQLite cannot add most constraints via ALTER TABLE.

Strategy: The planner collects all `ALTER TABLE ... ADD CONSTRAINT` statements and merges them into the corresponding `CREATE TABLE` IR before transformation. If the target table is not found in the IR, warn `ALTER_TARGET_MISSING` and skip.

### 7. Schema Collision Strategy

SQLite has no schema concept. When `--include-all-schemas` is used and tables from different schemas share names:

- Default: prefix with `schema__table` and warn `SCHEMA_PREFIXED`
- The name mapping is maintained in a lookup table used to rewrite FK references and index table names

### 8. Identifier Quoting

Store identifiers with both `raw` and `normalized` forms. Quote in output only when necessary:

- Contains uppercase letters, spaces, hyphens
- Is a SQLite reserved keyword
- Starts with a digit

Use double quotes (SQLite standard).

### 9. Dependencies

**Add:**

- `sqlparser = "0.53"` — PostgreSQL dialect SQL parser (core)
- `clap = "4.5"` with derive + cargo features (cli, already present)
- `anyhow = "1.0"` (cli, already present)
- `thiserror = "2.0"` (core, already present as 1.0 — upgrade)
- `colored = "2.1"` (cli, already present)

**Remove:**

- `yaml-rust2`, `serde_yaml`, `serde`, `regex`, `indexmap` (core)
- `walkdir`, `ignore`, `is-terminal` (cli — no directory walking needed, single file input)
- `cargo-husky` (optional, keep or remove)

### 10. Testing Strategy

- **Unit tests**: In each transform module — type_map, expr_map, constraint, name_resolve, identifier quoting. Each mapping from the rule doc gets a test case.
- **Golden tests**: `tests/fixtures/*.sql` (PG DDL input) → `tests/golden/*.out.sql` (expected SQLite output). Compare rendered output string against expected file. Cover: composite PK, unique constraints, FKs with cascade, partial indexes, enums, domains, arrays, multi-schema.
- **Integration tests**: End-to-end through `convert_pg_ddl_to_sqlite()` with various `ConvertOptions` combinations.

## Risks / Trade-offs

**[sqlparser dialect gaps]** → `sqlparser` may not parse some PostgreSQL-specific DDL syntax (e.g., advanced ALTER TABLE, exotic type syntax). Mitigation: skip unrecognized statements with `PARSE_SKIPPED` warning; users can pre-process DDL.

**[Lossy conversion by design]** → Many PostgreSQL features have no SQLite equivalent (arrays, enums, domains, GIN indexes, precision enforcement). Mitigation: comprehensive warning system with codes from the rule doc; `--strict` mode fails on lossy conversions.

**[Expression compatibility]** → CHECK constraints and partial index WHERE clauses may use PostgreSQL-specific functions or operators. Mitigation: `expr_map` attempts conversion; unsupported expressions are dropped with warnings. Conservative approach — skip rather than emit invalid SQLite.

**[No roundtrip guarantee]** → Converting PG→SQLite→PG is not supported and not a goal. The output is SQLite-only.
