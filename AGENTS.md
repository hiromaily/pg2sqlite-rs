# Agent Instructions

This document provides guidance for AI coding assistants working on pg2sqlc-rs.

## Project Overview

- **Purpose**: PostgreSQL 16 DDL to SQLite3 DDL schema converter
- **Language**: Rust (edition 2024)
- **Structure**: Cargo workspace with `core` (library) and `cli` (binary) crates

## Architecture

```text
pg2sqlc-rs/
├── core/                 # Library crate (pg2sqlc-core)
│   └── src/
│       ├── lib.rs        # Public API: convert_pg_ddl_to_sqlite()
│       ├── pg/           # PostgreSQL DDL parsing (sqlparser wrapper)
│       │   ├── parser.rs # Parse PG DDL → IR
│       │   └── normalize.rs # Schema filtering, identifier normalization
│       ├── ir/           # Internal representation
│       │   ├── model.rs  # SchemaModel, Table, Column, Index, Sequence
│       │   ├── expr.rs   # Expr enum (literals, operators, functions, casts)
│       │   ├── types.rs  # PgType, SqliteType enums
│       │   └── ident.rs  # Identifier, QualifiedName, quoting
│       ├── transform/    # PG IR → SQLite IR transformation
│       │   ├── planner.rs    # Merge ALTERs, resolve SERIAL/sequences
│       │   ├── type_map.rs   # PgType → SqliteType mapping
│       │   ├── expr_map.rs   # Default/CHECK/WHERE expression conversion
│       │   ├── constraint.rs # PK/UNIQUE/FK/CHECK transformation
│       │   ├── index.rs      # Index conversion
│       │   ├── name_resolve.rs # Schema stripping, collision handling
│       │   └── topo.rs       # Topological sort for FK dependencies
│       ├── sqlite/       # SQLite DDL rendering
│       │   └── render.rs # IR → DDL text output
│       └── diagnostics/  # Warning/error system
│           ├── warning.rs    # Warning struct, Severity enum, codes
│           └── reporter.rs   # Output formatting, strict mode
├── cli/                  # Binary crate (pg2sqlc)
│   └── src/main.rs       # CLI entry point (thin wrapper)
├── tests/                # Integration & golden tests
│   ├── fixtures/         # PostgreSQL DDL input files
│   └── golden/           # Expected SQLite DDL output files
└── docs/                 # Specifications
    ├── postgres-to-sqlite.md      # Base specification
    ├── postgres-to-sqlite-rule.md # Detailed mapping tables
    └── development-by-rust.md     # Rust module design
```

## Key Commands (Makefile)

### CI Commands (run before commit)

```bash
make ci          # Run all CI checks (fmt-check + lint + test)
make fmt-check   # Check formatting
make lint        # Run clippy
make test        # Run all tests
```

### Development

```bash
make build       # Build debug version
make release     # Build release version
make fmt         # Format code
make clean       # Clean build artifacts
make help        # Show all available targets
```

## Key Public API

```rust
// core/src/lib.rs
pub fn convert_pg_ddl_to_sqlite(input: &str, opts: &ConvertOptions) -> Result<ConvertResult, ConvertError>;

pub struct ConvertOptions {
    pub schema: Option<String>,          // default: "public"
    pub include_all_schemas: bool,
    pub enable_foreign_keys: bool,
    pub strict: bool,
    pub emit_warnings: Option<PathBuf>,
}

pub struct ConvertResult {
    pub sqlite_sql: String,
    pub warnings: Vec<Warning>,
}
```

## Pipeline Flow

1. **Parse** (`pg::parser`) — sqlparser AST → PG IR
2. **Normalize** (`pg::normalize`) — schema filtering, identifier normalization
3. **Plan** (`transform::planner`) — merge ALTERs, resolve SERIAL/sequences
4. **Transform** (`transform::*`) — type mapping, expression conversion, constraints, indexes
5. **Order** (`transform::topo`) — topological sort for FK dependencies
6. **Render** (`sqlite::render`) — SQLite IR → DDL text
7. **Report** (`diagnostics::reporter`) — emit warnings

## Code Style

- Follow Rust idioms and clippy suggestions
- Use `thiserror` for error types in `core`
- Use `anyhow` for error handling in `cli`
- Keep transform passes as pure functions (input IR + options → output IR + warnings)
- Add doc comments for public APIs

## Dependencies

- `sqlparser` — PostgreSQL dialect SQL parser (core)
- `clap` — CLI argument parsing with derive (cli)
- `thiserror` — Structured error types (core)
- `anyhow` — CLI error handling (cli)
- `colored` — Terminal color output (cli)

## Documentation

- `docs/postgres-to-sqlite.md` — Base specification (pipeline, IR, CLI, conversion rules)
- `docs/postgres-to-sqlite-rule.md` — Detailed type/constraint/default/index mapping tables
- `docs/development-by-rust.md` — Rust module design proposal
