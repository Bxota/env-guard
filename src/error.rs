use std::fmt;

use crate::types::EnvType;

/// A single validation failure for one environment variable.
#[derive(Debug, Clone)]
pub enum EnvError {
    /// The variable is required but neither set nor has a default.
    Missing { name: String },
    /// The raw string value could not be coerced into the expected type.
    ParseError {
        name: String,
        raw: String,
        expected: EnvType,
    },
    /// The raw string value does not match the declared regex pattern.
    RegexMismatch {
        name: String,
        raw: String,
        pattern: String,
    },
    /// The spec itself is invalid (e.g. the regex pattern is malformed).
    InvalidSpec { name: String, reason: String },
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvError::Missing { name } => {
                write!(f, "'{name}' is required but not set")
            }
            EnvError::ParseError {
                name,
                raw,
                expected,
            } => {
                write!(f, "'{name}' = {raw:?} cannot be parsed as {expected}")
            }
            EnvError::RegexMismatch { name, raw, pattern } => {
                write!(f, "'{name}' = {raw:?} does not match pattern /{pattern}/")
            }
            EnvError::InvalidSpec { name, reason } => {
                write!(f, "invalid spec for '{name}': {reason}")
            }
        }
    }
}

impl std::error::Error for EnvError {}

/// All validation failures collected in a single pass.
///
/// This is the `Err` variant returned by [`EnvSchema::validate`](crate::EnvSchema::validate).
/// It is a newtype over `Vec<EnvError>` so it can implement `std::error::Error`,
/// which allows using `?` in a `main` that returns `Result<(), Box<dyn Error>>`.
///
/// # Example
/// ```
/// # use envguard::{EnvSchema, VarSpec, EnvType};
/// if let Err(errors) = EnvSchema::new().var(VarSpec::new("MISSING", EnvType::Str)).validate() {
///     eprintln!("{errors}");
///     // prints all errors, one per line
/// }
/// ```
#[derive(Debug)]
pub struct EnvErrors(pub Vec<EnvError>);

impl EnvErrors {
    /// Returns the list of individual errors.
    pub fn errors(&self) -> &[EnvError] {
        &self.0
    }
}

impl fmt::Display for EnvErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Environment validation failed with {} error(s):",
            self.0.len()
        )?;
        for e in &self.0 {
            writeln!(f, "  - {e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for EnvErrors {}
