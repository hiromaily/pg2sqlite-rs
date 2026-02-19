/// Warning output formatting and strict mode enforcement.
use std::io::Write;
use std::path::Path;

use super::warning::{Severity, Warning};

/// Format and output warnings to the specified destination.
pub fn report_warnings(
    warnings: &[Warning],
    destination: &WarningDestination,
) -> std::io::Result<()> {
    if warnings.is_empty() {
        return Ok(());
    }

    let mut sorted = warnings.to_vec();
    sorted.sort_by(|a, b| a.object.cmp(&b.object).then_with(|| a.code.cmp(b.code)));

    match destination {
        WarningDestination::Stderr => {
            let stderr = std::io::stderr();
            let mut handle = stderr.lock();
            for w in &sorted {
                writeln!(handle, "{w}")?;
            }
        }
        WarningDestination::File(path) => {
            let mut file = std::fs::File::create(path)?;
            for w in &sorted {
                writeln!(file, "{w}")?;
            }
        }
    }

    Ok(())
}

/// Check strict mode: fail if any warning has severity >= Lossy.
pub fn check_strict(warnings: &[Warning]) -> Result<(), StrictViolation> {
    let violations: Vec<&Warning> = warnings
        .iter()
        .filter(|w| w.severity >= Severity::Lossy)
        .collect();

    if violations.is_empty() {
        Ok(())
    } else {
        let messages: Vec<String> = violations.iter().map(|w| w.to_string()).collect();
        Err(StrictViolation { messages })
    }
}

/// Error returned when strict mode finds lossy conversions.
#[derive(Debug)]
pub struct StrictViolation {
    pub messages: Vec<String>,
}

impl std::fmt::Display for StrictViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Strict mode: {} lossy conversion(s) found:",
            self.messages.len()
        )?;
        for msg in &self.messages {
            writeln!(f, "  {msg}")?;
        }
        Ok(())
    }
}

impl std::error::Error for StrictViolation {}

/// Where to send warning output.
pub enum WarningDestination {
    Stderr,
    File(std::path::PathBuf),
}

impl WarningDestination {
    pub fn from_option(path: Option<&Path>) -> Self {
        match path {
            Some(p) if p.to_str() == Some("stderr") => WarningDestination::Stderr,
            Some(p) => WarningDestination::File(p.to_path_buf()),
            None => WarningDestination::Stderr,
        }
    }
}
