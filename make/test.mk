# ==============================================================================
# Test targets
# ==============================================================================

# Run all tests (same as CI)
.PHONY: test
test:
	cargo test --all

# Run tests with output
.PHONY: test-verbose
test-verbose:
	cargo test --all -- --nocapture

# Run specific test
# Usage: make test-one TEST=test_name
.PHONY: test-one
test-one:
	@echo "Usage: make test-one TEST=test_name"
	cargo test $(TEST) -- --nocapture

# ==============================================================================
# Fixture Validation
# ==============================================================================

# Validate fixtures by running conversion on test SQL files
.PHONY: validate-fixtures
validate-fixtures: release
	@echo "=== Validating test fixtures ==="
	@echo ""
	@for f in tests/fixtures/*.sql; do \
		printf "  %-50s " "$$f:"; \
		if ./target/release/pg2sqlc --input "$$f" > /dev/null 2>&1; then \
			echo "✓ PASS"; \
		else \
			echo "✗ FAIL"; \
		fi; \
	done
	@echo ""
	@echo "=== Fixture validation complete ==="

# Validate fixtures with detailed output
.PHONY: validate-fixtures-detail
validate-fixtures-detail: release
	@echo "=== Detailed fixture validation ==="
	@echo ""
	@for f in tests/fixtures/*.sql; do \
		echo "--- $$f ---"; \
		./target/release/pg2sqlc --input "$$f" 2>&1 || true; \
		echo ""; \
	done
