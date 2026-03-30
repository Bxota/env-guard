/// Helper: build a fake environment from pairs and run validate_from.
///
/// This avoids `std::env::set_var` which is not thread-safe in tests.
/// `validate_from` accepts any `Fn(&str) -> Result<String, VarError>`,
/// so we can inject a closure over a HashMap instead.
use std::collections::HashMap;

use env_guard::{EnvSchema, EnvType, VarSpec};

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn validate(
    schema: &EnvSchema,
    map: HashMap<String, String>,
) -> Result<env_guard::ValidatedEnv, env_guard::EnvErrors> {
    schema.validate_from(move |name| map.get(name).cloned().ok_or(std::env::VarError::NotPresent))
}

// ── Required / Missing ───────────────────────────────────────────────────────

#[test]
fn missing_required_var_is_an_error() {
    let schema = EnvSchema::new().var(VarSpec::new("API_KEY", EnvType::Str));
    let result = validate(&schema, env(&[]));
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.errors().len(), 1);
    assert!(errors.to_string().contains("'API_KEY' is required"));
}

#[test]
fn missing_optional_var_is_not_an_error() {
    let schema = EnvSchema::new().var(VarSpec::new("LOG_LEVEL", EnvType::Str).optional());
    let result = validate(&schema, env(&[]));
    assert!(result.is_ok());
    let validated = result.unwrap();
    assert!(validated.get("LOG_LEVEL").is_none());
}

// ── Default values ────────────────────────────────────────────────────────────

#[test]
fn default_value_used_when_var_absent() {
    let schema = EnvSchema::new().var(VarSpec::new("PORT", EnvType::Int).default("9000"));
    let validated = validate(&schema, env(&[])).unwrap();
    assert_eq!(validated.get_int("PORT"), Some(9000));
}

#[test]
fn explicit_value_overrides_default() {
    let schema = EnvSchema::new().var(VarSpec::new("PORT", EnvType::Int).default("9000"));
    let validated = validate(&schema, env(&[("PORT", "3000")])).unwrap();
    assert_eq!(validated.get_int("PORT"), Some(3000));
}

// ── Type coercion ─────────────────────────────────────────────────────────────

#[test]
fn int_coercion_succeeds() {
    let schema = EnvSchema::new().var(VarSpec::new("COUNT", EnvType::Int));
    let validated = validate(&schema, env(&[("COUNT", "42")])).unwrap();
    assert_eq!(validated.get_int("COUNT"), Some(42));
}

#[test]
fn int_coercion_fails_with_parse_error() {
    let schema = EnvSchema::new().var(VarSpec::new("COUNT", EnvType::Int));
    let errors = validate(&schema, env(&[("COUNT", "not-a-number")])).unwrap_err();
    assert!(errors.to_string().contains("cannot be parsed as i64"));
}

#[test]
fn float_coercion_succeeds() {
    let schema = EnvSchema::new().var(VarSpec::new("RATIO", EnvType::Float));
    let validated = validate(&schema, env(&[("RATIO", "3.14")])).unwrap();
    let v = validated.get_float("RATIO").unwrap();
    assert!((v - 3.14).abs() < 1e-9);
}

#[test]
fn bool_coercion_accepts_all_truthy_forms() {
    let schema = EnvSchema::new().var(VarSpec::new("FLAG", EnvType::Bool));
    for truthy in ["true", "1", "yes", "on", "TRUE", "YES"] {
        let validated = validate(&schema, env(&[("FLAG", truthy)])).unwrap();
        assert_eq!(
            validated.get_bool("FLAG"),
            Some(true),
            "failed for '{truthy}'"
        );
    }
}

#[test]
fn bool_coercion_accepts_all_falsy_forms() {
    let schema = EnvSchema::new().var(VarSpec::new("FLAG", EnvType::Bool));
    for falsy in ["false", "0", "no", "off", "FALSE", "NO"] {
        let validated = validate(&schema, env(&[("FLAG", falsy)])).unwrap();
        assert_eq!(
            validated.get_bool("FLAG"),
            Some(false),
            "failed for '{falsy}'"
        );
    }
}

// ── Regex validation ──────────────────────────────────────────────────────────

#[test]
#[cfg(feature = "regex-validation")]
fn regex_match_passes() {
    let schema = EnvSchema::new()
        .var(VarSpec::new("LEVEL", EnvType::Str).regex(r"^(debug|info|warn|error)$"));
    let validated = validate(&schema, env(&[("LEVEL", "info")])).unwrap();
    assert_eq!(validated.get_str("LEVEL"), Some("info"));
}

#[test]
#[cfg(feature = "regex-validation")]
fn regex_mismatch_produces_error() {
    let schema = EnvSchema::new()
        .var(VarSpec::new("LEVEL", EnvType::Str).regex(r"^(debug|info|warn|error)$"));
    let errors = validate(&schema, env(&[("LEVEL", "verbose")])).unwrap_err();
    assert!(errors.to_string().contains("does not match pattern"));
}

#[test]
#[cfg(feature = "regex-validation")]
fn invalid_regex_pattern_is_invalid_spec_error() {
    let schema = EnvSchema::new().var(VarSpec::new("X", EnvType::Str).regex(r"[invalid("));
    let errors = validate(&schema, env(&[("X", "anything")])).unwrap_err();
    assert!(errors.to_string().contains("invalid spec for 'X'"));
}

// ── Multiple errors ───────────────────────────────────────────────────────────

#[test]
fn all_errors_collected_in_one_pass() {
    let schema = EnvSchema::new()
        .var(VarSpec::new("MISSING_ONE", EnvType::Str))
        .var(VarSpec::new("MISSING_TWO", EnvType::Int))
        .var(VarSpec::new("BAD_INT", EnvType::Int));

    let errors = validate(&schema, env(&[("BAD_INT", "oops")])).unwrap_err();
    // Three variables, three errors — not short-circuited after the first.
    assert_eq!(errors.errors().len(), 3);
}

// ── Typed accessors ───────────────────────────────────────────────────────────

#[test]
fn typed_accessors_return_none_for_absent_optional_var() {
    let schema = EnvSchema::new().var(VarSpec::new("OPT", EnvType::Bool).optional());
    let validated = validate(&schema, env(&[])).unwrap();
    assert_eq!(validated.get_bool("OPT"), None);
    assert_eq!(validated.get_str("OPT"), None);
    assert_eq!(validated.get("OPT"), None);
}
