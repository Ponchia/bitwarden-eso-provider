#![forbid(unsafe_code)]

//! Shared data types for Vaultwarden Secrets Operator.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A remote secret selector supplied by an integration adapter.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteRef {
    /// Provider-specific item key.
    pub key: String,
    /// Optional field or property within the item.
    #[serde(default)]
    pub property: Option<String>,
    /// Optional item version or revision selector.
    #[serde(default)]
    pub version: Option<String>,
}

/// A resolved secret document before adapter-specific response formatting.
#[derive(Debug, Clone, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretDocument {
    /// Secret key-value pairs.
    pub data: BTreeMap<String, String>,
    /// Non-sensitive metadata about the resolved source.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl SecretDocument {
    /// Build a single-key secret document.
    #[must_use]
    pub fn single(key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut data = BTreeMap::new();
        data.insert(key.into(), value.into());
        Self {
            data,
            metadata: BTreeMap::new(),
        }
    }
}

/// Shared validation errors.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// A required field was empty.
    #[error("{field} must not be empty")]
    EmptyField {
        /// Field name.
        field: &'static str,
    },
}

/// Result alias for core validation.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validate a non-empty string field.
///
/// # Errors
///
/// Returns [`ValidationError::EmptyField`] when the value is empty after
/// trimming whitespace.
pub fn require_non_empty(value: &str, field: &'static str) -> ValidationResult<()> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyField { field });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_secret_document_contains_one_value() {
        let doc = SecretDocument::single("DATABASE_URL", "postgres://example");

        assert_eq!(
            doc.data.get("DATABASE_URL"),
            Some(&"postgres://example".to_string())
        );
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn non_empty_validation_rejects_blank_values() {
        let Err(err) = require_non_empty("  ", "key") else {
            unreachable!("blank value should fail validation");
        };

        assert_eq!(err.to_string(), "key must not be empty");
    }
}
