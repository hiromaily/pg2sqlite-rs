# GitHub Issue Creation Rules

When the user asks to create a GitHub issue (e.g., "issueを作成して", "create an issue", "issue追加"), follow these guidelines.

## Project Context

- **Repository**: hiromaily/pg2sqlc-rs
- **Purpose**: PostgreSQL 16 DDL to SQLite3 DDL schema converter written in Rust
- **Structure**: Cargo workspace with `core` (library) and `cli` (binary) crates

## Issue Guidelines

### Title Conventions

| Type        | Format                          | Example                                       |
| ----------- | ------------------------------- | --------------------------------------------- |
| New Feature | `feat: Add feature description` | `feat: Add DOMAIN type conversion support`    |
| Bug Fix     | `fix: Brief description`        | `fix: Incorrect type mapping for NUMERIC`     |
| Enhancement | `feat: Improve description`     | `feat: Improve ALTER TABLE constraint merging`|
| CLI Option  | `feat: Add \`--flag\` option`   | `feat: Add \`--strict\` mode`                 |
| Docs        | `docs: Description`             | `docs: Add type mapping examples`             |

### Labels

- `enhancement` - New features and improvements
- `bug` - Bug fixes
- `documentation` - Documentation improvements

### Required Sections

1. **Summary** - 1-2 sentences describing the change
2. **Motivation** - Why it's needed, who benefits
3. **Proposed Implementation** - Technical approach, CLI usage, examples
4. **Implementation Notes** - Affected modules, dependencies
5. **Acceptance Criteria** - Checklist of requirements
6. **Priority** - High / Medium / Low with reason

## Issue Template

```markdown
## Summary

[1-2 sentence description]

## Motivation

- [Why this is needed]
- [Who benefits]

## Proposed Implementation

### Input (PostgreSQL DDL)

```sql
-- Example PG DDL
```

### Expected Output (SQLite DDL)

```sql
-- Expected SQLite DDL
```

## Implementation Notes

- [Affected modules: pg/, ir/, transform/, sqlite/]
- [Technical considerations]
- [Warning codes to add]

## Acceptance Criteria

- [ ] Core functionality implemented
- [ ] Unit tests added for transform module
- [ ] Golden tests added (fixtures/ + golden/)
- [ ] Warnings emitted for lossy conversions
- [ ] Documentation updated

## Priority

[High / Medium / Low] - [Brief reason]
```

## Command to Use

```bash
gh issue create \
  --title "feat: Title here" \
  --label "enhancement" \
  --body "$(cat <<'EOF'
## Summary
...

## Motivation
...

## Proposed Implementation
...

## Acceptance Criteria
- [ ] ...

## Priority
...
EOF
)"
```

## Before Creating Issues

1. Check existing issues: `gh issue list`
2. Understand current implementation by reading relevant source files
3. Reference spec docs: `docs/postgres-to-sqlite.md` and `docs/postgres-to-sqlite-rule.md`
4. Consider implementation complexity and user value
