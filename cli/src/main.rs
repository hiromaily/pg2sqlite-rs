use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use pg2sqlc_core::diagnostics::{WarningDestination, report_warnings};
use pg2sqlc_core::{ConvertOptions, convert_pg_ddl_to_sqlite};

#[derive(Parser, Debug)]
#[command(name = "pg2sqlc", about = "Convert PostgreSQL 16 DDL to SQLite3 DDL")]
#[command(version)]
struct Cli {
    /// PostgreSQL DDL input file (UTF-8)
    #[arg(short, long)]
    input: PathBuf,

    /// SQLite DDL output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Filter by schema name (default: "public")
    #[arg(short, long, default_value = "public")]
    schema: String,

    /// Include all schemas (bypass schema filtering)
    #[arg(long)]
    include_all_schemas: bool,

    /// Emit PRAGMA foreign_keys = ON and include FK constraints
    #[arg(long)]
    enable_foreign_keys: bool,

    /// Fail on lossy conversions instead of emitting warnings
    #[arg(long)]
    strict: bool,

    /// Warning output destination: file path or "stderr" (default: stderr)
    #[arg(long)]
    emit_warnings: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read input file
    let input = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("Failed to read input file: {}", cli.input.display()))?;

    // Build options
    let opts = ConvertOptions {
        schema: if cli.include_all_schemas {
            None
        } else {
            Some(cli.schema)
        },
        include_all_schemas: cli.include_all_schemas,
        enable_foreign_keys: cli.enable_foreign_keys,
        strict: cli.strict,
        emit_warnings: cli.emit_warnings.as_ref().map(PathBuf::from),
    };

    // Convert
    let result = convert_pg_ddl_to_sqlite(&input, &opts).context("Conversion failed")?;

    // Output warnings
    let warn_dest = WarningDestination::from_option(opts.emit_warnings.as_deref());
    report_warnings(&result.warnings, &warn_dest).context("Failed to write warnings")?;

    // Write output
    match &cli.output {
        Some(path) => {
            std::fs::write(path, &result.sqlite_sql)
                .with_context(|| format!("Failed to write output file: {}", path.display()))?;
        }
        None => {
            print!("{}", result.sqlite_sql);
        }
    }

    Ok(())
}
