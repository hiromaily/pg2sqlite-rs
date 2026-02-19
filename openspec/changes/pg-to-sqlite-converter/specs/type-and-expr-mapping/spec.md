## ADDED Requirements

### Requirement: Integer type mapping
The system SHALL map `smallint`, `integer`, `int`, `int4`, `bigint`, `int8` to `INTEGER`. The system SHALL emit `TYPE_WIDTH_IGNORED` warning for `smallint`.

#### Scenario: All integer types
- **WHEN** columns have types smallint, integer, int, int4, bigint, int8
- **THEN** all are mapped to `INTEGER`

### Requirement: SERIAL and BIGSERIAL mapping
The system SHALL map `serial` and `bigserial` columns to `INTEGER`. When the column is a single-column primary key, it SHALL be emitted as `INTEGER PRIMARY KEY` (rowid alias). The system SHALL emit `SERIAL_TO_ROWID` warning.

#### Scenario: SERIAL as sole primary key
- **WHEN** a column is `id serial PRIMARY KEY`
- **THEN** it is emitted as `id INTEGER PRIMARY KEY` without AUTOINCREMENT

#### Scenario: SERIAL not a primary key
- **WHEN** a column is `counter serial` without PRIMARY KEY
- **THEN** it is mapped to `INTEGER`, the DEFAULT nextval() is dropped, and `SERIAL_NOT_PRIMARY_KEY` warning is emitted

### Requirement: Numeric and floating-point type mapping
The system SHALL map `numeric(p,s)` and `decimal(p,s)` to `NUMERIC` with `NUMERIC_PRECISION_LOSS` warning. The system SHALL map `real`, `float4`, `float8`, `double precision` to `REAL`.

#### Scenario: Numeric with precision
- **WHEN** a column has type `numeric(10,2)`
- **THEN** it is mapped to `NUMERIC` and `NUMERIC_PRECISION_LOSS` warning is emitted

#### Scenario: Float types
- **WHEN** columns have types real, double precision, float4, float8
- **THEN** all are mapped to `REAL`

### Requirement: Text type mapping
The system SHALL map `text`, `varchar(n)`, `character varying(n)`, `char(n)`, `character(n)` to `TEXT`. Length constraints SHALL be dropped with `VARCHAR_LENGTH_IGNORED` or `CHAR_LENGTH_IGNORED` warnings.

#### Scenario: VARCHAR with length
- **WHEN** a column has type `varchar(255)`
- **THEN** it is mapped to `TEXT` and `VARCHAR_LENGTH_IGNORED` warning is emitted

### Requirement: Boolean type mapping
The system SHALL map `boolean` to `INTEGER` with `BOOLEAN_AS_INTEGER` warning. Default values `true` and `false` SHALL be mapped to `1` and `0`.

#### Scenario: Boolean column with default
- **WHEN** a column is `active boolean DEFAULT true`
- **THEN** it is emitted as `active INTEGER DEFAULT 1` with `BOOLEAN_AS_INTEGER` warning

### Requirement: Date/time type mapping
The system SHALL map `date`, `timestamp`, `timestamptz`, `time`, `timetz` to `TEXT` with `DATETIME_TEXT_STORAGE` warning. `timestamptz` and `timetz` SHALL additionally emit `TIMEZONE_LOSS` warning.

#### Scenario: Timestamptz column
- **WHEN** a column has type `timestamptz`
- **THEN** it is mapped to `TEXT` with both `DATETIME_TEXT_STORAGE` and `TIMEZONE_LOSS` warnings

### Requirement: UUID type mapping
The system SHALL map `uuid` to `TEXT` with `UUID_AS_TEXT` warning.

#### Scenario: UUID column
- **WHEN** a column has type `uuid`
- **THEN** it is mapped to `TEXT` with `UUID_AS_TEXT` warning

### Requirement: JSON type mapping
The system SHALL map `json` to `TEXT` with `JSON_AS_TEXT` warning and `jsonb` to `TEXT` with `JSONB_LOSS` warning.

#### Scenario: JSONB column
- **WHEN** a column has type `jsonb`
- **THEN** it is mapped to `TEXT` with `JSONB_LOSS` warning

### Requirement: Binary type mapping
The system SHALL map `bytea` to `BLOB`.

#### Scenario: Bytea column
- **WHEN** a column has type `bytea`
- **THEN** it is mapped to `BLOB`

### Requirement: Enum type mapping
The system SHALL map columns using a PostgreSQL enum type to `TEXT` with `ENUM_AS_TEXT` warning. When `emulate_enum_check` is enabled, the system SHALL add a `CHECK (col IN (...))` constraint using the enum values.

#### Scenario: Enum column without check emulation
- **WHEN** a column has enum type `mood` and emulate_enum_check is disabled
- **THEN** it is mapped to `TEXT` with `ENUM_AS_TEXT` warning

#### Scenario: Enum column with check emulation
- **WHEN** a column has enum type `mood` with values ('happy','sad') and emulate_enum_check is enabled
- **THEN** it is mapped to `TEXT` with `CHECK (col IN ('happy','sad'))`

### Requirement: Array type mapping
The system SHALL map array types (`type[]`) to `TEXT` with `ARRAY_LOSSY` warning.

#### Scenario: Integer array column
- **WHEN** a column has type `integer[]`
- **THEN** it is mapped to `TEXT` with `ARRAY_LOSSY` warning

### Requirement: Domain type flattening
The system SHALL flatten domain types to their base type, merging NOT NULL and CHECK constraints from the domain definition. The system SHALL emit `DOMAIN_FLATTENED` warning.

#### Scenario: Domain with NOT NULL
- **WHEN** a domain `email_addr` is defined as `text NOT NULL` and a column uses type `email_addr`
- **THEN** the column is mapped to `TEXT NOT NULL` with `DOMAIN_FLATTENED` warning

### Requirement: Default literal preservation
The system SHALL preserve numeric and string literal defaults as-is.

#### Scenario: Numeric default
- **WHEN** a column has `DEFAULT 0`
- **THEN** the output preserves `DEFAULT 0`

#### Scenario: String default
- **WHEN** a column has `DEFAULT 'unknown'`
- **THEN** the output preserves `DEFAULT 'unknown'`

### Requirement: Default now() mapping
The system SHALL map `DEFAULT now()` and `DEFAULT CURRENT_TIMESTAMP` to `DEFAULT (CURRENT_TIMESTAMP)`.

#### Scenario: now() default
- **WHEN** a column has `DEFAULT now()`
- **THEN** it is emitted as `DEFAULT (CURRENT_TIMESTAMP)`

### Requirement: Default nextval() handling
The system SHALL remove `DEFAULT nextval('seq')` when the column is handled as SERIAL/IDENTITY (mapped to INTEGER PRIMARY KEY). Otherwise, the default SHALL be dropped with `NEXTVAL_REMOVED` warning.

#### Scenario: nextval on SERIAL primary key
- **WHEN** a serial column is the primary key with `DEFAULT nextval('users_id_seq')`
- **THEN** the default is removed and the column becomes `INTEGER PRIMARY KEY`

### Requirement: Default boolean mapping
The system SHALL map `DEFAULT true` to `DEFAULT 1` and `DEFAULT false` to `DEFAULT 0`.

#### Scenario: Boolean defaults
- **WHEN** a column has `DEFAULT false`
- **THEN** it is emitted as `DEFAULT 0`

### Requirement: Cast removal in defaults
The system SHALL strip `::type` casts from default expressions with `CAST_REMOVED` warning.

#### Scenario: Cast in default
- **WHEN** a column has `DEFAULT 'active'::text`
- **THEN** it is emitted as `DEFAULT 'active'` with `CAST_REMOVED` warning

### Requirement: Unsupported default handling
The system SHALL drop defaults containing complex expressions, schema-qualified functions, or subqueries with `DEFAULT_UNSUPPORTED` warning. In strict mode, this SHALL cause failure.

#### Scenario: Complex default in non-strict mode
- **WHEN** a column has `DEFAULT uuid_generate_v4()` and strict mode is off
- **THEN** the default is dropped and `UUID_DEFAULT_REMOVED` warning is emitted

#### Scenario: Complex default in strict mode
- **WHEN** a column has `DEFAULT uuid_generate_v4()` and strict mode is on
- **THEN** the conversion fails with an error
