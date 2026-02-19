## Why

There is no lightweight, offline tool to convert PostgreSQL 16 DDL schemas into SQLite3-compatible DDL. Teams that use PostgreSQL in production but SQLite for local development, testing, or embedded use need a reliable schema converter. Building this as a Rust CLI with a reusable library core enables fast, deterministic conversion with clear diagnostics for lossy mappings.

## Preconditions

This project starts from a **copied pg2sqlite-rs codebase** (`core/` + `cli/` workspace). Several patterns are directly reusable:

- **Workspace structure**: `core/` (library) + `cli/` (thin binary) — keep as-is
- **CLI patterns**: `clap` derive-based argument parsing, `ColorMode`, `configure_colors()`, exit code strategy (0/1/2), `anyhow::Context` error wrapping — adapt for pg2sqlc flags
- **Error handling**: `thiserror`-based error enum with `#[from]` variants, `type Result<T>` alias — reuse pattern
- **Diagnostics/problem model**: `LintProblem { line, column, message, rule, level }` with `Ord` sorting — becomes `Warning { code, severity, message, object, span }`
- **Output formatters**: `OutputFormatter` trait with Standard/Colored/Parsable variants — reuse for diagnostic output
- **Makefile infrastructure**: `make/dev.mk` (build/lint/fmt/ci), `make/release.mk` (versioning/tagging), `make/test.mk` (golden test pattern) — adapt targets

Detailed specifications already exist in `docs/`:

- `docs/postgres-to-sqlite.md` — full base specification (pipeline, IR design, CLI flags, conversion rules, test plan)
- `docs/postgres-to-sqlite-rule.md` — detailed type/constraint/default/index mapping tables with warning codes
- `docs/development-by-rust.md` — Rust module design proposal (crate layout, data flow, public APIs, IR structs, testing strategy)

## What Changes

- **New CLI tool** (`pg2sqlc`) that reads PostgreSQL DDL text and outputs SQLite3 DDL
- **PostgreSQL DDL parser** that extracts CREATE TABLE, ALTER TABLE ADD CONSTRAINT, CREATE INDEX, CREATE SEQUENCE, CREATE TYPE (enum), and CREATE DOMAIN statements into an internal representation
- **Type mapping engine** covering all PostgreSQL types → SQLite type affinities (INTEGER, TEXT, REAL, NUMERIC, BLOB) with ~30 warning codes for lossy conversions
- **Constraint transformation** that merges ALTER TABLE constraints into consolidated CREATE TABLE output, maps PK/UNIQUE/FK/CHECK constraints, and handles SERIAL/IDENTITY → INTEGER PRIMARY KEY
- **Default expression converter** for literals, booleans (→ 0/1), now() (→ CURRENT_TIMESTAMP), with drop-and-warn for unsupported functions
- **Index converter** supporting standard, unique, partial, and expression indexes with method-agnostic output (GIN/GiST/BRIN → warn and skip method)
- **Schema handling** with configurable schema filtering (default: `public`), schema stripping for SQLite output, and collision strategy for multi-schema input
- **Dependency-ordered output** using topological sort for FK relationships, with cycle detection fallback to alphabetical
- **Diagnostics system** with severity levels (Info, Lossy, Unsupported, Error), strict mode, and configurable warning output (stderr or file)
- **Replace pg2sqlite source**: All pg2sqlite-specific source files replaced; reusable infrastructure patterns (CLI, error handling, output formatting, Makefile) adapted in place

## Capabilities

### New Capabilities

- `pg-ddl-parsing`: Parse PostgreSQL 16 DDL text into an internal representation (IR) — handles CREATE TABLE, ALTER TABLE, CREATE INDEX, CREATE SEQUENCE, CREATE TYPE ENUM, CREATE DOMAIN, with schema filtering and identifier normalization
- `type-and-expr-mapping`: Map PostgreSQL types to SQLite type affinities, convert default expressions and CHECK constraints, handle SERIAL/IDENTITY to INTEGER PRIMARY KEY, enum-to-TEXT with optional CHECK emulation
- `constraint-and-index-transform`: Merge ALTER TABLE constraints into CREATE TABLE, transform PK/UNIQUE/FK/CHECK constraints, convert indexes with partial/expression support, topological ordering for FK dependencies
- `sqlite-rendering`: Render SQLite-ready IR into valid DDL text with deterministic ordering, proper identifier quoting, PRAGMA emission, and statement formatting
- `diagnostics`: Warning and error reporting system with severity levels, warning codes, strict mode enforcement, and configurable output destinations
- `cli-interface`: Command-line interface with --input, --output, --schema, --include-all-schemas, --enable-foreign-keys, --strict, --emit-warnings options

### Modified Capabilities

_(none — no existing specs)_

## Impact

- **Crate structure**: Workspace stays as `core/` (library) + `cli/` (binary); source files replaced but structural patterns preserved
- **Dependencies added**: `sqlparser` (PostgreSQL dialect SQL parser) — the main new dependency; keep `clap`, `colored`, `anyhow`, `thiserror`
- **Dependencies removed**: `yaml-rust2`, `serde_yaml`, `regex`, `indexmap`, `walkdir`, `ignore` — no longer needed
- **Cargo.toml**: Workspace package metadata updated (name → pg2sqlc, repository, description); crate names updated
- **Tests**: New test infrastructure with unit tests for type/expr mapping, golden tests (DDL snapshot comparison with `tests/fixtures/*.sql` → `tests/golden/*.out.sql`)
- **Makefile**: `make/dev.mk` and `make/release.mk` reused as-is; `make/test.mk` and `make/cli.mk` adapted for pg2sqlc commands and SQL fixtures
