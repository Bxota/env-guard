//! # env_guard
//!
//! Validate environment variables at application startup with a declarative schema.
//!
//! Define what you expect; `env_guard` reads from the environment, applies defaults,
//! checks types and regex constraints, and returns **all** errors at once.
//!
//! ## Quick start
//!
//! ```no_run
//! use env_guard::{EnvSchema, VarSpec, EnvType};
//!
//! let env = EnvSchema::new()
//!     .var(VarSpec::new("PORT", EnvType::Int).default("8080"))
//!     .var(VarSpec::new("API_KEY", EnvType::Str).required())
//!     .var(
//!         VarSpec::new("LOG_LEVEL", EnvType::Str)
//!             .optional()
//!             .default("info")
//!             .regex(r"^(debug|info|warn|error)$"),
//!     )
//!     .validate()
//!     .unwrap_or_else(|errors| {
//!         eprintln!("{errors}");
//!         std::process::exit(1);
//!     });
//!
//! let port = env.get_int("PORT").unwrap();
//! ```
//!
//! ## Features
//!
//! - `regex-validation` *(default)*: enables [`VarSpec::regex`] using the `regex` crate.
//!   Disable with `default-features = false` for a zero-dependency build.

mod error;
mod schema;
mod spec;
mod types;
mod validated;

pub use error::{EnvError, EnvErrors};
pub use schema::EnvSchema;
pub use spec::VarSpec;
pub use types::{EnvType, EnvValue};
pub use validated::ValidatedEnv;
