# ==============================================================================
# CLI Usage Examples
# ==============================================================================

# Run the CLI with sample file
.PHONY: run
run:
	cargo run --package pg2sqlite -- examples/sample.yaml

# Run the CLI with arguments
# Usage: make run-args ARGS="--help"
.PHONY: run-args
run-args:
	cargo run --package pg2sqlite -- $(ARGS)

# Run with colored output
.PHONY: run-colored
run-colored:
	cargo run --package pg2sqlite -- -f colored examples/sample.yaml

# ==============================================================================
# YAML Linting Commands
# ==============================================================================

# Lint a single YAML file
# Usage: make pg2sqlite FILE=path/to/file.yaml
.PHONY: pg2sqlite
pg2sqlite:
	cargo run --package pg2sqlite -- $(FILE)

# Lint a directory recursively
# Usage: make pg2sqlite-dir DIR=path/to/dir
.PHONY: pg2sqlite-dir
pg2sqlite-dir:
	cargo run --package pg2sqlite -- $(DIR)

# Lint examples directory
.PHONY: pg2sqlite-examples
pg2sqlite-examples:
	cargo run --package pg2sqlite -- examples/

# Lint with strict mode (warnings cause non-zero exit)
.PHONY: pg2sqlite-strict
pg2sqlite-strict:
	cargo run --package pg2sqlite -- --strict examples/

# Lint with custom config file
# Usage: make pg2sqlite-config CONFIG=.yamllint FILE=examples/
.PHONY: pg2sqlite-config
pg2sqlite-config:
	cargo run --package pg2sqlite -- -c $(CONFIG) $(FILE)

# Lint with relaxed preset
.PHONY: pg2sqlite-relaxed
pg2sqlite-relaxed:
	cargo run --package pg2sqlite -- -d relaxed examples/

# Lint with parsable output (for CI/editors)
.PHONY: pg2sqlite-parsable
pg2sqlite-parsable:
	cargo run --package pg2sqlite -- -f parsable examples/

# Lint with colored output
.PHONY: pg2sqlite-colored
pg2sqlite-colored:
	cargo run --package pg2sqlite -- -f colored examples/

# List files that would be linted
.PHONY: pg2sqlite-list
pg2sqlite-list:
	cargo run --package pg2sqlite -- --list-files examples/

# Lint test fixtures (valid files - should pass)
.PHONY: pg2sqlite-valid
pg2sqlite-valid:
	cargo run --package pg2sqlite -- tests/fixtures/valid/

# Lint test fixtures (invalid files - expected errors)
.PHONY: pg2sqlite-invalid
pg2sqlite-invalid:
	cargo run --package pg2sqlite -- tests/fixtures/invalid/ || true

# Show CLI help
.PHONY: pg2sqlite-help
pg2sqlite-help:
	cargo run --package pg2sqlite -- --help
