# pg2sqlite-rs

A PostgreSQL 16 DDL to SQLite3 DDL schema converter written in Rust.

Built with [Claude Code](https://www.anthropic.com/claude-code).

## Why pg2sqlite-rs?

- 🔄 **Offline conversion** — no database connections required, pure text-to-text
- 📦 **Single binary** — no runtime dependencies
- 🎯 **Accurate mapping** — comprehensive type, constraint, and expression conversion
- ⚠️ **Lossy conversion warnings** — clearly reports what information is lost
- 🦀 **Memory safe** — written in Rust

## Features

- ✅ PostgreSQL 16 DDL parsing via [sqlparser](https://github.com/sqlparser-rs/sqlparser-rs)
- ✅ Comprehensive type mapping (INTEGER, TEXT, REAL, NUMERIC, BLOB)
- ✅ Constraint conversion (PK, UNIQUE, FK, CHECK)
- ✅ SERIAL/BIGSERIAL → INTEGER PRIMARY KEY AUTOINCREMENT
- ✅ ALTER TABLE constraint merging into CREATE TABLE
- ✅ Foreign key support with `PRAGMA foreign_keys`
- ✅ Topological sort for FK dependency ordering
- ✅ Schema filtering (`public`, custom, or all schemas)
- ✅ Strict mode — fail on lossy conversions
- ✅ Golden test suite for regression testing

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap hiromaily/tap
brew install pg2sqlite
```

### From Source

Requires Rust 1.93+ (install from [rustup.rs](https://rustup.rs/)):

```bash
git clone https://github.com/hiromaily/pg2sqlite-rs.git
cd pg2sqlite-rs
make install
# The binary will be installed as `pg2sqlite`
```

### Build Only

```bash
make build    # Debug build
make release  # Release build
```

## Usage

### Basic usage

```bash
# Convert and print to stdout
pg2sqlite -i schema.sql

# Convert and write to file
pg2sqlite -i schema.sql -o sqlite_schema.sql

# Filter by schema
pg2sqlite -i schema.sql -s myschema

# Include all schemas
pg2sqlite -i schema.sql --include-all-schemas
```

### Foreign key support

```bash
# Enable PRAGMA foreign_keys and include FK constraints
pg2sqlite -i schema.sql --enable-foreign-keys
```

### Strict mode

```bash
# Fail on lossy conversions instead of emitting warnings
pg2sqlite -i schema.sql --strict
```

### Warning output

```bash
# Emit warnings to stderr
pg2sqlite -i schema.sql --emit-warnings stderr

# Emit warnings to a file
pg2sqlite -i schema.sql --emit-warnings warnings.log
```

### Options

```text
-i, --input <PATH>              PostgreSQL DDL input file
-o, --output <PATH>             SQLite DDL output file (default: stdout)
-s, --schema <NAME>             Filter by schema (default: "public")
    --include-all-schemas       Include all schemas
    --enable-foreign-keys       Emit PRAGMA and FK constraints
    --strict                    Fail on lossy conversions
    --emit-warnings <PATH>      Warning destination (file path or "stderr")
-h, --help                      Print help
-V, --version                   Print version
```

## Conversion Examples

### Basic table

```sql
-- Input (PostgreSQL)
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email VARCHAR(255) UNIQUE,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT now()
);

-- Output (SQLite)
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT UNIQUE,
  active INTEGER DEFAULT 1,
  created_at TEXT DEFAULT (CURRENT_TIMESTAMP)
);
```

### SERIAL types

```sql
-- Input (PostgreSQL)
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    total NUMERIC(10, 2) NOT NULL
);

-- Output (SQLite)
CREATE TABLE orders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  total NUMERIC NOT NULL
);
```

### Foreign keys

```sql
-- Input (PostgreSQL)
CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE
);

-- Output (SQLite) with --enable-foreign-keys
PRAGMA foreign_keys = ON;

CREATE TABLE users (
  ...
);

CREATE TABLE posts (
  id INTEGER PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) ON DELETE CASCADE
);
```

Tables are topologically sorted so that referenced tables appear before referencing tables.

## Type Mapping

| PostgreSQL | SQLite | Notes |
| --- | --- | --- |
| `smallint`, `integer`, `bigint` | `INTEGER` | Width ignored |
| `serial`, `bigserial` | `INTEGER` (AUTOINCREMENT) | Sequence resolved |
| `boolean` | `INTEGER` | 1/0 convention |
| `real` | `REAL` | |
| `double precision` | `REAL` | |
| `numeric`, `decimal` | `NUMERIC` | Precision lost |
| `text` | `TEXT` | |
| `varchar(n)`, `char(n)` | `TEXT` | Length ignored |
| `date`, `timestamp`, `time` | `TEXT` | ISO 8601 storage |
| `uuid` | `TEXT` | |
| `json`, `jsonb` | `TEXT` | JSONB features lost |
| `bytea` | `BLOB` | |
| `enum` types | `TEXT` | Enum values lost |
| `array` types | `TEXT` | Array semantics lost |

## Default Expression Mapping

| PostgreSQL | SQLite |
| --- | --- |
| `true` / `false` | `1` / `0` |
| `now()` / `CURRENT_TIMESTAMP` | `(CURRENT_TIMESTAMP)` |
| `CURRENT_DATE` | `(CURRENT_DATE)` |
| `CURRENT_TIME` | `(CURRENT_TIME)` |

## Warning Codes

pg2sqlite emits warnings when conversion is lossy:

| Code | Description |
| --- | --- |
| `TYPE_WIDTH_IGNORED` | Integer width information dropped |
| `VARCHAR_LENGTH_IGNORED` | VARCHAR length constraint dropped |
| `NUMERIC_PRECISION_LOSS` | Numeric precision/scale dropped |
| `BOOLEAN_AS_INTEGER` | Boolean converted to INTEGER |
| `DATETIME_TEXT_STORAGE` | Date/time stored as TEXT |
| `TIMEZONE_LOSS` | Timezone information dropped |
| `UUID_AS_TEXT` | UUID stored as TEXT |
| `JSON_AS_TEXT` | JSON stored as TEXT |
| `JSONB_LOSS` | JSONB features lost |
| `ENUM_AS_TEXT` | Enum stored as TEXT |
| `ARRAY_LOSSY` | Array stored as TEXT |
| `SERIAL_TO_ROWID` | SERIAL mapped to AUTOINCREMENT |

## Architecture

```text
pg2sqlite-rs/
├── core/                 # Library crate (pg2sqlite-core)
│   └── src/
│       ├── lib.rs        # Public API: convert_pg_ddl_to_sqlite()
│       ├── pg/           # PostgreSQL DDL parsing
│       ├── ir/           # Internal representation
│       ├── transform/    # PG IR → SQLite IR transformation
│       ├── sqlite/       # SQLite DDL rendering
│       └── diagnostics/  # Warning/error system
├── cli/                  # Binary crate (pg2sqlite)
│   └── src/main.rs       # CLI entry point
├── tests/                # Integration & golden tests
│   ├── fixtures/         # PostgreSQL DDL input files
│   └── golden/           # Expected SQLite DDL output files
└── docs/                 # Specifications
```

### Pipeline

```text
PostgreSQL DDL
  → Parse (sqlparser)
  → Normalize (schema filter, identifiers)
  → Plan (merge ALTERs, resolve SERIAL/sequences)
  → Transform (types, expressions, constraints, indexes)
  → Order (topological sort for FK dependencies)
  → Render (SQLite DDL text)
  → Report (warnings)
```

## Library Usage

`pg2sqlite-core` can be used as a library:

```rust
use pg2sqlite_core::{convert_pg_ddl_to_sqlite, ConvertOptions};

let pg_ddl = r#"
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL
    );
"#;

let opts = ConvertOptions::default();
let result = convert_pg_ddl_to_sqlite(pg_ddl, &opts).unwrap();

println!("{}", result.sqlite_sql);
for w in &result.warnings {
    eprintln!("warning: {}", w);
}
```

## Development

### Commands

```bash
make ci              # Run all CI checks (fmt-check + lint + test)
make fmt             # Format code
make lint            # Run clippy
make test            # Run all tests
make test-verbose    # Tests with output
make test-one TEST=name  # Run specific test
make watch           # Watch and test on change
```

### Adding Golden Tests

1. Add a PostgreSQL DDL file to `tests/fixtures/`
2. Add the expected SQLite output to `tests/golden/`
3. Run `make test` to verify

## Exit Codes

- **0**: Success
- **1**: Conversion error or strict mode violation

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT

## Acknowledgments

- [sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs) — PostgreSQL SQL parser
