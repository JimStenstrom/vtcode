//! Language-specific parser implementations
//!
//! This module provides a trait-based abstraction for language-specific parsing behavior.
//! Each language has its own parser implementation that handles language-specific concerns
//! like symbol extraction, dependency resolution, and code metrics calculation.

pub mod rust;
pub mod python;
pub mod javascript;
pub mod generic;

use super::analyzer::{LanguageSupport, Position, TreeSitterError};
use super::languages::{SymbolInfo, SymbolKind};
use crate::tools::tree_sitter::analysis::{CodeMetrics, DependencyInfo};
use anyhow::Result;
use tree_sitter::{Language, Tree};

/// Trait for language-specific parsing behavior
///
/// Each language parser implements this trait to provide language-specific
/// analysis capabilities while maintaining a common interface.
pub trait LanguageParser: Send + Sync {
    /// Get the tree-sitter language for this parser
    fn language(&self) -> Language;

    /// Parse source code into an AST
    ///
    /// # Arguments
    /// * `source` - The source code to parse
    ///
    /// # Returns
    /// * `Ok(Tree)` - The parsed syntax tree
    /// * `Err` - If parsing fails
    fn parse(&mut self, source: &str) -> Result<Tree>;

    /// Extract symbols (functions, classes, variables) from the syntax tree
    ///
    /// # Arguments
    /// * `tree` - The parsed syntax tree
    /// * `source` - The original source code
    ///
    /// # Returns
    /// * `Ok(Vec<SymbolInfo>)` - List of extracted symbols
    /// * `Err` - If extraction fails
    fn extract_symbols(&self, tree: &Tree, source: &str) -> Result<Vec<SymbolInfo>>;

    /// Extract dependencies (imports, includes, use statements) from the syntax tree
    ///
    /// # Arguments
    /// * `tree` - The parsed syntax tree
    /// * `source` - The original source code
    ///
    /// # Returns
    /// * `Ok(Vec<DependencyInfo>)` - List of extracted dependencies
    /// * `Err` - If extraction fails
    fn extract_dependencies(&self, tree: &Tree, source: &str) -> Result<Vec<DependencyInfo>>;

    /// Calculate code metrics from the syntax tree
    ///
    /// # Arguments
    /// * `tree` - The parsed syntax tree
    /// * `source` - The original source code
    ///
    /// # Returns
    /// * `Ok(CodeMetrics)` - Calculated code metrics
    /// * `Err` - If calculation fails
    fn calculate_metrics(&self, tree: &Tree, source: &str) -> Result<CodeMetrics>;

    /// Get the language support type for this parser
    fn language_support(&self) -> LanguageSupport;
}

/// Create a language parser for the specified language
///
/// This factory function returns the appropriate parser implementation
/// for the given language.
///
/// # Arguments
/// * `language` - The language to create a parser for
///
/// # Returns
/// * `Ok(Box<dyn LanguageParser>)` - The language parser
/// * `Err` - If the language is not supported
pub fn create_parser(language: LanguageSupport) -> Result<Box<dyn LanguageParser>> {
    match language {
        LanguageSupport::Rust => Ok(Box::new(rust::RustParser::new()?)),
        LanguageSupport::Python => Ok(Box::new(python::PythonParser::new()?)),
        LanguageSupport::JavaScript | LanguageSupport::TypeScript => {
            Ok(Box::new(javascript::JavaScriptParser::new(language)?))
        }
        _ => Ok(Box::new(generic::GenericParser::new(language)?)),
    }
}
