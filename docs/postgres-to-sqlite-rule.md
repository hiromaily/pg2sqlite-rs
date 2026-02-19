# PostgreSQL → SQLite Conversion Rules (Detailed Mapping Table)

## 1. Data Type Mapping

SQLite uses dynamic typing with type affinity. The following mappings aim to preserve semantic intent while respecting SQLite capabilities.

### 1.1 Integer Types

| PostgreSQL Type            | SQLite Type | Notes                                         | Warning Code       |
| -------------------------- | ----------- | --------------------------------------------- | ------------------ |
| `smallint`                 | `INTEGER`   | No size enforcement                           | TYPE_WIDTH_IGNORED |
| `integer` / `int` / `int4` | `INTEGER`   |                                               | —                  |
| `bigint` / `int8`          | `INTEGER`   | SQLite INTEGER is 64-bit                      | —                  |
| `serial`                   | `INTEGER`   | Prefer mapping to `INTEGER PRIMARY KEY` if PK | SERIAL_TO_ROWID    |
| `bigserial`                | `INTEGER`   | Same as above                                 | SERIAL_TO_ROWID    |

---

### 1.2 Numeric / Floating

| PostgreSQL Type    | SQLite Type | Notes                  | Warning Code           |
| ------------------ | ----------- | ---------------------- | ---------------------- |
| `numeric(p,s)`     | `NUMERIC`   | Precision not enforced | NUMERIC_PRECISION_LOSS |
| `decimal(p,s)`     | `NUMERIC`   | Same as above          | NUMERIC_PRECISION_LOSS |
| `real`             | `REAL`      |                        | —                      |
| `double precision` | `REAL`      |                        | —                      |
| `float4`           | `REAL`      |                        | —                      |
| `float8`           | `REAL`      |                        | —                      |

---

### 1.3 Text Types

| PostgreSQL Type        | SQLite Type | Notes               | Warning Code           |
| ---------------------- | ----------- | ------------------- | ---------------------- |
| `text`                 | `TEXT`      |                     | —                      |
| `varchar(n)`           | `TEXT`      | Length not enforced | VARCHAR_LENGTH_IGNORED |
| `character varying(n)` | `TEXT`      |                     | VARCHAR_LENGTH_IGNORED |
| `char(n)`              | `TEXT`      |                     | CHAR_LENGTH_IGNORED    |
| `character(n)`         | `TEXT`      |                     | CHAR_LENGTH_IGNORED    |

---

### 1.4 Boolean

| PostgreSQL Type | SQLite Type | Notes                       | Warning Code       |
| --------------- | ----------- | --------------------------- | ------------------ |
| `boolean`       | `INTEGER`   | Convention: 0=false, 1=true | BOOLEAN_AS_INTEGER |

Default value mapping:

| PostgreSQL Default | SQLite Default |
| ------------------ | -------------- |
| `true`             | `1`            |
| `false`            | `0`            |

---

### 1.5 Date / Time

| PostgreSQL Type | SQLite Type | Notes            | Warning Code          |
| --------------- | ----------- | ---------------- | --------------------- |
| `date`          | `TEXT`      | ISO8601 storage  | DATETIME_TEXT_STORAGE |
| `timestamp`     | `TEXT`      |                  | DATETIME_TEXT_STORAGE |
| `timestamptz`   | `TEXT`      | TZ not preserved | TIMEZONE_LOSS         |
| `time`          | `TEXT`      |                  | DATETIME_TEXT_STORAGE |
| `timetz`        | `TEXT`      | TZ not preserved | TIMEZONE_LOSS         |

Default mapping:

| PostgreSQL          | SQLite                |
| ------------------- | --------------------- |
| `now()`             | `(CURRENT_TIMESTAMP)` |
| `CURRENT_TIMESTAMP` | `(CURRENT_TIMESTAMP)` |

---

### 1.6 UUID

| PostgreSQL Type | SQLite Type | Notes               | Warning Code |
| --------------- | ----------- | ------------------- | ------------ |
| `uuid`          | `TEXT`      | No native UUID type | UUID_AS_TEXT |

Optional (if enabled):

* `uuid_generate_v4()` → `lower(hex(randomblob(16)))`
  Warning: UUID_FORMAT_DIFFERENCE

---

### 1.7 JSON

| PostgreSQL Type | SQLite Type | Notes                     | Warning Code |
| --------------- | ----------- | ------------------------- | ------------ |
| `json`          | `TEXT`      | JSON1 not assumed         | JSON_AS_TEXT |
| `jsonb`         | `TEXT`      | Binary JSON not preserved | JSONB_LOSS   |

---

### 1.8 Binary

| PostgreSQL Type | SQLite Type |
| --------------- | ----------- |
| `bytea`         | `BLOB`      |

---

### 1.9 Enum Types

PostgreSQL:

```sql
CREATE TYPE mood AS ENUM ('happy','sad');
```

Mapping:

| PG Enum Usage        | SQLite Equivalent      | Notes                              |
| -------------------- | ---------------------- | ---------------------------------- |
| Column type `mood`   | `TEXT`                 |                                    |
| Optional enforcement | `CHECK (col IN (...))` | Only if emulate-enum-check enabled |

Warning: ENUM_AS_TEXT

---

### 1.10 Arrays

| PostgreSQL Type | SQLite Type | Notes                   | Warning Code |
| --------------- | ----------- | ----------------------- | ------------ |
| `integer[]`     | `TEXT`      | Recommend JSON encoding | ARRAY_LOSSY  |
| `text[]`        | `TEXT`      |                         | ARRAY_LOSSY  |

---

### 1.11 Domains

| PostgreSQL      | SQLite                     |
| --------------- | -------------------------- |
| Domain type     | Flatten to base type       |
| Domain NOT NULL | Inline NOT NULL            |
| Domain CHECK    | Inline CHECK if compatible |

Warning: DOMAIN_FLATTENED

---

## 2. Constraint Mapping

---

### 2.1 Primary Key

| PostgreSQL        | SQLite                       |
| ----------------- | ---------------------------- |
| `PRIMARY KEY (a)` | `PRIMARY KEY (a)`            |
| Single integer PK | Prefer `INTEGER PRIMARY KEY` |

Rules:

* If single-column integer PK → emit as `INTEGER PRIMARY KEY`
* Composite PK → table-level constraint

Warning if SERIAL not sole PK: SERIAL_NOT_PRIMARY_KEY

---

### 2.2 Unique

| PostgreSQL     | SQLite         |
| -------------- | -------------- |
| `UNIQUE (a,b)` | `UNIQUE (a,b)` |

Constraint names are dropped (SQLite ignores names).

Warning: CONSTRAINT_NAME_DROPPED

---

### 2.3 Foreign Keys

| PostgreSQL          | SQLite      |
| ------------------- | ----------- |
| `REFERENCES t(id)`  | Same syntax |
| `ON DELETE CASCADE` | Same        |
| `ON UPDATE CASCADE` | Same        |

Unsupported:

* MATCH FULL
* Complex deferrable semantics

Warnings:

* DEFERRABLE_SEMANTICS_CHANGED
* FK_TARGET_MISSING

Requires:

```sql
PRAGMA foreign_keys = ON;
```

---

### 2.4 CHECK

| PostgreSQL            | SQLite    |
| --------------------- | --------- |
| Simple expression     | Preserved |
| PG-specific functions | Dropped   |

Warning: CHECK_EXPRESSION_UNSUPPORTED

---

### 2.5 NOT NULL

Directly preserved.

---

## 3. Default Value Mapping

| PostgreSQL Default   | SQLite Output           | Notes | Warning              |
| -------------------- | ----------------------- | ----- | -------------------- |
| Literal number       | Same                    |       |                      |
| Literal string       | Same                    |       |                      |
| `true/false`         | `1/0`                   |       | BOOLEAN_AS_INTEGER   |
| `now()`              | `(CURRENT_TIMESTAMP)`   |       |                      |
| `nextval('seq')`     | Removed (handled by PK) |       | NEXTVAL_REMOVED      |
| `uuid_generate_v4()` | Removed or emulated     |       | UUID_DEFAULT_REMOVED |
| Casts `::type`       | Remove cast             |       | CAST_REMOVED         |
| Complex expression   | Drop                    |       | DEFAULT_UNSUPPORTED  |

---

## 4. Index Mapping

---

### 4.1 Standard Index

| PostgreSQL                 | SQLite |
| -------------------------- | ------ |
| `CREATE INDEX idx ON t(a)` | Same   |

---

### 4.2 Unique Index

Same syntax supported.

---

### 4.3 Partial Index

| PostgreSQL            | SQLite                  |
| --------------------- | ----------------------- |
| `WHERE a IS NOT NULL` | Preserved if compatible |

If unsupported expression:
Warning: PARTIAL_INDEX_UNSUPPORTED

---

### 4.4 Expression Index

SQLite supports expression indexes (modern versions).

If expression incompatible:
Warning: EXPRESSION_INDEX_UNSUPPORTED

---

### 4.5 Index Method

| PostgreSQL               | SQLite  |
| ------------------------ | ------- |
| USING btree              | Ignored |
| USING gin/gist/brin/hash | Ignored |

Warning: INDEX_METHOD_IGNORED

---

## 5. Sequences

| PostgreSQL        | SQLite  |
| ----------------- | ------- |
| `CREATE SEQUENCE` | Ignored |

Warning: SEQUENCE_IGNORED

Handled implicitly for SERIAL if possible.

---

## 6. ALTER TABLE

Strategy:

| PostgreSQL        | SQLite                  |
| ----------------- | ----------------------- |
| ADD CONSTRAINT    | Merge into CREATE TABLE |
| DROP CONSTRAINT   | Ignored                 |
| ALTER COLUMN TYPE | Not supported           |

Warning:

* ALTER_NOT_SUPPORTED

---

## 7. Schema Handling

| PostgreSQL       | SQLite                 |
| ---------------- | ---------------------- |
| `schema.table`   | `table`                |
| Multiple schemas | prefix with `schema__` |

Warning: SCHEMA_PREFIXED

---

## 8. Unsupported Objects (Ignored in MVP)

| Object            | Action |
| ----------------- | ------ |
| VIEW              | Skip   |
| FUNCTION          | Skip   |
| TRIGGER           | Skip   |
| MATERIALIZED VIEW | Skip   |
| POLICY            | Skip   |
| EXTENSION         | Skip   |

Warning codes:

* OBJECT_SKIPPED_VIEW
* OBJECT_SKIPPED_FUNCTION
* OBJECT_SKIPPED_TRIGGER

---

# Recommended Warning Severity Levels

| Level       | Meaning                  |
| ----------- | ------------------------ |
| INFO        | Minor change             |
| LOSSY       | Semantics partially lost |
| UNSUPPORTED | Dropped entirely         |
| ERROR       | Strict mode failure      |

---

# Deterministic Rendering Rules

1. Tables sorted alphabetically
2. Columns preserve input order
3. Constraints sorted: PK → UNIQUE → CHECK → FK
4. Indexes sorted alphabetically
5. Always terminate statements with semicolon
