# Specification (Base) — PostgreSQL 16 Schema to SQLite3 Schema Converter

## 1. Purpose

Build a program that converts a PostgreSQL 16 schema definition into an equivalent SQLite3 schema definition.

The converter focuses on schema-level objects primarily expressed via DDL (e.g., `CREATE TABLE`, `CREATE INDEX`, `ALTER TABLE ... ADD CONSTRAINT`, etc.). The output should be valid SQLite DDL suitable for initializing a database.

## 2. Goals & Non-Goals

### 2.1 Goals

* Parse PostgreSQL schema DDL and emit SQLite-compatible DDL.
* Preserve table structures, columns, primary keys, unique constraints, indexes, and (when feasible) foreign keys.
* Provide deterministic output ordering for stable diffs.
* Produce warnings for lossy conversions.

### 2.2 Non-Goals (initial version)

* Data migration (COPY/INSERT) is out of scope.
* View/function/trigger conversion is out of scope (can be extended later).
* Full semantic equivalence is not guaranteed (e.g., partial indexes, expression indexes, advanced constraints).
* Not required to connect to a live Postgres database (input is DDL text). (Optional extension: `pg_dump --schema-only` compatibility.)

## 3. Inputs & Outputs

### 3.1 Input

* A text file containing PostgreSQL 16 DDL.
* Encoding: UTF-8.
* The input may contain:

  * `CREATE TABLE ...`
  * `ALTER TABLE ... ADD CONSTRAINT ...`
  * `CREATE INDEX ...`
  * `CREATE SEQUENCE ...` (often used for `SERIAL` legacy)
  * `COMMENT ON ...` (ignore)
  * `SET ...`, `SELECT pg_catalog.set_config...` (ignore)
* Multiple schemas may exist; default behavior is to convert objects within a target schema (default: `public`) or all schemas depending on config.

### 3.2 Output

* SQLite3 DDL text in UTF-8.
* Statements separated by semicolons and newlines.
* Output should be runnable by `sqlite3` CLI without modifications.

## 4. CLI / Configuration

Provide a CLI with at least:

* `--input <path>`: PostgreSQL DDL file
* `--output <path>`: SQLite DDL file (or stdout if omitted)
* `--schema <name>`: convert only objects in this schema (default: `public`)
* `--include-all-schemas`: ignore schema filter and convert everything
* `--enable-foreign-keys`: include `PRAGMA foreign_keys = ON;` and emit FK constraints where possible
* `--strict`: if enabled, fail on unsupported features instead of emitting warnings
* `--emit-warnings <path|stderr>`: where to write warnings

## 5. High-Level Pipeline

1. **Lex/Parse** PostgreSQL DDL into an AST-like internal representation (IR).
2. **Normalize** identifiers:

   * Strip schema qualification depending on mode.
   * Preserve quoting rules.
3. **Transform** Postgres IR → SQLite IR:

   * Map types, defaults, constraints, indexes.
4. **Render** SQLite IR → SQLite DDL text.
5. Emit warnings/errors and diagnostics.

## 6. Internal Representation (IR)

Define IR structs (language-agnostic; implement in Rust):

* `SchemaModel`

  * `tables: Vec<Table>`
  * `indexes: Vec<Index>`
  * `sequences: Vec<Sequence>` (optional)
  * `warnings: Vec<Warning>`

* `Table`

  * `name: QualifiedName` (schema + name)
  * `columns: Vec<Column>`
  * `table_constraints: Vec<TableConstraint>` (PK, UNIQUE, CHECK, FK)
  * `options` (ignored by SQLite)

* `Column`

  * `name: String`
  * `pg_type: PgType`
  * `nullable: bool`
  * `default: Option<Expr>`
  * `constraints: Vec<ColumnConstraint>` (inline PK/UNIQUE/CHECK/REFERENCES)

* `Index`

  * `name: QualifiedName`
  * `table: QualifiedName`
  * `columns_or_expr: Vec<IndexKey>` (column names or expressions)
  * `unique: bool`
  * `where_clause: Option<Expr>` (partial index)
  * `method: Option<String>` (btree, gin, gist, etc.)

* `Warning`

  * `code: String` (e.g., `TYPE_LOSSY`, `UNSUPPORTED_PARTIAL_INDEX`)
  * `message: String`
  * `location: Option<Span>` (line/col)

## 7. Identifier & Quoting Rules

* SQLite supports quoted identifiers using double quotes.
* Strategy:

  * Preserve original case if quoted; otherwise fold to lower-case (Postgres behavior) OR preserve raw input tokens—choose deterministic rule.
  * Recommended: store both `raw` and `normalized` forms; render with quotes only when needed:

    * contains uppercase, spaces, hyphens, reserved keywords, starts with digit.
* Strip schema prefix in output (SQLite has no schemas). If multiple schemas are included, namespace collisions must be handled:

  * default strategy: prefix with `schema__table` and warn.

## 8. DDL Conversion Rules

### 8.1 CREATE TABLE

* Convert `CREATE TABLE schema.table (...)` → `CREATE TABLE "table" (...)`.
* Inline constraints preferred where possible.

#### Column attributes

* `NOT NULL`: preserve
* `NULL`: ignore (SQLite default)
* `DEFAULT`: convert if supported (see 8.4)
* `GENERATED ... AS IDENTITY`: convert to `INTEGER PRIMARY KEY` if it is the sole PK and type is integer-compatible; otherwise warn.
* `SERIAL`/`BIGSERIAL`: map to integer with auto-increment semantics:

  * Prefer `INTEGER PRIMARY KEY` (rowid alias) when feasible.
  * Avoid `AUTOINCREMENT` unless explicitly configured (SQLite recommendation: AUTOINCREMENT has tradeoffs).

### 8.2 Primary Key

* Postgres supports composite PK. SQLite supports composite PK via table constraint:

  * `PRIMARY KEY (a,b)` is allowed.
* If a table has a single-column integer PK and it is declared as `INTEGER PRIMARY KEY`, it becomes rowid alias.

  * If original column is `BIGINT`, it can still be `INTEGER` in SQLite (SQLite uses dynamic typing), but warn if strict mapping is desired.

### 8.3 UNIQUE Constraints

* Convert table-level and column-level UNIQUE constraints:

  * `UNIQUE (a,b)` supported.
* If constraint name exists, SQLite ignores names for constraints; you may preserve via comments (optional) or drop silently.

### 8.4 DEFAULT Expressions

Supported mappings:

* `DEFAULT <numeric literal>` → same
* `DEFAULT '<text>'` → same
* `DEFAULT true/false` → `DEFAULT 1/0` (warn `BOOLEAN_AS_INTEGER`)
* `DEFAULT now()` / `CURRENT_TIMESTAMP`:

  * Map to `DEFAULT (CURRENT_TIMESTAMP)` if semantics acceptable.
* `DEFAULT uuid_generate_v4()` or other functions:

  * Not supported natively; either:

    * Drop default and warn, or
    * Convert to `DEFAULT (lower(hex(randomblob(16))))` (not a real UUID format) — only if explicit flag `--sqlite-uuid-emulation`.
* `DEFAULT nextval('seq')`:

  * Typically from SERIAL; handle via integer PK mapping. If not resolvable, drop and warn.

Unsupported defaults:

* Complex expressions, casts, schema-qualified functions, subqueries → drop + warn or error in `--strict`.

### 8.5 Type Mapping

SQLite has type affinity; we still map to conventional type names.

Baseline mapping table:

* Integer-like:

  * `smallint`, `integer`, `int`, `int4`, `bigint`, `int8` → `INTEGER`
* Numeric/decimal:

  * `numeric(p,s)`, `decimal(p,s)` → `NUMERIC` (warn that precision enforcement differs)
  * `real`, `float4`, `float8`, `double precision` → `REAL`
* Text:

  * `text`, `varchar(n)`, `character varying(n)`, `char(n)`, `character(n)` → `TEXT` (optionally warn that length not enforced)
* Boolean:

  * `boolean` → `INTEGER` with convention 0/1
* Date/time:

  * `date`, `timestamp`, `timestamptz`, `time`, `timetz` → `TEXT` (ISO8601) by default
  * Optional mode: map to `NUMERIC` (Unix epoch) if configured; default to TEXT.
* UUID:

  * `uuid` → `TEXT`
* JSON:

  * `json`, `jsonb` → `TEXT` (warn; SQLite JSON1 extension might be available but not assumed)
* Binary:

  * `bytea` → `BLOB`
* Enum types:

  * `CREATE TYPE ... AS ENUM (...)`:

    * Convert column type to `TEXT` and optionally add `CHECK (col IN (...))` if `--emulate-enum-check` enabled.
* Arrays:

  * `type[]` → `TEXT` (e.g., JSON array) with warning `ARRAY_LOSSY`
* Domains:

  * Flatten to base type; merge NOT NULL/default/check if possible; otherwise warn.

### 8.6 Foreign Keys

SQLite supports FK constraints but requires `PRAGMA foreign_keys=ON`.

* Convert `REFERENCES other_table(col)` constraints.
* Ensure referenced table and columns exist in output; otherwise warn.
* Support `ON DELETE` / `ON UPDATE` actions where they match SQLite capabilities (CASCADE, SET NULL, RESTRICT, NO ACTION).
* DEFERRABLE constraints: SQLite supports DEFERRABLE but semantics differ; either drop and warn or emit limited form.

### 8.7 CHECK Constraints

SQLite supports CHECK.

* Convert simple CHECK expressions if they use SQLite-compatible syntax.
* Postgres-specific operators/functions/casts may be unsupported:

  * If not translatable, drop and warn (or error in strict mode).

### 8.8 Indexes

* Convert `CREATE INDEX` and `CREATE UNIQUE INDEX` to SQLite equivalents when possible.
* Index methods (GIN/GiST/BRIN/HASH) are not supported; ignore method and warn.
* Expression indexes:

  * SQLite supports expression indexes (modern SQLite versions) but function compatibility may differ.
  * If expression contains unsupported functions/casts → warn and skip.
* Partial indexes (`WHERE ...`):

  * SQLite supports partial indexes, but expression compatibility may differ.
  * If where clause unsupported → warn and skip.

### 8.9 ALTER TABLE

* Postgres uses `ALTER TABLE ... ADD CONSTRAINT ...` frequently.
* Strategy:

  * Collect constraints and merge into the owning `CREATE TABLE` when possible.
  * SQLite has limited ALTER TABLE; cannot add most constraints after creation.
  * Therefore: **emit consolidated CREATE TABLE** with all constraints that can be represented.
  * If input relies on late constraints that cannot be merged, warn and skip.

### 8.10 Sequences

* SQLite has no sequences.
* Sequences referenced only by SERIAL/IDENTITY should be absorbed into integer PK behavior.
* Standalone sequences: ignore with warning unless `--emulate-sequence-table` is enabled (optional extension) to generate a sequence table.

## 9. Output Ordering

Deterministic ordering recommended:

1. `PRAGMA foreign_keys = ON;` (if enabled)
2. `CREATE TABLE ...` in dependency order if FKs enabled (topological sort), otherwise alphabetical.
3. `CREATE INDEX ...` sorted by table then index name.

When cycles exist in FK graph, fall back to alphabetical and warn.

## 10. Diagnostics

* Always produce a warnings report unless `--strict`.
* Include:

  * warning code
  * object (table/column/index)
  * short message
  * optional source location (line/col)
* In `--strict`, treat any warning with severity >= “lossy” as error.

## 11. Test Plan

### 11.1 Unit Tests

* Type mapping: each Postgres type → expected SQLite type.
* Default mapping: literals, booleans, now(), nextval().
* Identifier quoting: keywords, uppercase, spaces.

### 11.2 Golden Tests (DDL snapshots)

* Feed sample Postgres DDL files and compare SQLite output with expected `.sql`.
* Include cases:

  * composite PK
  * unique constraints
  * FKs with cascade
  * partial indexes
  * enums/domains
  * array columns

### 11.3 Property Tests (optional)

* Parser robustness: random whitespace/comments, ordering changes.

## 12. Compatibility Notes

* Assume SQLite version is “modern enough” to support partial and expression indexes only if explicitly configured; otherwise skip those features.
* SQLite type affinity differs from Postgres; output preserves intent but not strict enforcement.

## 13. Security & Safety

* Input is untrusted text; parser must avoid panics and handle large files gracefully.
* Limit recursion depth or use iterative parsing to avoid stack overflow on adversarial inputs.

## 14. Implementation Notes for Rust

* Prefer a proper SQL parser for Postgres dialect to build the IR.
* Avoid relying on SQLite engine to parse Postgres SQL directly.
* Keep rendering and transformation separate from parsing.

