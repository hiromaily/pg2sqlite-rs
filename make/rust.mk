
.PHONY: update-rustup
update-rustup:
	rustup update

# Publish to crates.io
# 1. Bump version in Cargo.toml (workspace.package.version)
# 2. Update pg2sqlite-core version in cli/Cargo.toml to match
# 3. Commit the version bump and tag (e.g., git tag v0.1.3)
# 4. Push commit and tag (git push && git push --tags)
# 5. Run `make publish-dry-run` to verify
# 6. Run `make publish` to upload
.PHONY: publish-dry-run
publish-dry-run:
	cargo publish --dry-run -p pg2sqlite-core
	cargo publish --dry-run -p pg2sqlite

.PHONY: publish
publish:
	cargo publish -p pg2sqlite-core
	@echo "Waiting for crates.io to index pg2sqlite-core..."
	@sleep 60
	cargo publish -p pg2sqlite
