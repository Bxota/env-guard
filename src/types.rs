use std::fmt;

/// The expected type for an environment variable, declared at schema time.
///
/// Used in [`VarSpec`](crate::VarSpec) to describe what type the raw string
/// value should be coerced into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvType {
    /// A plain UTF-8 string (no coercion needed).
    Str,
    /// A 64-bit signed integer (`i64`).
    Int,
    /// A 64-bit float (`f64`).
    Float,
    /// A boolean. Accepts `"true"`, `"false"`, `"1"`, `"0"`, `"yes"`, `"no"`,
    /// `"on"`, `"off"` (case-insensitive).
    Bool,
}

impl fmt::Display for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvType::Str => write!(f, "String"),
            EnvType::Int => write!(f, "i64"),
            EnvType::Float => write!(f, "f64"),
            EnvType::Bool => write!(f, "bool"),
        }
    }
}

/// A parsed environment variable value, produced after successful validation.
///
/// Each variant holds the coerced Rust value. Use the typed accessors on
/// [`ValidatedEnv`](crate::ValidatedEnv) instead of matching directly.
#[derive(Debug, Clone, PartialEq)]
pub enum EnvValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl fmt::Display for EnvValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvValue::Str(s) => write!(f, "{s}"),
            EnvValue::Int(i) => write!(f, "{i}"),
            EnvValue::Float(v) => write!(f, "{v}"),
            EnvValue::Bool(b) => write!(f, "{b}"),
        }
    }
}
