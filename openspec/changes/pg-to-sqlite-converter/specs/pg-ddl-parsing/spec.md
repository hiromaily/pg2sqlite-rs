## ADDED Requirements

### Requirement: Parse CREATE TABLE statements
The system SHALL parse PostgreSQL `CREATE TABLE` statements including column definitions, inline column constraints (NOT NULL, DEFAULT, PRIMARY KEY, UNIQUE, REFERENCES, CHECK), and table-level constraints (PRIMARY KEY, UNIQUE, FOREIGN KEY, CHECK). The parser SHALL handle schema-qualified table names (`schema.table`).

#### Scenario: Simple table with columns
- **WHEN** input contains `CREATE TABLE public.users (id integer NOT NULL, name text, email varchar(255))`
- **THEN** the parser produces a Table IR with name `public.users` and 3 columns with correct types and nullable flags

#### Scenario: Table with inline and table-level constraints
- **WHEN** input contains a CREATE TABLE with `id serial PRIMARY KEY`, `email text UNIQUE`, and `CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES orgs(id)`
- **THEN** the parser captures the inline PK on `id`, inline UNIQUE on `email`, and the table-level FK constraint with name `fk_org`

#### Scenario: Schema-qualified table name
- **WHEN** input contains `CREATE TABLE myschema.orders (...)`
- **THEN** the parser stores schema=`myschema` and name=`orders` in the QualifiedName

### Requirement: Parse ALTER TABLE ADD CONSTRAINT statements
The system SHALL parse `ALTER TABLE ... ADD CONSTRAINT` statements for PRIMARY KEY, UNIQUE, FOREIGN KEY, and CHECK constraints and associate them with the target table.

#### Scenario: ALTER TABLE adds a foreign key
- **WHEN** input contains `ALTER TABLE public.orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE`
- **THEN** the parser produces a FK constraint with columns=[user_id], references=public.users(id), on_delete=CASCADE, associated with table `public.orders`

#### Scenario: ALTER TABLE target table not in input
- **WHEN** input contains an ALTER TABLE for a table that has no corresponding CREATE TABLE
- **THEN** the parser emits a warning with code `ALTER_TARGET_MISSING` and skips the constraint

### Requirement: Parse CREATE INDEX statements
The system SHALL parse `CREATE INDEX` and `CREATE UNIQUE INDEX` statements, capturing the index name, target table, column list or expressions, uniqueness, optional WHERE clause, and optional USING method.

#### Scenario: Simple index
- **WHEN** input contains `CREATE INDEX idx_users_email ON public.users (email)`
- **THEN** the parser produces an Index IR with name=`idx_users_email`, table=`public.users`, columns=[email], unique=false

#### Scenario: Partial index with WHERE clause
- **WHEN** input contains `CREATE INDEX idx_active ON users (status) WHERE status = 'active'`
- **THEN** the parser captures the WHERE clause expression in the Index IR

#### Scenario: Index with USING method
- **WHEN** input contains `CREATE INDEX idx_data ON items USING gin (data)`
- **THEN** the parser stores method=`gin` in the Index IR

### Requirement: Parse CREATE TYPE AS ENUM statements
The system SHALL parse `CREATE TYPE ... AS ENUM (...)` statements and store the enum name and value list for use during type mapping.

#### Scenario: Enum type definition
- **WHEN** input contains `CREATE TYPE mood AS ENUM ('happy', 'sad', 'neutral')`
- **THEN** the parser stores enum name=`mood` with values=['happy', 'sad', 'neutral']

### Requirement: Parse CREATE SEQUENCE statements
The system SHALL parse `CREATE SEQUENCE` statements and store them for SERIAL/IDENTITY resolution.

#### Scenario: Sequence associated with SERIAL
- **WHEN** input contains `CREATE SEQUENCE users_id_seq` and a column with `DEFAULT nextval('users_id_seq')`
- **THEN** the parser associates the sequence with the column for SERIAL resolution

### Requirement: Ignore non-DDL statements
The system SHALL skip `COMMENT ON`, `SET`, `SELECT pg_catalog.set_config(...)`, and other non-DDL statements without error.

#### Scenario: Input with COMMENT and SET statements
- **WHEN** input contains `SET search_path = public;` and `COMMENT ON TABLE users IS 'User table';`
- **THEN** the parser skips these statements and produces no IR objects or warnings for them

### Requirement: Schema filtering
The system SHALL filter parsed objects by schema name. By default, only objects in the `public` schema are included. When `include_all_schemas` is enabled, all schemas are included.

#### Scenario: Default schema filter
- **WHEN** input contains tables in `public` and `analytics` schemas and no schema override is set
- **THEN** only tables from the `public` schema appear in the output IR

#### Scenario: Custom schema filter
- **WHEN** `schema` option is set to `analytics`
- **THEN** only tables from the `analytics` schema appear in the output IR

#### Scenario: Include all schemas
- **WHEN** `include_all_schemas` is enabled
- **THEN** tables from all schemas appear in the output IR

### Requirement: Identifier normalization
The system SHALL normalize unquoted identifiers to lowercase (PostgreSQL behavior) and preserve the original case for quoted identifiers. Both `raw` and `normalized` forms SHALL be stored.

#### Scenario: Unquoted identifier
- **WHEN** input contains `CREATE TABLE Users (...)`
- **THEN** the normalized form is `users` (lowercase)

#### Scenario: Quoted identifier
- **WHEN** input contains `CREATE TABLE "UserData" (...)`
- **THEN** the normalized form preserves `UserData` as-is

### Requirement: Graceful parse error handling
The system SHALL NOT panic on malformed input. Unrecognized or invalid statements SHALL be skipped with a warning. In `strict` mode, parse errors SHALL cause the conversion to fail.

#### Scenario: Malformed statement in non-strict mode
- **WHEN** input contains an unparseable statement and strict mode is off
- **THEN** the parser skips the statement, emits a warning with code `PARSE_SKIPPED`, and continues processing

#### Scenario: Malformed statement in strict mode
- **WHEN** input contains an unparseable statement and strict mode is on
- **THEN** the conversion fails with a `ParseError`
