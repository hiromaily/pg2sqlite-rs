## ADDED Requirements

### Requirement: Render CREATE TABLE statements
The system SHALL render CREATE TABLE statements with column definitions, inline constraints, and table-level constraints. Each statement SHALL be terminated with a semicolon.

#### Scenario: Table with columns and constraints
- **WHEN** the SQLite IR contains a table with columns, a PK, and a UNIQUE constraint
- **THEN** the output is a valid `CREATE TABLE` statement with columns listed in input order, followed by table-level constraints in order: PK, UNIQUE, CHECK, FK

### Requirement: Render CREATE INDEX statements
The system SHALL render CREATE INDEX and CREATE UNIQUE INDEX statements after all CREATE TABLE statements.

#### Scenario: Index rendering
- **WHEN** the SQLite IR contains indexes
- **THEN** each index is rendered as `CREATE [UNIQUE] INDEX "name" ON "table" ("col1", "col2");`

### Requirement: PRAGMA foreign_keys emission
When `enable_foreign_keys` is enabled, the system SHALL emit `PRAGMA foreign_keys = ON;` as the first statement in the output.

#### Scenario: Foreign keys enabled
- **WHEN** `enable_foreign_keys` is true
- **THEN** the first line of output is `PRAGMA foreign_keys = ON;`

#### Scenario: Foreign keys disabled
- **WHEN** `enable_foreign_keys` is false
- **THEN** no PRAGMA statement is emitted

### Requirement: Identifier quoting
The system SHALL quote identifiers using double quotes when they contain uppercase letters, spaces, hyphens, start with a digit, or are SQLite reserved keywords. Identifiers that are safe (lowercase, no special characters, not reserved) MAY be emitted without quotes.

#### Scenario: Reserved keyword as identifier
- **WHEN** a table is named `order`
- **THEN** it is rendered as `"order"`

#### Scenario: Simple lowercase identifier
- **WHEN** a column is named `email`
- **THEN** it MAY be rendered as `email` or `"email"` (both valid)

#### Scenario: Identifier with uppercase
- **WHEN** a column preserves quoted name `"UserName"`
- **THEN** it is rendered as `"UserName"`

### Requirement: Deterministic output ordering
The system SHALL produce deterministic output: tables in dependency order (if FK enabled) or alphabetical order, indexes sorted by table name then index name, constraints within a table sorted by type (PK → UNIQUE → CHECK → FK).

#### Scenario: Alphabetical table ordering without FK
- **WHEN** input has tables `users`, `orders`, `products` and FKs are disabled
- **THEN** output order is: `orders`, `products`, `users`

#### Scenario: Dependency ordering with FK
- **WHEN** `orders` references `users` and FKs are enabled
- **THEN** `users` is emitted before `orders`

### Requirement: Statement formatting
The system SHALL separate statements with semicolons and newlines. The output SHALL be directly executable by the `sqlite3` CLI.

#### Scenario: Multi-statement output
- **WHEN** the IR contains 2 tables and 1 index
- **THEN** the output contains 3 statements, each terminated by `;` and separated by blank lines

### Requirement: Column output preserves input order
The system SHALL preserve the original column order from the input DDL within each table.

#### Scenario: Column ordering
- **WHEN** input has columns `id`, `name`, `email` in that order
- **THEN** output preserves `id`, `name`, `email` in the same order
