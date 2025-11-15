//! Gemini function calling and tool support
//!
//! This module provides utilities for working with Gemini's function declarations
//! and sanitizing tool parameters to match Gemini API requirements.

use serde_json::{Map, Value};

/// Sanitize function parameters for Gemini API compatibility
///
/// Gemini's function calling API doesn't support certain JSON Schema fields.
/// This function recursively removes unsupported fields to ensure compatibility.
///
/// # Unsupported Fields
/// - additionalProperties
/// - oneOf, anyOf, allOf
/// - exclusiveMaximum, exclusiveMinimum
/// - minimum, maximum
/// - $schema, $id, $ref
/// - definitions
/// - patternProperties
/// - dependencies
/// - const, if, then, else, not
/// - contentMediaType, contentEncoding
///
/// # Reference
/// <https://ai.google.dev/gemini-api/docs/function-calling>
pub fn sanitize_function_parameters(parameters: Value) -> Value {
    match parameters {
        Value::Object(map) => {
            // List of unsupported fields for Gemini API
            const UNSUPPORTED_FIELDS: &[&str] = &[
                "additionalProperties",
                "oneOf",
                "anyOf",
                "allOf",
                "exclusiveMaximum",
                "exclusiveMinimum",
                "minimum",
                "maximum",
                "$schema",
                "$id",
                "$ref",
                "definitions",
                "patternProperties",
                "dependencies",
                "const",
                "if",
                "then",
                "else",
                "not",
                "contentMediaType",
                "contentEncoding",
            ];

            // Process all properties recursively, removing unsupported fields
            let mut sanitized = Map::new();
            for (key, value) in map {
                // Skip unsupported fields at this level
                if UNSUPPORTED_FIELDS.contains(&key.as_str()) {
                    continue;
                }
                // Recursively sanitize nested values
                sanitized.insert(key, sanitize_function_parameters(value));
            }
            Value::Object(sanitized)
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_function_parameters)
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sanitize_function_parameters_removes_additional_properties() {
        let parameters = json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            },
            "additionalProperties": false
        });

        let sanitized = sanitize_function_parameters(parameters);
        let root = sanitized
            .as_object()
            .expect("root parameters should remain an object");
        assert!(!root.contains_key("additionalProperties"));

        let nested = root
            .get("properties")
            .and_then(|value| value.as_object())
            .and_then(|props| props.get("input"))
            .and_then(|value| value.as_object())
            .expect("nested object should be preserved");
        assert!(!nested.contains_key("additionalProperties"));
    }

    #[test]
    fn sanitize_function_parameters_removes_exclusive_min_max() {
        let parameters = json!({
            "type": "object",
            "properties": {
                "max_length": {
                    "type": "integer",
                    "exclusiveMaximum": 1000000,
                    "exclusiveMinimum": 0,
                    "minimum": 1,
                    "maximum": 999999,
                    "description": "Maximum number of characters"
                }
            }
        });

        let sanitized = sanitize_function_parameters(parameters);
        let props = sanitized
            .get("properties")
            .and_then(|v| v.as_object())
            .and_then(|p| p.get("max_length"))
            .and_then(|v| v.as_object())
            .expect("max_length property should exist");

        // These unsupported fields should be removed
        assert!(
            !props.contains_key("exclusiveMaximum"),
            "exclusiveMaximum should be removed"
        );
        assert!(
            !props.contains_key("exclusiveMinimum"),
            "exclusiveMinimum should be removed"
        );
        assert!(!props.contains_key("minimum"), "minimum should be removed");
        assert!(!props.contains_key("maximum"), "maximum should be removed");

        // These supported fields should be preserved
        assert_eq!(props.get("type").and_then(|v| v.as_str()), Some("integer"));
        assert_eq!(
            props.get("description").and_then(|v| v.as_str()),
            Some("Maximum number of characters")
        );
    }

    #[test]
    fn sanitize_preserves_valid_schema() {
        let parameters = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "User name" },
                "age": { "type": "integer", "description": "User age" }
            },
            "required": ["name"]
        });

        let sanitized = sanitize_function_parameters(parameters);
        assert_eq!(sanitized.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(sanitized.get("properties").is_some());
        assert!(sanitized.get("required").is_some());
    }
}
