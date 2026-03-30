use crate::types::EnvType;

/// Declares a single environment variable: its name, expected type, and constraints.
///
/// Built with a fluent API — chain calls to configure, then pass to
/// [`EnvSchema::var`](crate::EnvSchema::var).
///
/// # Defaults
///
/// - `required: true` — if you name a variable, you almost always need it.
///   Opt into optional explicitly with [`.optional()`](VarSpec::optional).
///
/// # Example
///
/// ```
/// use envguard_rs::{VarSpec, EnvType};
///
/// let spec = VarSpec::new("PORT", EnvType::Int)
///     .required()
///     .default("8080");
///
/// let spec = VarSpec::new("LOG_LEVEL", EnvType::Str)
///     .optional()
///     .default("info")
///     .regex(r"^(debug|info|warn|error)$");
/// ```
#[derive(Debug, Clone)]
pub struct VarSpec {
    pub(crate) name: String,
    pub(crate) ty: EnvType,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) regex: Option<String>,
}

impl VarSpec {
    /// Creates a new spec for a variable named `name` with expected type `ty`.
    ///
    /// The variable is **required** by default. Call [`.optional()`](VarSpec::optional)
    /// to change this.
    pub fn new(name: impl Into<String>, ty: EnvType) -> Self {
        Self {
            name: name.into(),
            ty,
            required: true,
            default: None,
            regex: None,
        }
    }

    /// Marks this variable as required (the default).
    ///
    /// Validation will produce [`EnvError::Missing`](crate::EnvError::Missing)
    /// if the variable is absent and has no default.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Marks this variable as optional.
    ///
    /// If absent and no default is set, the variable is simply omitted from
    /// [`ValidatedEnv`](crate::ValidatedEnv) — no error is produced.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Sets a default value (as a raw string) used when the variable is not set.
    ///
    /// The default goes through the same type coercion and regex checks as a
    /// real value, so an invalid default will still produce an error.
    pub fn default(mut self, val: impl Into<String>) -> Self {
        self.default = Some(val.into());
        self
    }

    /// Constrains the raw string value to match a regex pattern.
    ///
    /// The check runs **before** type coercion, always against the original string.
    /// Requires the `regex-validation` feature (enabled by default).
    ///
    /// Produces [`EnvError::RegexMismatch`](crate::EnvError::RegexMismatch) on failure,
    /// or [`EnvError::InvalidSpec`](crate::EnvError::InvalidSpec) if the pattern itself
    /// is malformed.
    pub fn regex(mut self, pattern: impl Into<String>) -> Self {
        self.regex = Some(pattern.into());
        self
    }
}
