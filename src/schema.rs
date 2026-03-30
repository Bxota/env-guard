use std::collections::HashMap;

use crate::{
    error::{EnvError, EnvErrors},
    spec::VarSpec,
    types::{EnvType, EnvValue},
    validated::ValidatedEnv,
};

/// A declarative schema describing all expected environment variables.
///
/// Build one with [`EnvSchema::new`] and chain [`.var()`](EnvSchema::var) calls,
/// then call [`.validate()`](EnvSchema::validate) at application startup.
///
/// # Example
///
/// ```no_run
/// use envguard::{EnvSchema, VarSpec, EnvType};
///
/// let env = EnvSchema::new()
///     .var(VarSpec::new("PORT", EnvType::Int).default("8080"))
///     .var(VarSpec::new("API_KEY", EnvType::Str).required())
///     .validate()
///     .expect("invalid environment");
/// ```
#[derive(Debug, Default)]
pub struct EnvSchema {
    specs: Vec<VarSpec>,
}

impl EnvSchema {
    /// Creates an empty schema.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a variable spec to the schema.
    pub fn var(mut self, spec: VarSpec) -> Self {
        self.specs.push(spec);
        self
    }

    /// Validates the current process environment against the schema.
    ///
    /// Reads every declared variable from `std::env`, applies defaults,
    /// checks types and regex constraints, and collects **all** errors before
    /// returning — never short-circuits on the first failure.
    ///
    /// # Errors
    ///
    /// Returns [`EnvErrors`] if one or more variables fail validation.
    pub fn validate(&self) -> Result<ValidatedEnv, EnvErrors> {
        self.validate_from(|name| std::env::var(name))
    }

    /// Like [`validate`](Self::validate) but reads variables from `source` instead
    /// of `std::env`.
    ///
    /// Useful in tests: pass a closure over a `HashMap` to avoid mutating
    /// the process environment with `std::env::set_var` (which is not
    /// thread-safe across parallel tests).
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use envguard::{EnvSchema, VarSpec, EnvType};
    ///
    /// let mut map = HashMap::new();
    /// map.insert("PORT".to_string(), "8080".to_string());
    ///
    /// let env = EnvSchema::new()
    ///     .var(VarSpec::new("PORT", EnvType::Int))
    ///     .validate_from(move |name| {
    ///         map.get(name).cloned().ok_or(std::env::VarError::NotPresent)
    ///     })
    ///     .unwrap();
    ///
    /// assert_eq!(env.get_int("PORT"), Some(8080));
    /// ```
    pub fn validate_from<F>(&self, source: F) -> Result<ValidatedEnv, EnvErrors>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        let mut values: HashMap<String, EnvValue> = HashMap::new();
        let mut errors: Vec<EnvError> = Vec::new();

        for spec in &self.specs {
            // Step 1: resolve the raw string value (env var or default).
            let raw: Option<String> = match source(&spec.name) {
                Ok(v) => Some(v),
                Err(_) => spec.default.clone(),
            };

            // Step 2: handle absence.
            let raw: String = match raw {
                Some(v) => v,
                None => {
                    if spec.required {
                        errors.push(EnvError::Missing {
                            name: spec.name.clone(),
                        });
                    }
                    // Nothing left to validate for this variable.
                    continue;
                }
            };

            // Step 3: regex check on the raw string (before coercion).
            #[cfg(feature = "regex-validation")]
            if let Some(pattern) = &spec.regex {
                match regex::Regex::new(pattern) {
                    Err(e) => {
                        errors.push(EnvError::InvalidSpec {
                            name: spec.name.clone(),
                            reason: e.to_string(),
                        });
                        // The spec is broken — skip coercion.
                        continue;
                    }
                    Ok(re) if !re.is_match(&raw) => {
                        errors.push(EnvError::RegexMismatch {
                            name: spec.name.clone(),
                            raw: raw.clone(),
                            pattern: pattern.clone(),
                        });
                        // Don't attempt coercion after a regex failure.
                        continue;
                    }
                    Ok(_) => {}
                }
            }

            // Step 4: type coercion.
            match coerce(&raw, spec.ty) {
                Ok(value) => {
                    values.insert(spec.name.clone(), value);
                }
                Err(()) => {
                    errors.push(EnvError::ParseError {
                        name: spec.name.clone(),
                        raw,
                        expected: spec.ty,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(ValidatedEnv::new(values))
        } else {
            Err(EnvErrors(errors))
        }
    }
}

/// Coerces a raw string into the target [`EnvValue`].
///
/// Pure function: no I/O, no side effects. Returns `Err(())` on failure;
/// the caller is responsible for building the right [`EnvError`].
fn coerce(raw: &str, ty: EnvType) -> Result<EnvValue, ()> {
    match ty {
        EnvType::Str => Ok(EnvValue::Str(raw.to_string())),
        EnvType::Int => raw.trim().parse::<i64>().map(EnvValue::Int).map_err(|_| ()),
        EnvType::Float => raw
            .trim()
            .parse::<f64>()
            .map(EnvValue::Float)
            .map_err(|_| ()),
        EnvType::Bool => match raw.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(EnvValue::Bool(true)),
            "false" | "0" | "no" | "off" => Ok(EnvValue::Bool(false)),
            _ => Err(()),
        },
    }
}
