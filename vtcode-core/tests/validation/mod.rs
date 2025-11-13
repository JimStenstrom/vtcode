//! Validation test suite for Phase 3 provider refactoring
//!
//! This module contains comprehensive validation tests including:
//! - Regression tests for all providers
//! - Provider edge case tests
//! - Integration tests
//! - Compatibility matrix tests
//!
//! See VALIDATION_STRATEGY.md for the complete validation plan.

pub mod mocks;
pub mod fixtures;
pub mod utils;
pub mod provider_regression;
pub mod compatibility_matrix;
// pub mod provider_edge_cases;  // TODO: Implement provider edge case tests
// pub mod integration;  // TODO: Implement integration tests
