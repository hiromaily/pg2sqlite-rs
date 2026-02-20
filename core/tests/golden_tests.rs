// Golden tests: compare convert_pg_ddl_to_sqlite() output against expected files.

use std::path::PathBuf;

use pg2sqlite_core::{ConvertOptions, convert_pg_ddl_to_sqlite};

/// Resolve a path relative to the workspace root (parent of core/).
fn workspace_path(rel: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().join(rel)
}

fn run_golden_test(fixture: &str, golden: &str, opts: &ConvertOptions) {
    let fixture_path = workspace_path(fixture);
    let golden_path = workspace_path(golden);

    let input = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", fixture_path.display()));
    let expected = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("Failed to read golden {}: {e}", golden_path.display()));

    let result = convert_pg_ddl_to_sqlite(&input, opts)
        .unwrap_or_else(|e| panic!("Conversion failed for {}: {e}", fixture_path.display()));

    assert_eq!(
        result.sqlite_sql,
        expected,
        "Output mismatch for {}\n\n--- Expected ---\n{expected}\n--- Got ---\n{}",
        fixture_path.display(),
        result.sqlite_sql
    );
}

#[test]
fn test_golden_basic_table() {
    run_golden_test(
        "tests/fixtures/basic_table.sql",
        "tests/golden/basic_table.out.sql",
        &ConvertOptions::default(),
    );
}

#[test]
fn test_golden_composite_pk() {
    run_golden_test(
        "tests/fixtures/composite_pk.sql",
        "tests/golden/composite_pk.out.sql",
        &ConvertOptions::default(),
    );
}

#[test]
fn test_golden_foreign_keys() {
    run_golden_test(
        "tests/fixtures/foreign_keys.sql",
        "tests/golden/foreign_keys.out.sql",
        &ConvertOptions::default(),
    );
}

#[test]
fn test_golden_foreign_keys_enabled() {
    run_golden_test(
        "tests/fixtures/foreign_keys.sql",
        "tests/golden/foreign_keys_enabled.out.sql",
        &ConvertOptions {
            enable_foreign_keys: true,
            ..Default::default()
        },
    );
}

#[test]
fn test_golden_serial_types() {
    run_golden_test(
        "tests/fixtures/serial_types.sql",
        "tests/golden/serial_types.out.sql",
        &ConvertOptions::default(),
    );
}

#[test]
fn test_golden_various_types() {
    run_golden_test(
        "tests/fixtures/various_types.sql",
        "tests/golden/various_types.out.sql",
        &ConvertOptions::default(),
    );
}

#[test]
fn test_golden_check_constraint() {
    run_golden_test(
        "tests/fixtures/check_constraint.sql",
        "tests/golden/check_constraint.out.sql",
        &ConvertOptions::default(),
    );
}
