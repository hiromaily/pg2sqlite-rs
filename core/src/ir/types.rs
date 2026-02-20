//! PostgreSQL and SQLite type definitions.

/// PostgreSQL data types recognized by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PgType {
    // Integer types
    SmallInt,
    Integer,
    BigInt,

    // Serial types (auto-increment)
    SmallSerial,
    Serial,
    BigSerial,

    // Numeric types
    Numeric {
        precision: Option<u32>,
        scale: Option<u32>,
    },
    Real,
    DoublePrecision,

    // Character types
    Text,
    Varchar {
        length: Option<u32>,
    },
    Char {
        length: Option<u32>,
    },

    // Boolean
    Boolean,

    // Date/Time types
    Date,
    Time {
        with_tz: bool,
    },
    Timestamp {
        with_tz: bool,
    },
    Interval,

    // Binary
    Bytea,

    // UUID
    Uuid,

    // JSON types
    Json,
    Jsonb,

    // Network types
    Inet,
    Cidr,
    MacAddr,

    // Geometric types
    Point,
    Line,
    Lseg,
    Box,
    Path,
    Polygon,
    Circle,

    // Monetary
    Money,

    // Bit string
    Bit {
        length: Option<u32>,
    },
    VarBit {
        length: Option<u32>,
    },

    // XML
    Xml,

    // Range types
    Int4Range,
    Int8Range,
    NumRange,
    TsRange,
    TsTzRange,
    DateRange,

    // Enum (user-defined)
    Enum {
        name: String,
    },

    // Domain (user-defined)
    Domain {
        name: String,
    },

    // Array of another type
    Array {
        element: std::boxed::Box<PgType>,
    },

    // Catch-all for unrecognized types
    Other {
        name: String,
    },
}

/// SQLite type affinities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteType {
    Integer,
    Text,
    Real,
    Numeric,
    Blob,
}

impl std::fmt::Display for SqliteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqliteType::Integer => write!(f, "INTEGER"),
            SqliteType::Text => write!(f, "TEXT"),
            SqliteType::Real => write!(f, "REAL"),
            SqliteType::Numeric => write!(f, "NUMERIC"),
            SqliteType::Blob => write!(f, "BLOB"),
        }
    }
}

impl std::fmt::Display for PgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgType::SmallInt => write!(f, "smallint"),
            PgType::Integer => write!(f, "integer"),
            PgType::BigInt => write!(f, "bigint"),
            PgType::SmallSerial => write!(f, "smallserial"),
            PgType::Serial => write!(f, "serial"),
            PgType::BigSerial => write!(f, "bigserial"),
            PgType::Numeric { precision, scale } => match (precision, scale) {
                (Some(p), Some(s)) => write!(f, "numeric({p},{s})"),
                (Some(p), None) => write!(f, "numeric({p})"),
                _ => write!(f, "numeric"),
            },
            PgType::Real => write!(f, "real"),
            PgType::DoublePrecision => write!(f, "double precision"),
            PgType::Text => write!(f, "text"),
            PgType::Varchar { length } => match length {
                Some(n) => write!(f, "varchar({n})"),
                None => write!(f, "varchar"),
            },
            PgType::Char { length } => match length {
                Some(n) => write!(f, "char({n})"),
                None => write!(f, "char"),
            },
            PgType::Boolean => write!(f, "boolean"),
            PgType::Date => write!(f, "date"),
            PgType::Time { with_tz } => {
                if *with_tz {
                    write!(f, "time with time zone")
                } else {
                    write!(f, "time")
                }
            }
            PgType::Timestamp { with_tz } => {
                if *with_tz {
                    write!(f, "timestamp with time zone")
                } else {
                    write!(f, "timestamp")
                }
            }
            PgType::Interval => write!(f, "interval"),
            PgType::Bytea => write!(f, "bytea"),
            PgType::Uuid => write!(f, "uuid"),
            PgType::Json => write!(f, "json"),
            PgType::Jsonb => write!(f, "jsonb"),
            PgType::Inet => write!(f, "inet"),
            PgType::Cidr => write!(f, "cidr"),
            PgType::MacAddr => write!(f, "macaddr"),
            PgType::Point => write!(f, "point"),
            PgType::Line => write!(f, "line"),
            PgType::Lseg => write!(f, "lseg"),
            PgType::Box => write!(f, "box"),
            PgType::Path => write!(f, "path"),
            PgType::Polygon => write!(f, "polygon"),
            PgType::Circle => write!(f, "circle"),
            PgType::Money => write!(f, "money"),
            PgType::Bit { length } => match length {
                Some(n) => write!(f, "bit({n})"),
                None => write!(f, "bit"),
            },
            PgType::VarBit { length } => match length {
                Some(n) => write!(f, "varbit({n})"),
                None => write!(f, "varbit"),
            },
            PgType::Xml => write!(f, "xml"),
            PgType::Int4Range => write!(f, "int4range"),
            PgType::Int8Range => write!(f, "int8range"),
            PgType::NumRange => write!(f, "numrange"),
            PgType::TsRange => write!(f, "tsrange"),
            PgType::TsTzRange => write!(f, "tstzrange"),
            PgType::DateRange => write!(f, "daterange"),
            PgType::Enum { name } => write!(f, "{name}"),
            PgType::Domain { name } => write!(f, "{name}"),
            PgType::Array { element } => write!(f, "{element}[]"),
            PgType::Other { name } => write!(f, "{name}"),
        }
    }
}
