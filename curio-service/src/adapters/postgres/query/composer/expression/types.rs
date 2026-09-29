//! Allowlisted PostgreSQL cast targets.

/// An allowlist of PostgreSQL types needed by repositories.
#[derive(Clone, Copy)]
pub enum SqlType {
    Jsonb,
    TextArray,
    RealArray,
    Float8,
    TimestampMillis,
    Regconfig,
}
impl SqlType {
    pub(super) fn sql(self) -> &'static str {
        match self {
            Self::Jsonb => "jsonb",
            Self::TextArray => "text[]",
            Self::RealArray => "real[]",
            Self::Float8 => "float8",
            Self::TimestampMillis => "timestamptz(3)",
            Self::Regconfig => "regconfig",
        }
    }
}
