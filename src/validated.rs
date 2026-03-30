use std::collections::HashMap;

use crate::types::EnvValue;

/// The result of a successful schema validation.
///
/// Provides typed accessors so callers never need to match on [`EnvValue`]
/// directly. Variables that are optional and absent are simply not present —
/// the typed accessors return `None` for them.
///
/// # Example
///
/// ```no_run
/// use env_guard::{EnvSchema, VarSpec, EnvType};
///
/// let env = EnvSchema::new()
///     .var(VarSpec::new("PORT", EnvType::Int).default("8080"))
///     .validate()
///     .unwrap();
///
/// let port: i64 = env.get_int("PORT").unwrap();
/// ```
#[derive(Debug)]
pub struct ValidatedEnv {
    values: HashMap<String, EnvValue>,
}

impl ValidatedEnv {
    pub(crate) fn new(values: HashMap<String, EnvValue>) -> Self {
        Self { values }
    }

    /// Returns the raw [`EnvValue`] for `name`, or `None` if absent.
    pub fn get(&self, name: &str) -> Option<&EnvValue> {
        self.values.get(name)
    }

    /// Returns the value as a string slice, or `None` if absent or wrong type.
    pub fn get_str(&self, name: &str) -> Option<&str> {
        match self.values.get(name) {
            Some(EnvValue::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the value as an `i64`, or `None` if absent or wrong type.
    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.values.get(name) {
            Some(EnvValue::Int(i)) => Some(*i),
            _ => None,
        }
    }

    /// Returns the value as an `f64`, or `None` if absent or wrong type.
    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.values.get(name) {
            Some(EnvValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a `bool`, or `None` if absent or wrong type.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.values.get(name) {
            Some(EnvValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Returns an iterator over all validated `(name, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &EnvValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }
}
