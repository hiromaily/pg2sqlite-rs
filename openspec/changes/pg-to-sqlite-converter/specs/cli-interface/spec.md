## ADDED Requirements

### Requirement: Input file option
The CLI SHALL accept `--input <path>` to specify the PostgreSQL DDL input file. The file SHALL be read as UTF-8 text.

#### Scenario: Valid input file
- **WHEN** `--input schema.sql` is provided and the file exists
- **THEN** the file contents are read and passed to the converter

#### Scenario: Missing input file
- **WHEN** `--input missing.sql` is provided and the file does not exist
- **THEN** the CLI exits with a non-zero code and an error message

### Requirement: Output file option
The CLI SHALL accept `--output <path>` to specify the SQLite DDL output file. When omitted, the output SHALL be written to stdout.

#### Scenario: Output to file
- **WHEN** `--output sqlite.sql` is provided
- **THEN** the converted DDL is written to `sqlite.sql`

#### Scenario: Output to stdout
- **WHEN** `--output` is omitted
- **THEN** the converted DDL is written to stdout

### Requirement: Schema filter option
The CLI SHALL accept `--schema <name>` to filter objects by schema (default: `public`).

#### Scenario: Custom schema
- **WHEN** `--schema analytics` is provided
- **THEN** only objects in the `analytics` schema are converted

#### Scenario: Default schema
- **WHEN** `--schema` is omitted
- **THEN** only objects in the `public` schema are converted

### Requirement: Include all schemas option
The CLI SHALL accept `--include-all-schemas` to bypass schema filtering and convert all objects.

#### Scenario: All schemas included
- **WHEN** `--include-all-schemas` is provided
- **THEN** objects from all schemas are converted

### Requirement: Enable foreign keys option
The CLI SHALL accept `--enable-foreign-keys` to emit `PRAGMA foreign_keys = ON;` and include FK constraints in the output.

#### Scenario: Foreign keys enabled
- **WHEN** `--enable-foreign-keys` is provided
- **THEN** FK constraints are included and PRAGMA is emitted

### Requirement: Strict mode option
The CLI SHALL accept `--strict` to fail on lossy conversions instead of emitting warnings.

#### Scenario: Strict mode failure
- **WHEN** `--strict` is provided and lossy conversions are detected
- **THEN** the CLI exits with a non-zero code and reports the violations

### Requirement: Emit warnings option
The CLI SHALL accept `--emit-warnings <path|stderr>` to control where warnings are written (default: stderr).

#### Scenario: Warnings to file
- **WHEN** `--emit-warnings /tmp/warn.txt` is provided
- **THEN** warnings are written to `/tmp/warn.txt`

### Requirement: Exit codes
The CLI SHALL exit with code 0 on success, code 1 on errors (parse failures, strict violations, I/O errors).

#### Scenario: Successful conversion
- **WHEN** conversion completes without errors
- **THEN** exit code is 0

#### Scenario: Conversion error
- **WHEN** conversion fails due to parse error or strict violation
- **THEN** exit code is 1

### Requirement: Library-first architecture
The CLI SHALL be a thin wrapper around the `core` library's `convert_pg_ddl_to_sqlite()` function. All conversion logic SHALL reside in the `core` crate.

#### Scenario: CLI delegates to core
- **WHEN** the CLI is invoked
- **THEN** it constructs `ConvertOptions` from CLI args, calls `convert_pg_ddl_to_sqlite()`, and writes the result
