//! Warning types and codes for the conversion diagnostics system.

/// Severity levels for conversion warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Minor change with no semantic loss.
    Info,
    /// Semantics partially lost (e.g., numeric precision not enforced).
    Lossy,
    /// Feature dropped entirely (e.g., GIN index method).
    Unsupported,
    /// Conversion failure.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Lossy => write!(f, "lossy"),
            Severity::Unsupported => write!(f, "unsupported"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A conversion warning or diagnostic message.
#[derive(Debug, Clone)]
pub struct Warning {
    /// Warning code (e.g., "TYPE_LOSSY", "SERIAL_TO_ROWID").
    pub code: &'static str,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// Optional object identifier (table, column, index name).
    pub object: Option<String>,
}

impl Warning {
    pub fn new(code: &'static str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            object: None,
        }
    }

    pub fn with_object(mut self, object: impl Into<String>) -> Self {
        self.object = Some(object.into());
        self
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(obj) = &self.object {
            write!(f, "[{}] {}: {}", self.code, obj, self.message)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}

// Warning code constants

// Type mapping warnings
pub const TYPE_WIDTH_IGNORED: &str = "TYPE_WIDTH_IGNORED";
pub const NUMERIC_PRECISION_LOSS: &str = "NUMERIC_PRECISION_LOSS";
pub const BOOLEAN_AS_INTEGER: &str = "BOOLEAN_AS_INTEGER";
pub const DATETIME_TEXT_STORAGE: &str = "DATETIME_TEXT_STORAGE";
pub const TIMEZONE_LOSS: &str = "TIMEZONE_LOSS";
pub const UUID_AS_TEXT: &str = "UUID_AS_TEXT";
pub const JSONB_LOSS: &str = "JSONB_LOSS";
pub const ENUM_AS_TEXT: &str = "ENUM_AS_TEXT";
pub const ARRAY_LOSSY: &str = "ARRAY_LOSSY";
pub const DOMAIN_FLATTENED: &str = "DOMAIN_FLATTENED";
pub const VARCHAR_LENGTH_IGNORED: &str = "VARCHAR_LENGTH_IGNORED";
pub const CHAR_LENGTH_IGNORED: &str = "CHAR_LENGTH_IGNORED";
pub const INTERVAL_AS_TEXT: &str = "INTERVAL_AS_TEXT";
pub const MONEY_AS_TEXT: &str = "MONEY_AS_TEXT";
pub const NETWORK_AS_TEXT: &str = "NETWORK_AS_TEXT";
pub const GEO_AS_TEXT: &str = "GEO_AS_TEXT";
pub const BIT_AS_TEXT: &str = "BIT_AS_TEXT";
pub const XML_AS_TEXT: &str = "XML_AS_TEXT";
pub const RANGE_AS_TEXT: &str = "RANGE_AS_TEXT";
pub const TYPE_UNKNOWN: &str = "TYPE_UNKNOWN";

// Serial/identity warnings
pub const SERIAL_TO_ROWID: &str = "SERIAL_TO_ROWID";
pub const SERIAL_NOT_PRIMARY_KEY: &str = "SERIAL_NOT_PRIMARY_KEY";

// Expression warnings
pub const NEXTVAL_REMOVED: &str = "NEXTVAL_REMOVED";
pub const CAST_REMOVED: &str = "CAST_REMOVED";
pub const DEFAULT_UNSUPPORTED: &str = "DEFAULT_UNSUPPORTED";

// Constraint warnings
pub const FK_TARGET_MISSING: &str = "FK_TARGET_MISSING";
pub const DEFERRABLE_IGNORED: &str = "DEFERRABLE_IGNORED";
pub const CHECK_EXPRESSION_UNSUPPORTED: &str = "CHECK_EXPRESSION_UNSUPPORTED";
pub const ALTER_TARGET_MISSING: &str = "ALTER_TARGET_MISSING";

// Index warnings
pub const INDEX_METHOD_IGNORED: &str = "INDEX_METHOD_IGNORED";
pub const PARTIAL_INDEX_UNSUPPORTED: &str = "PARTIAL_INDEX_UNSUPPORTED";
pub const EXPRESSION_INDEX_UNSUPPORTED: &str = "EXPRESSION_INDEX_UNSUPPORTED";

// Schema warnings
pub const SCHEMA_PREFIXED: &str = "SCHEMA_PREFIXED";

// Sequence warnings
pub const SEQUENCE_IGNORED: &str = "SEQUENCE_IGNORED";

// Parse warnings
pub const PARSE_SKIPPED: &str = "PARSE_SKIPPED";
