## ADDED Requirements

### Requirement: Warning data model
Each warning SHALL contain: a code (string, e.g. `TYPE_LOSSY`), a severity level, a human-readable message, an optional object identifier (table/column/index name), and an optional source location (line/column).

#### Scenario: Warning structure
- **WHEN** a boolean column is mapped to INTEGER
- **THEN** a warning is produced with code=`BOOLEAN_AS_INTEGER`, severity=Info, message describing the mapping, object=`tablename.columnname`

### Requirement: Severity levels
The system SHALL support four severity levels: `Info` (minor change, no semantic loss), `Lossy` (semantics partially lost), `Unsupported` (feature dropped entirely), `Error` (conversion failure).

#### Scenario: Info severity
- **WHEN** a constraint name is dropped (SQLite ignores constraint names)
- **THEN** severity is `Info`

#### Scenario: Lossy severity
- **WHEN** a `numeric(10,2)` precision is not enforced
- **THEN** severity is `Lossy`

#### Scenario: Unsupported severity
- **WHEN** a GIN index method is ignored
- **THEN** severity is `Unsupported`

### Requirement: Strict mode enforcement
When `strict` mode is enabled, the system SHALL treat any warning with severity `Lossy` or higher as an error and fail the conversion.

#### Scenario: Strict mode with lossy warning
- **WHEN** strict mode is on and a `NUMERIC_PRECISION_LOSS` warning is produced
- **THEN** the conversion fails with a `StrictViolation` error listing the offending warnings

#### Scenario: Non-strict mode with lossy warning
- **WHEN** strict mode is off and a `NUMERIC_PRECISION_LOSS` warning is produced
- **THEN** the conversion succeeds and the warning is included in the output

### Requirement: Warning output destination
The system SHALL support writing warnings to stderr (default) or to a file path specified by the `emit_warnings` option.

#### Scenario: Warnings to stderr
- **WHEN** no `emit_warnings` path is specified
- **THEN** warnings are written to stderr

#### Scenario: Warnings to file
- **WHEN** `emit_warnings` is set to `/tmp/warnings.txt`
- **THEN** warnings are written to that file

### Requirement: Warning report format
Each warning in the report SHALL include: the warning code, the affected object, and the message. Warnings SHALL be sorted by source location when available, then by object name.

#### Scenario: Warning report content
- **WHEN** conversion produces 3 warnings
- **THEN** each line contains `[CODE] object: message` format, sorted by location

### Requirement: Warning accumulation across pipeline
The system SHALL accumulate warnings from all pipeline stages (parsing, transformation, rendering) into a single warnings collection in the result.

#### Scenario: Warnings from multiple stages
- **WHEN** parsing skips an unknown statement and transformation drops a complex default
- **THEN** both warnings appear in the final ConvertResult.warnings list
