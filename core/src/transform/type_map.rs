/// PostgreSQL type → SQLite type affinity mapping.
use crate::diagnostics::warning::{self, Severity, Warning};
use crate::ir::types::{PgType, SqliteType};

/// Map a PostgreSQL type to a SQLite type affinity, emitting warnings for lossy conversions.
pub fn map_type(pg_type: &PgType, object: &str, warnings: &mut Vec<Warning>) -> SqliteType {
    match pg_type {
        // Integer types
        PgType::SmallInt => {
            warnings.push(
                Warning::new(
                    warning::TYPE_WIDTH_IGNORED,
                    Severity::Info,
                    "smallint width not enforced in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Integer
        }
        PgType::Integer => SqliteType::Integer,
        PgType::BigInt => SqliteType::Integer,

        // Serial types
        PgType::SmallSerial | PgType::Serial | PgType::BigSerial => {
            // Actual handling (SERIAL_TO_ROWID vs SERIAL_NOT_PRIMARY_KEY) is done in planner
            SqliteType::Integer
        }

        // Numeric types
        PgType::Numeric { .. } => {
            warnings.push(
                Warning::new(
                    warning::NUMERIC_PRECISION_LOSS,
                    Severity::Lossy,
                    "numeric precision/scale not enforced in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Numeric
        }
        PgType::Real | PgType::DoublePrecision => SqliteType::Real,

        // Text types
        PgType::Text => SqliteType::Text,
        PgType::Varchar { length } => {
            if length.is_some() {
                warnings.push(
                    Warning::new(
                        warning::VARCHAR_LENGTH_IGNORED,
                        Severity::Lossy,
                        "varchar length constraint not enforced in SQLite",
                    )
                    .with_object(object),
                );
            }
            SqliteType::Text
        }
        PgType::Char { length } => {
            if length.is_some() {
                warnings.push(
                    Warning::new(
                        warning::CHAR_LENGTH_IGNORED,
                        Severity::Lossy,
                        "char length constraint not enforced in SQLite",
                    )
                    .with_object(object),
                );
            }
            SqliteType::Text
        }

        // Boolean
        PgType::Boolean => {
            warnings.push(
                Warning::new(
                    warning::BOOLEAN_AS_INTEGER,
                    Severity::Lossy,
                    "boolean stored as INTEGER (0/1) in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Integer
        }

        // Date/Time types
        PgType::Date => {
            warnings.push(
                Warning::new(
                    warning::DATETIME_TEXT_STORAGE,
                    Severity::Lossy,
                    "date stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }
        PgType::Time { with_tz } => {
            warnings.push(
                Warning::new(
                    warning::DATETIME_TEXT_STORAGE,
                    Severity::Lossy,
                    "time stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            if *with_tz {
                warnings.push(
                    Warning::new(
                        warning::TIMEZONE_LOSS,
                        Severity::Lossy,
                        "timezone information not preserved in SQLite",
                    )
                    .with_object(object),
                );
            }
            SqliteType::Text
        }
        PgType::Timestamp { with_tz } => {
            warnings.push(
                Warning::new(
                    warning::DATETIME_TEXT_STORAGE,
                    Severity::Lossy,
                    "timestamp stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            if *with_tz {
                warnings.push(
                    Warning::new(
                        warning::TIMEZONE_LOSS,
                        Severity::Lossy,
                        "timezone information not preserved in SQLite",
                    )
                    .with_object(object),
                );
            }
            SqliteType::Text
        }
        PgType::Interval => {
            warnings.push(
                Warning::new(
                    warning::INTERVAL_AS_TEXT,
                    Severity::Lossy,
                    "interval stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Binary
        PgType::Bytea => SqliteType::Blob,

        // UUID
        PgType::Uuid => {
            warnings.push(
                Warning::new(
                    warning::UUID_AS_TEXT,
                    Severity::Lossy,
                    "uuid stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // JSON
        PgType::Json => SqliteType::Text,
        PgType::Jsonb => {
            warnings.push(
                Warning::new(
                    warning::JSONB_LOSS,
                    Severity::Lossy,
                    "jsonb binary optimization lost; stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Network types
        PgType::Inet | PgType::Cidr | PgType::MacAddr => {
            warnings.push(
                Warning::new(
                    warning::NETWORK_AS_TEXT,
                    Severity::Lossy,
                    "network type stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Geometric types
        PgType::Point
        | PgType::Line
        | PgType::Lseg
        | PgType::Box
        | PgType::Path
        | PgType::Polygon
        | PgType::Circle => {
            warnings.push(
                Warning::new(
                    warning::GEO_AS_TEXT,
                    Severity::Lossy,
                    "geometric type stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Monetary
        PgType::Money => {
            warnings.push(
                Warning::new(
                    warning::MONEY_AS_TEXT,
                    Severity::Lossy,
                    "money stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Bit string
        PgType::Bit { .. } | PgType::VarBit { .. } => {
            warnings.push(
                Warning::new(
                    warning::BIT_AS_TEXT,
                    Severity::Lossy,
                    "bit string stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // XML
        PgType::Xml => {
            warnings.push(
                Warning::new(
                    warning::XML_AS_TEXT,
                    Severity::Lossy,
                    "xml stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Range types
        PgType::Int4Range
        | PgType::Int8Range
        | PgType::NumRange
        | PgType::TsRange
        | PgType::TsTzRange
        | PgType::DateRange => {
            warnings.push(
                Warning::new(
                    warning::RANGE_AS_TEXT,
                    Severity::Lossy,
                    "range type stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Enum → TEXT
        PgType::Enum { .. } => {
            warnings.push(
                Warning::new(
                    warning::ENUM_AS_TEXT,
                    Severity::Lossy,
                    "enum stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Domain → flatten to base type (handled in planner, but map here as fallback)
        PgType::Domain { name } => {
            warnings.push(
                Warning::new(
                    warning::DOMAIN_FLATTENED,
                    Severity::Info,
                    format!("domain '{name}' flattened to base type"),
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Array → TEXT
        PgType::Array { .. } => {
            warnings.push(
                Warning::new(
                    warning::ARRAY_LOSSY,
                    Severity::Lossy,
                    "array stored as TEXT in SQLite",
                )
                .with_object(object),
            );
            SqliteType::Text
        }

        // Unknown / Other
        PgType::Other { name } => {
            warnings.push(
                Warning::new(
                    warning::TYPE_UNKNOWN,
                    Severity::Lossy,
                    format!("unrecognized type '{name}' mapped to TEXT"),
                )
                .with_object(object),
            );
            SqliteType::Text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_types() {
        let mut w = Vec::new();
        assert_eq!(
            map_type(&PgType::Integer, "t.id", &mut w),
            SqliteType::Integer
        );
        assert!(w.is_empty());

        assert_eq!(
            map_type(&PgType::BigInt, "t.id", &mut w),
            SqliteType::Integer
        );
        assert!(w.is_empty());

        assert_eq!(
            map_type(&PgType::SmallInt, "t.id", &mut w),
            SqliteType::Integer
        );
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, warning::TYPE_WIDTH_IGNORED);
    }

    #[test]
    fn test_numeric_type() {
        let mut w = Vec::new();
        let t = PgType::Numeric {
            precision: Some(10),
            scale: Some(2),
        };
        assert_eq!(map_type(&t, "t.price", &mut w), SqliteType::Numeric);
        assert_eq!(w[0].code, warning::NUMERIC_PRECISION_LOSS);
    }

    #[test]
    fn test_boolean_type() {
        let mut w = Vec::new();
        assert_eq!(
            map_type(&PgType::Boolean, "t.active", &mut w),
            SqliteType::Integer
        );
        assert_eq!(w[0].code, warning::BOOLEAN_AS_INTEGER);
    }

    #[test]
    fn test_timestamp_with_tz() {
        let mut w = Vec::new();
        let t = PgType::Timestamp { with_tz: true };
        assert_eq!(map_type(&t, "t.ts", &mut w), SqliteType::Text);
        assert!(w.iter().any(|w| w.code == warning::TIMEZONE_LOSS));
    }

    #[test]
    fn test_uuid_type() {
        let mut w = Vec::new();
        assert_eq!(map_type(&PgType::Uuid, "t.id", &mut w), SqliteType::Text);
        assert_eq!(w[0].code, warning::UUID_AS_TEXT);
    }

    #[test]
    fn test_bytea_type() {
        let mut w = Vec::new();
        assert_eq!(map_type(&PgType::Bytea, "t.data", &mut w), SqliteType::Blob);
        assert!(w.is_empty());
    }

    #[test]
    fn test_array_type() {
        let mut w = Vec::new();
        let t = PgType::Array {
            element: std::boxed::Box::new(PgType::Integer),
        };
        assert_eq!(map_type(&t, "t.tags", &mut w), SqliteType::Text);
        assert_eq!(w[0].code, warning::ARRAY_LOSSY);
    }

    #[test]
    fn test_enum_type() {
        let mut w = Vec::new();
        let t = PgType::Enum {
            name: "mood".to_string(),
        };
        assert_eq!(map_type(&t, "t.mood", &mut w), SqliteType::Text);
        assert_eq!(w[0].code, warning::ENUM_AS_TEXT);
    }

    #[test]
    fn test_text_types() {
        let mut w = Vec::new();
        assert_eq!(map_type(&PgType::Text, "t.name", &mut w), SqliteType::Text);
        assert!(w.is_empty());

        assert_eq!(
            map_type(&PgType::Varchar { length: Some(255) }, "t.name", &mut w),
            SqliteType::Text
        );
        assert_eq!(w[0].code, warning::VARCHAR_LENGTH_IGNORED);
    }

    #[test]
    fn test_real_types() {
        let mut w = Vec::new();
        assert_eq!(map_type(&PgType::Real, "t.val", &mut w), SqliteType::Real);
        assert!(w.is_empty());
        assert_eq!(
            map_type(&PgType::DoublePrecision, "t.val", &mut w),
            SqliteType::Real
        );
        assert!(w.is_empty());
    }
}
