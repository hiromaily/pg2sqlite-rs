## ADDED Requirements

### Requirement: ALTER TABLE constraint merging
The system SHALL collect `ALTER TABLE ... ADD CONSTRAINT` statements and merge them into the corresponding `CREATE TABLE` IR. SQLite cannot add constraints via ALTER TABLE, so all constraints MUST be emitted within the CREATE TABLE statement.

#### Scenario: FK added via ALTER TABLE
- **WHEN** input has `CREATE TABLE orders (...)` and separately `ALTER TABLE orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)`
- **THEN** the output CREATE TABLE for `orders` includes the FK constraint inline

#### Scenario: ALTER TABLE for missing table
- **WHEN** an ALTER TABLE references a table not in the parsed input
- **THEN** the constraint is skipped and `ALTER_TARGET_MISSING` warning is emitted

### Requirement: Single-column integer primary key as rowid alias
The system SHALL emit a single-column integer primary key as `INTEGER PRIMARY KEY` (SQLite rowid alias). The system SHALL NOT emit AUTOINCREMENT unless explicitly configured.

#### Scenario: Single integer PK
- **WHEN** a table has `id integer PRIMARY KEY`
- **THEN** the output is `id INTEGER PRIMARY KEY`

#### Scenario: BIGINT primary key
- **WHEN** a table has `id bigint PRIMARY KEY`
- **THEN** the output is `id INTEGER PRIMARY KEY` (SQLite INTEGER is 64-bit)

### Requirement: Composite primary key
The system SHALL emit composite primary keys as table-level constraints.

#### Scenario: Composite PK
- **WHEN** a table has `PRIMARY KEY (user_id, role_id)`
- **THEN** the output includes `PRIMARY KEY ("user_id", "role_id")` as a table-level constraint

### Requirement: UNIQUE constraint conversion
The system SHALL convert both column-level and table-level UNIQUE constraints. Constraint names SHALL be dropped silently (SQLite ignores constraint names).

#### Scenario: Column-level UNIQUE
- **WHEN** a column has `email text UNIQUE`
- **THEN** the output preserves `UNIQUE` on the column

#### Scenario: Table-level composite UNIQUE
- **WHEN** a table has `UNIQUE (tenant_id, email)`
- **THEN** the output includes `UNIQUE ("tenant_id", "email")` as a table-level constraint

### Requirement: Foreign key conversion
The system SHALL convert REFERENCES constraints including `ON DELETE` and `ON UPDATE` actions (CASCADE, SET NULL, RESTRICT, NO ACTION). FK constraints SHALL only be emitted when `enable_foreign_keys` is enabled.

#### Scenario: FK with cascade actions
- **WHEN** a constraint is `FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE NO ACTION` and foreign keys are enabled
- **THEN** the output includes the same FK constraint with matching actions

#### Scenario: FK disabled
- **WHEN** `enable_foreign_keys` is false
- **THEN** FK constraints are omitted from the output

#### Scenario: FK references missing table
- **WHEN** a FK references a table not present in the output
- **THEN** `FK_TARGET_MISSING` warning is emitted

### Requirement: DEFERRABLE constraint handling
The system SHALL drop DEFERRABLE/INITIALLY DEFERRED modifiers from FK constraints with `DEFERRABLE_SEMANTICS_CHANGED` warning.

#### Scenario: Deferrable FK
- **WHEN** a FK constraint has `DEFERRABLE INITIALLY DEFERRED`
- **THEN** the deferrable modifier is dropped and `DEFERRABLE_SEMANTICS_CHANGED` warning is emitted

### Requirement: CHECK constraint conversion
The system SHALL convert CHECK constraints when the expression uses SQLite-compatible syntax. CHECK constraints with PostgreSQL-specific functions or operators SHALL be dropped with `CHECK_EXPRESSION_UNSUPPORTED` warning.

#### Scenario: Simple CHECK constraint
- **WHEN** a table has `CHECK (age >= 0)`
- **THEN** the output preserves `CHECK (age >= 0)`

#### Scenario: PG-specific CHECK constraint
- **WHEN** a CHECK constraint uses a PostgreSQL-specific operator or function
- **THEN** the constraint is dropped and `CHECK_EXPRESSION_UNSUPPORTED` warning is emitted

### Requirement: Standard index conversion
The system SHALL convert `CREATE INDEX` and `CREATE UNIQUE INDEX` to SQLite equivalents.

#### Scenario: Simple index
- **WHEN** input has `CREATE INDEX idx_email ON users (email)`
- **THEN** output has `CREATE INDEX "idx_email" ON "users" ("email");`

#### Scenario: Unique index
- **WHEN** input has `CREATE UNIQUE INDEX idx_uniq ON users (email)`
- **THEN** output has `CREATE UNIQUE INDEX "idx_uniq" ON "users" ("email");`

### Requirement: Index method handling
The system SHALL ignore USING method clauses (btree, gin, gist, brin, hash) and emit `INDEX_METHOD_IGNORED` warning for non-btree methods.

#### Scenario: GIN index
- **WHEN** input has `CREATE INDEX idx_data ON items USING gin (data)`
- **THEN** the output omits the USING clause and emits `INDEX_METHOD_IGNORED` warning

#### Scenario: B-tree index
- **WHEN** input has `CREATE INDEX idx_name ON users USING btree (name)`
- **THEN** the output omits the USING clause without warning (btree is default)

### Requirement: Partial index handling
The system SHALL preserve partial index WHERE clauses when the expression is SQLite-compatible. Incompatible WHERE clauses SHALL cause the index to be skipped with `PARTIAL_INDEX_UNSUPPORTED` warning.

#### Scenario: Compatible partial index
- **WHEN** input has `CREATE INDEX idx_active ON users (status) WHERE status = 'active'`
- **THEN** output preserves the WHERE clause

#### Scenario: Incompatible partial index
- **WHEN** a partial index WHERE clause uses PostgreSQL-specific functions
- **THEN** the entire index is skipped and `PARTIAL_INDEX_UNSUPPORTED` warning is emitted

### Requirement: Expression index handling
The system SHALL preserve expression indexes when the expression is SQLite-compatible. Incompatible expressions SHALL cause the index to be skipped with `EXPRESSION_INDEX_UNSUPPORTED` warning.

#### Scenario: Compatible expression index
- **WHEN** input has `CREATE INDEX idx_lower ON users (lower(email))`
- **THEN** output preserves the expression index (lower() is available in SQLite)

#### Scenario: Incompatible expression index
- **WHEN** an expression index uses a PostgreSQL-specific function
- **THEN** the index is skipped and `EXPRESSION_INDEX_UNSUPPORTED` warning is emitted

### Requirement: Topological ordering for FK dependencies
When `enable_foreign_keys` is enabled, the system SHALL emit CREATE TABLE statements in dependency order (topological sort of FK references). When cycles exist, the system SHALL fall back to alphabetical order and emit a warning.

#### Scenario: Tables with FK dependency chain
- **WHEN** `orders` references `users` and `order_items` references `orders`, with FKs enabled
- **THEN** output order is: `users`, `orders`, `order_items`

#### Scenario: FK cycle
- **WHEN** table A references table B and table B references table A
- **THEN** tables are emitted in alphabetical order and a cycle warning is emitted

### Requirement: Sequence handling
The system SHALL ignore `CREATE SEQUENCE` statements with `SEQUENCE_IGNORED` warning. Sequences used by SERIAL/IDENTITY columns SHALL be absorbed into INTEGER PRIMARY KEY mapping.

#### Scenario: Standalone sequence
- **WHEN** input has `CREATE SEQUENCE audit_seq`
- **THEN** the sequence is ignored and `SEQUENCE_IGNORED` warning is emitted

### Requirement: Schema name stripping
The system SHALL strip schema prefixes from all identifiers in the output (SQLite has no schemas). When `include_all_schemas` is enabled and table names collide across schemas, the system SHALL prefix with `schema__table` and emit `SCHEMA_PREFIXED` warning.

#### Scenario: Simple schema stripping
- **WHEN** input has `CREATE TABLE public.users (...)`
- **THEN** output has `CREATE TABLE "users" (...)`

#### Scenario: Multi-schema name collision
- **WHEN** both `public.users` and `analytics.users` exist with include_all_schemas enabled
- **THEN** output has `public__users` and `analytics__users` with `SCHEMA_PREFIXED` warnings
