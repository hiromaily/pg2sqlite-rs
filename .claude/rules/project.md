# Project Rules

## Overview
- **Purpose**: PostgreSQL 16 DDL → SQLite3 DDL schema converter
- **Language**: Rust (edition 2024)
- **No database connections** — offline text-to-text conversion

## Architecture
- `core/` = library crate (`pg2sqlc-core`, reusable)
- `cli/` = binary crate (`pg2sqlc`, thin wrapper)
- Pipeline: Parse → Normalize → Plan → Transform → Order → Render → Report

### Module Layout
- `core/src/pg/` — PostgreSQL DDL parsing (sqlparser wrapper)
- `core/src/ir/` — Internal representation (SchemaModel, types, expressions, identifiers)
- `core/src/transform/` — PG IR → SQLite IR (type mapping, constraints, indexes, topo sort)
- `core/src/sqlite/` — SQLite DDL rendering
- `core/src/diagnostics/` — Warning/error system

## Commands (Makefile)

### CI Commands (run before commit)
```bash
make ci          # Run all CI checks (fmt-check + lint + test)
make fmt-check   # Check formatting
make lint        # Run clippy
make test        # Run all tests
```

### Development
```bash
make build       # Build debug
make release     # Build release
make fmt         # Format code
make watch       # Watch and test
make clean       # Clean build artifacts
```

### Testing
```bash
make test-verbose           # Tests with output
make test-one TEST=name     # Run specific test
make validate-fixtures      # Validate test fixtures
```

## Dependencies
- `sqlparser` for PostgreSQL DDL parsing
- `clap` for CLI argument parsing (cli)
- `thiserror` for error types (core)
- `anyhow` for CLI error handling (cli)
- `colored` for terminal colors (cli)

## Specifications
- `docs/postgres-to-sqlite.md` — Base spec (pipeline, IR, CLI, conversion rules)
- `docs/postgres-to-sqlite-rule.md` — Detailed mapping tables (types, constraints, defaults, indexes)
- `docs/development-by-rust.md` — Rust module design
