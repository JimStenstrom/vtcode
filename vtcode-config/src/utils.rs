//! Common utility functions used across the vtcode-config crate.

use anyhow::{ensure, Result};

/// Returns `true` - commonly used as a serde default for boolean fields.
pub fn default_true() -> bool {
    true
}

/// Returns `false` - commonly used as a serde default for boolean fields.
pub fn default_false() -> bool {
    false
}

/// Validates that a value is within a specified range (inclusive).
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
/// * `field_name` - Name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the value is within the range
/// * `Err` if the value is outside the range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<()> {
    ensure!(
        value >= min && value <= max,
        "{} must be between {} and {}, got {}",
        field_name,
        min,
        max,
        value
    );
    Ok(())
}

/// Validates that a string is not empty after trimming.
///
/// # Arguments
/// * `value` - The string to validate
/// * `field_name` - Name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the string is not empty
/// * `Err` if the string is empty or only whitespace
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty(),
        "{} must not be empty",
        field_name
    );
    Ok(())
}

/// Validates that all strings in a collection are non-empty after trimming.
///
/// # Arguments
/// * `values` - Collection of strings to validate
/// * `field_name` - Name of the field for error messages
///
/// # Returns
/// * `Ok(())` if all strings are non-empty
/// * `Err` if any string is empty or only whitespace
pub fn validate_all_non_empty<'a, I>(values: I, field_name: &str) -> Result<()>
where
    I: IntoIterator<Item = &'a str>,
{
    ensure!(
        values.into_iter().all(|s| !s.trim().is_empty()),
        "{} must not contain empty entries",
        field_name
    );
    Ok(())
}

/// Validates that a value is greater than another value.
///
/// # Arguments
/// * `value` - The value to validate
/// * `other` - The value to compare against
/// * `value_name` - Name of the first value for error messages
/// * `other_name` - Name of the second value for error messages
///
/// # Returns
/// * `Ok(())` if value > other
/// * `Err` if value <= other
pub fn validate_greater_than<T: PartialOrd + std::fmt::Display>(
    value: T,
    other: T,
    value_name: &str,
    other_name: &str,
) -> Result<()> {
    ensure!(
        value > other,
        "{} must be greater than {}",
        value_name,
        other_name
    );
    Ok(())
}
