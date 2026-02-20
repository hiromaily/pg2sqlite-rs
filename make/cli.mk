# ==============================================================================
# CLI Usage Examples
# ==============================================================================

# Show CLI help
.PHONY: pg2sqlite-help
pg2sqlite-help:
	cargo run --package pg2sqlite -- --help

# Convert a PostgreSQL DDL file to SQLite
# Usage: make convert INPUT=path/to/input.sql
.PHONY: convert
convert:
	cargo run --package pg2sqlite -- --input $(INPUT)

# Convert with output file
# Usage: make convert-to INPUT=input.sql OUTPUT=output.sql
.PHONY: convert-to
convert-to:
	cargo run --package pg2sqlite -- --input $(INPUT) --output $(OUTPUT)

# Convert with foreign keys enabled
# Usage: make convert-fk INPUT=input.sql
.PHONY: convert-fk
convert-fk:
	cargo run --package pg2sqlite -- --input $(INPUT) --enable-foreign-keys

# Convert in strict mode
# Usage: make convert-strict INPUT=input.sql
.PHONY: convert-strict
convert-strict:
	cargo run --package pg2sqlite -- --input $(INPUT) --strict

# Convert all schemas
# Usage: make convert-all INPUT=input.sql
.PHONY: convert-all
convert-all:
	cargo run --package pg2sqlite -- --input $(INPUT) --include-all-schemas

# Run with custom arguments
# Usage: make run-args ARGS="--input foo.sql --strict"
.PHONY: run-args
run-args:
	cargo run --package pg2sqlite -- $(ARGS)
