## 1. Project Setup & Dependencies

- [ ] 1.1 Update workspace Cargo.toml: remove yaml-rust2/serde_yaml/regex/indexmap, add sqlparser
- [ ] 1.2 Update core/Cargo.toml: set crate name to pg2sqlc-core, add sqlparser and thiserror dependencies
- [ ] 1.3 Update cli/Cargo.toml: set crate name to pg2sqlc, add clap/anyhow/colored dependencies, remove walkdir/ignore/is-terminal
- [ ] 1.4 Remove all existing yaml-lint source files from core/src/ and cli/src/
- [ ] 1.5 Create module directory structure: core/src/{pg,ir,transform,sqlite,diagnostics}/mod.rs
- [ ] 1.6 Update Makefile targets (make/test.mk, make/cli.mk) for pg2sqlc commands and SQL fixtures

## 2. IR Data Model

- [ ] 2.1 Implement core/src/ir/ident.rs: Identifier and QualifiedName types with raw/normalized forms and needs_quotes()
- [ ] 2.2 Implement core/src/ir/types.rs: PgType enum (all PG types) and SqliteType enum (INTEGER, TEXT, REAL, NUMERIC, BLOB)
- [ ] 2.3 Implement core/src/ir/expr.rs: Expr enum for literals, operators, function calls, casts, column references
- [ ] 2.4 Implement core/src/ir/model.rs: SchemaModel, Table, Column, Index, Sequence, Constraint, EnumDef, DomainDef structs
- [ ] 2.5 Wire up core/src/ir/mod.rs with public re-exports

## 3. Diagnostics System

- [ ] 3.1 Implement core/src/diagnostics/warning.rs: Warning struct, Severity enum (Info/Lossy/Unsupported/Error), warning code constants
- [ ] 3.2 Implement core/src/diagnostics/reporter.rs: output formatting (stderr/file), sorted output, strict mode enforcement
- [ ] 3.3 Wire up core/src/diagnostics/mod.rs with public re-exports

## 4. PostgreSQL DDL Parsing

- [ ] 4.1 Implement core/src/pg/parser.rs: sqlparser wrapper that parses PG DDL text into IR (CREATE TABLE, ALTER TABLE, CREATE INDEX, CREATE SEQUENCE, CREATE TYPE ENUM, CREATE DOMAIN)
- [ ] 4.2 Implement core/src/pg/normalize.rs: schema filtering (default public, custom, all-schemas) and identifier normalization (unquoted → lowercase)
- [ ] 4.3 Wire up core/src/pg/mod.rs with public re-exports
- [ ] 4.4 Add unit tests for parser: CREATE TABLE with inline constraints, ALTER TABLE ADD CONSTRAINT, CREATE INDEX, CREATE TYPE ENUM, non-DDL filtering
- [ ] 4.5 Add unit tests for normalize: schema filtering, identifier case folding, schema-qualified names

## 5. Transform — Type & Expression Mapping

- [ ] 5.1 Implement core/src/transform/type_map.rs: PgType → SqliteType mapping with warning emission for lossy conversions
- [ ] 5.2 Implement core/src/transform/expr_map.rs: default/CHECK/WHERE expression conversion (now()→CURRENT_TIMESTAMP, bool→0/1, cast stripping, nextval removal)
- [ ] 5.3 Add unit tests for type_map: all PG type categories (integer, numeric, text, boolean, date/time, uuid, json, bytea, enum, array, domain)
- [ ] 5.4 Add unit tests for expr_map: literals, now(), boolean defaults, cast removal, unsupported expression drop

## 6. Transform — Constraints, Indexes & Planning

- [ ] 6.1 Implement core/src/transform/planner.rs: merge ALTER TABLE constraints into CREATE TABLE, resolve SERIAL/IDENTITY/sequences
- [ ] 6.2 Implement core/src/transform/constraint.rs: PK (single-col INTEGER PRIMARY KEY, composite), UNIQUE, FK (with actions, enable_foreign_keys gating), CHECK transformation
- [ ] 6.3 Implement core/src/transform/index.rs: index conversion with method filtering (btree pass-through, GIN/GiST/BRIN warn), partial index WHERE, expression index handling
- [ ] 6.4 Implement core/src/transform/name_resolve.rs: schema stripping, collision detection with schema__table prefixing
- [ ] 6.5 Implement core/src/transform/topo.rs: topological sort for FK dependencies with cycle detection fallback
- [ ] 6.6 Wire up core/src/transform/mod.rs with public re-exports
- [ ] 6.7 Add unit tests for planner: ALTER merging, SERIAL resolution, IDENTITY handling
- [ ] 6.8 Add unit tests for constraint/index/topo: FK ordering, cycle handling, partial index pass-through, method warnings

## 7. SQLite DDL Rendering

- [ ] 7.1 Implement core/src/sqlite/render.rs: IR → DDL text with PRAGMA emission, CREATE TABLE (columns + constraints), CREATE INDEX, identifier quoting, deterministic ordering
- [ ] 7.2 Wire up core/src/sqlite/mod.rs
- [ ] 7.3 Add unit tests for render: single table, composite PK, FK with cascade, unique index, partial index, PRAGMA output, identifier quoting

## 8. Public API & Pipeline

- [ ] 8.1 Implement core/src/lib.rs: convert_pg_ddl_to_sqlite() orchestrating full pipeline (parse → normalize → plan → transform → order → render → report), ConvertOptions, ConvertResult, ConvertError
- [ ] 8.2 Add integration tests: end-to-end conversion with various ConvertOptions combinations (strict mode, schema filtering, FK enabled/disabled)

## 9. CLI

- [ ] 9.1 Implement cli/src/main.rs: clap derive-based CLI with --input, --output, --schema, --include-all-schemas, --enable-foreign-keys, --strict, --emit-warnings options
- [ ] 9.2 Wire CLI to core::convert_pg_ddl_to_sqlite(), handle I/O and exit codes (0 success, 1 error)

## 10. Golden Tests & Fixtures

- [ ] 10.1 Create test fixtures: tests/fixtures/ with PG DDL files covering basic table, composite PK, FKs with cascade, enums, domains, arrays, multi-schema, SERIAL/IDENTITY
- [ ] 10.2 Create golden files: tests/golden/ with expected SQLite DDL output for each fixture
- [ ] 10.3 Implement golden test runner that compares convert_pg_ddl_to_sqlite() output against golden files
- [ ] 10.4 Verify all golden tests pass and run `make ci` to confirm full CI suite passes
