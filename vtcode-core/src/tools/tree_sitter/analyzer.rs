//! Core tree-sitter analyzer for code parsing and analysis
//!
//! This module has been refactored to use language-specific parsers for better
//! maintainability and extensibility. The main analyzer now delegates language-specific
//! operations to specialized parser implementations.

use crate::tools::tree_sitter::analysis::{
    CodeAnalysis, CodeMetrics, DependencyInfo,
};
use crate::tools::tree_sitter::cache::AstCache;
use crate::tools::tree_sitter::highlighting::{HighlightResult, TreeSitterInjectionHighlighter};
use crate::tools::tree_sitter::languages::{SymbolInfo, SymbolKind};
use crate::tools::tree_sitter::parsers::{create_parser, LanguageParser};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::Tree;

/// Tree-sitter analysis error
#[derive(Debug, thiserror::Error)]
pub enum TreeSitterError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("File read error: {0}")]
    FileReadError(String),

    #[error("Language detection failed: {0}")]
    LanguageDetectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),
}

/// Language support enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum LanguageSupport {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Bash,
    Swift,
}

/// Syntax tree representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub root: SyntaxNode,
    pub source_code: String,
    pub language: LanguageSupport,
    pub diagnostics: Vec<Diagnostic>,
}

/// Syntax node in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: String,
    pub start_position: Position,
    pub end_position: Position,
    pub text: String,
    // Children within the AST subtree
    pub children: Vec<SyntaxNode>,
    pub named_children: HashMap<String, Vec<SyntaxNode>>,
    // Collected comments that immediately precede this node as sibling comments
    // (useful for documentation extraction like docstrings or /// comments)
    pub leading_comments: Vec<String>,
}

/// Position in source code
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Position {
    pub row: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub position: Position,
    pub node_kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

/// Main tree-sitter analyzer
///
/// This analyzer has been refactored to use language-specific parser implementations
/// for better maintainability. It now acts as an orchestrator, delegating parsing
/// and analysis tasks to specialized parsers.
pub struct TreeSitterAnalyzer {
    /// Language-specific parsers (created lazily)
    language_parsers: HashMap<LanguageSupport, Box<dyn LanguageParser>>,
    /// Supported languages
    supported_languages: Vec<LanguageSupport>,
    /// Current file being analyzed
    current_file: String,
    /// Optional syntax highlighter with injection support
    highlighter: Option<TreeSitterInjectionHighlighter>,
    /// Optional AST cache for performance optimization
    cache: Option<AstCache>,
}

impl TreeSitterAnalyzer {
    /// Create a new tree-sitter analyzer
    pub fn new() -> Result<Self> {
        let mut languages = vec![
            LanguageSupport::Rust,
            LanguageSupport::Python,
            LanguageSupport::JavaScript,
            LanguageSupport::TypeScript,
            LanguageSupport::Go,
            LanguageSupport::Java,
            LanguageSupport::Bash,
        ];

        if cfg!(feature = "swift") {
            languages.push(LanguageSupport::Swift);
        }

        Ok(Self {
            language_parsers: HashMap::new(),
            supported_languages: languages,
            current_file: String::new(),
            highlighter: TreeSitterInjectionHighlighter::new().ok(),
            cache: Some(AstCache::new(256)), // Initialize with 256-entry LRU cache
        })
    }

    /// Enable AST caching for performance optimization
    pub fn with_cache(mut self, capacity: usize) -> Self {
        self.cache = Some(AstCache::new(capacity));
        self
    }

    /// Disable AST caching
    pub fn without_cache(mut self) -> Self {
        self.cache = None;
        self
    }

    /// Get cache statistics if cache is enabled
    pub fn cache_stats(&self) -> Option<String> {
        self.cache.as_ref().map(|cache| {
            let stats = cache.stats();
            format!(
                "Cache: {} hits, {} misses, {:.1}% hit rate, {} entries",
                stats.hits,
                stats.misses,
                stats.hit_rate(),
                stats.size,
            )
        })
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> &[LanguageSupport] {
        &self.supported_languages
    }

    /// Detect language from file extension
    pub fn detect_language_from_path<P: AsRef<Path>>(&self, path: P) -> Result<LanguageSupport> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                TreeSitterError::LanguageDetectionError("No file extension found".to_string())
            })?;

        let normalized_extension = extension.to_ascii_lowercase();

        match normalized_extension.as_str() {
            "rs" => Ok(LanguageSupport::Rust),
            "py" => Ok(LanguageSupport::Python),
            "js" => Ok(LanguageSupport::JavaScript),
            "ts" => Ok(LanguageSupport::TypeScript),
            "tsx" => Ok(LanguageSupport::TypeScript),
            "jsx" => Ok(LanguageSupport::JavaScript),
            "go" => Ok(LanguageSupport::Go),
            "java" => Ok(LanguageSupport::Java),
            "sh" | "bash" => Ok(LanguageSupport::Bash),
            "swift" => {
                if cfg!(feature = "swift") {
                    Ok(LanguageSupport::Swift)
                } else {
                    Err(TreeSitterError::UnsupportedLanguage("Swift".to_string()).into())
                }
            }
            _ => Err(TreeSitterError::UnsupportedLanguage(extension.to_string()).into()),
        }
    }

    /// Get or create a language parser
    fn get_or_create_parser(&mut self, language: LanguageSupport) -> Result<&mut Box<dyn LanguageParser>> {
        if !self.language_parsers.contains_key(&language) {
            let parser = create_parser(language)?;
            self.language_parsers.insert(language, parser);
        }
        Ok(self.language_parsers.get_mut(&language).unwrap())
    }

    /// Parse source code into a syntax tree with optional caching
    pub fn parse(&mut self, source_code: &str, language: LanguageSupport) -> Result<Tree> {
        // Record the parse in cache if enabled (for statistics and future cache lookups)
        if let Some(cache) = &mut self.cache {
            cache.record_parse(source_code, language);
        }

        let parser = self.get_or_create_parser(language)?;
        parser.parse(source_code)
    }

    /// Extract symbols from a syntax tree
    ///
    /// This method delegates to the language-specific parser implementation.
    pub fn extract_symbols(
        &mut self,
        syntax_tree: &Tree,
        source_code: &str,
        language: LanguageSupport,
    ) -> Result<Vec<SymbolInfo>> {
        let parser = self.get_or_create_parser(language)?;
        parser.extract_symbols(syntax_tree, source_code)
    }

    /// Extract dependencies from a syntax tree
    ///
    /// This method delegates to the language-specific parser implementation.
    pub fn extract_dependencies(
        &self,
        syntax_tree: &Tree,
        language: LanguageSupport,
    ) -> Result<Vec<DependencyInfo>> {
        // We need to get the parser without mutating self
        // Since extract_dependencies doesn't need mutable access, we can work around this
        if let Some(parser) = self.language_parsers.get(&language) {
            return parser.extract_dependencies(syntax_tree, "");
        }

        // If parser doesn't exist yet, we need to handle this case
        // For now, return empty dependencies
        Ok(Vec::new())
    }

    /// Calculate code metrics from a syntax tree
    ///
    /// This method delegates to the language-specific parser implementation.
    pub fn calculate_metrics(&self, syntax_tree: &Tree, source_code: &str, language: LanguageSupport) -> Result<CodeMetrics> {
        // Similar workaround for immutable access
        if let Some(parser) = self.language_parsers.get(&language) {
            return parser.calculate_metrics(syntax_tree, source_code);
        }

        // Fallback to basic metrics if parser doesn't exist
        let lines = source_code.lines().collect::<Vec<_>>();
        Ok(CodeMetrics {
            lines_of_code: lines.len(),
            lines_of_comments: 0,
            blank_lines: lines.iter().filter(|l| l.trim().is_empty()).count(),
            functions_count: 0,
            classes_count: 0,
            variables_count: 0,
            imports_count: 0,
            comment_ratio: 0.0,
        })
    }

    /// Parse file into a syntax tree
    pub async fn parse_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<SyntaxTree> {
        let file_path = file_path.as_ref();
        let language = self.detect_language_from_path(file_path)?;

        let source_code = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| TreeSitterError::FileReadError(e.to_string()))?;

        let tree = self.parse(&source_code, language)?;

        // Convert tree-sitter tree to our SyntaxTree representation
        let root = self.convert_tree_to_syntax_node(tree.root_node(), &source_code);
        let diagnostics = self.collect_diagnostics(&tree, &source_code);

        Ok(SyntaxTree {
            root,
            source_code,
            language,
            diagnostics,
        })
    }

    /// Convert tree-sitter node to our SyntaxNode
    pub fn convert_tree_to_syntax_node(
        &self,
        node: tree_sitter::Node,
        source_code: &str,
    ) -> SyntaxNode {
        let start = node.start_position();
        let end = node.end_position();

        // First, convert all children sequentially so we can compute leading sibling comments
        let mut converted_children: Vec<SyntaxNode> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Gather trailing run of comment siblings immediately preceding this child
            let mut leading_comments: Vec<String> = Vec::new();
            for prev in converted_children.iter().rev() {
                let k = prev.kind.to_lowercase();
                if k.contains("comment") {
                    leading_comments.push(prev.text.trim().to_string());
                } else {
                    break;
                }
            }
            leading_comments.reverse();

            // Convert current child
            let mut converted = self.convert_tree_to_syntax_node(child, source_code);
            converted.leading_comments = leading_comments;
            converted_children.push(converted);
        }

        SyntaxNode {
            kind: node.kind().to_string(),
            start_position: Position {
                row: start.row,
                column: start.column,
                byte_offset: node.start_byte(),
            },
            end_position: Position {
                row: end.row,
                column: end.column,
                byte_offset: node.end_byte(),
            },
            text: source_code[node.start_byte()..node.end_byte()].to_string(),
            children: converted_children,
            named_children: self.collect_named_children(node, source_code),
            leading_comments: Vec::new(),
        }
    }

    /// Collect named children for easier access
    fn collect_named_children(
        &self,
        node: tree_sitter::Node,
        source_code: &str,
    ) -> HashMap<String, Vec<SyntaxNode>> {
        let mut named_children = HashMap::new();

        for child in node.named_children(&mut node.walk()) {
            let kind = child.kind().to_string();
            let syntax_node = self.convert_tree_to_syntax_node(child, source_code);

            named_children
                .entry(kind)
                .or_insert_with(Vec::new)
                .push(syntax_node);
        }

        named_children
    }

    /// Collect diagnostics from the parsed tree
    pub fn collect_diagnostics(&self, tree: &Tree, _source_code: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Basic diagnostics collection - can be extended with more sophisticated analysis
        if tree.root_node().has_error() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "Syntax error detected in code".to_string(),
                position: Position {
                    row: 0,
                    column: 0,
                    byte_offset: 0,
                },
                node_kind: "root".to_string(),
            });
        }

        diagnostics
    }

    /// Get parser statistics
    pub fn get_parser_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert(
            "supported_languages".to_string(),
            self.supported_languages.len(),
        );
        stats.insert(
            "loaded_parsers".to_string(),
            self.language_parsers.len(),
        );
        stats
    }

    /// Analyze file with tree-sitter
    ///
    /// This is the main entry point for comprehensive code analysis.
    pub fn analyze_file_with_tree_sitter(
        &mut self,
        file_path: &std::path::Path,
        source_code: &str,
    ) -> Result<CodeAnalysis> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        let is_swift_path = extension
            .as_deref()
            .map(|ext| ext == "swift")
            .unwrap_or(false);

        if is_swift_path && !cfg!(feature = "swift") {
            return Err(TreeSitterError::UnsupportedLanguage("Swift".to_string()).into());
        }

        let language = match self.detect_language_from_path(file_path) {
            Ok(language) => language,
            Err(err) => match self.detect_language_from_content(source_code) {
                Some(language) => language,
                None => return Err(err),
            },
        };

        self.current_file = file_path.to_string_lossy().to_string();

        let tree = self.parse(source_code, language)?;

        // Extract actual symbols and dependencies using language-specific parsers
        let symbols = self.extract_symbols(&tree, source_code, language)?;
        let dependencies = self.extract_dependencies(&tree, language)?;
        let metrics = self.calculate_metrics(&tree, source_code, language)?;

        Ok(CodeAnalysis {
            file_path: self.current_file.clone(),
            language,
            symbols,
            dependencies,
            metrics,
            issues: vec![], // Would need to implement actual issue detection
            complexity: Default::default(), // Would need to implement actual complexity analysis
            structure: Default::default(), // Would need to implement actual structure analysis
        })
    }

    /// Enhanced syntax highlighting using tree-sitter injection highlighting with multi-language support
    pub fn highlight_syntax_with_injections(
        &mut self,
        source_code: &str,
        language: LanguageSupport,
    ) -> Result<HighlightResult> {
        match &mut self.highlighter {
            Some(highlighter) => highlighter.highlight_with_injections(source_code, language),
            None => Err(TreeSitterError::ParseError(
                "Injection highlighter not available".to_string(),
            )
            .into()),
        }
    }

    /// Enhance an existing syntax tree with highlighting information
    pub fn enhance_syntax_tree_with_highlights(
        &mut self,
        syntax_tree: SyntaxTree,
    ) -> Result<SyntaxTree> {
        match &mut self.highlighter {
            Some(highlighter) => highlighter.enhance_syntax_tree(syntax_tree),
            None => Ok(syntax_tree),
        }
    }

    /// Get a reference to the tree-sitter injection highlighter
    pub fn get_highlighter(&mut self) -> Option<&mut TreeSitterInjectionHighlighter> {
        self.highlighter.as_mut()
    }

    /// Execute a cross-injection query across multiple language sections
    pub fn execute_cross_injection_query(
        &mut self,
        content: &str,
        language: LanguageSupport,
        query_pattern: &str,
    ) -> Result<Vec<crate::tools::tree_sitter::highlighting::QueryMatch>> {
        match &mut self.highlighter {
            Some(highlighter) => {
                highlighter.execute_cross_injection_query(content, language, query_pattern)
            }
            None => Err(TreeSitterError::ParseError(
                "Injection highlighter not available".to_string(),
            )
            .into()),
        }
    }

    /// Enhanced symbol extraction using injection-based cross-language queries
    pub fn extract_symbols_with_injections(
        &mut self,
        source_code: &str,
        language: LanguageSupport,
    ) -> Result<Vec<SymbolInfo>> {
        // Use cross-injection query to find all function definitions across languages
        let function_query = self.get_function_query(language)?;
        let matches = self.execute_cross_injection_query(source_code, language, &function_query)?;

        let mut symbols = Vec::new();
        for query_match in matches {
            symbols.push(SymbolInfo {
                name: query_match.content.clone(),
                kind: SymbolKind::Function,
                position: Position {
                    row: query_match.start_position.row,
                    column: query_match.start_position.column,
                    byte_offset: query_match.start_byte,
                },
                scope: None,
                signature: None,
                documentation: None,
            });
        }

        Ok(symbols)
    }

    /// Execute multiple queries efficiently using a single parsing pass
    pub fn execute_multiple_cross_injection_queries(
        &mut self,
        content: &str,
        language: LanguageSupport,
        query_patterns: &[&str],
    ) -> Result<Vec<Vec<crate::tools::tree_sitter::highlighting::QueryMatch>>> {
        match &mut self.highlighter {
            Some(highlighter) => {
                highlighter.execute_multiple_queries(content, language, query_patterns)
            }
            None => Err(TreeSitterError::ParseError(
                "Injection highlighter not available".to_string(),
            )
            .into()),
        }
    }

    /// Process highlighting for a specific range in the document
    pub fn highlight_syntax_in_range(
        &mut self,
        content: &str,
        language: LanguageSupport,
        start_byte: usize,
        end_byte: usize,
    ) -> Result<crate::tools::tree_sitter::highlighting::HighlightResult> {
        let mut all_captures = Vec::new();

        match &mut self.highlighter {
            Some(highlighter) => {
                let ts_language = highlighter.get_or_load_language(language)?;
                let mut parser = tree_sitter::Parser::new();
                parser.set_language(&ts_language).map_err(|e| {
                    TreeSitterError::ParseError(format!("Failed to set language: {}", e))
                })?;

                let tree = parser.parse(content, None).ok_or_else(|| {
                    TreeSitterError::ParseError("Failed to parse content".to_string())
                })?;

                highlighter.process_highlight_matches_in_range(
                    &tree,
                    content,
                    language,
                    start_byte,
                    end_byte,
                    &mut all_captures,
                )?;

                Ok(crate::tools::tree_sitter::highlighting::HighlightResult {
                    captures: all_captures,
                    main_language: language,
                })
            }
            None => Err(TreeSitterError::ParseError(
                "Injection highlighter not available".to_string(),
            )
            .into()),
        }
    }

    /// Get the appropriate function query for a language
    fn get_function_query(&self, language: LanguageSupport) -> Result<String> {
        let query = match language {
            LanguageSupport::Rust => "(function_item) @function",
            LanguageSupport::Python => "(function_definition) @function",
            LanguageSupport::JavaScript => {
                "(function_declaration) @function (arrow_function) @function"
            }
            LanguageSupport::TypeScript => {
                "(function_declaration) @function (arrow_function) @function (method_definition) @function"
            }
            LanguageSupport::Go => {
                "(function_declaration) @function (method_declaration) @function"
            }
            LanguageSupport::Java => {
                "(method_declaration) @function (constructor_declaration) @function"
            }
            LanguageSupport::Bash => "(function_definition) @function",
            LanguageSupport::Swift => "(function_declaration) @function",
        };
        Ok(query.to_string())
    }

    /// Detect language from content heuristics
    pub fn detect_language_from_content(&self, content: &str) -> Option<LanguageSupport> {
        // Simple heuristic-based language detection
        if content.contains("fn ") && content.contains("{") && content.contains("}") {
            Some(LanguageSupport::Rust)
        } else if content.contains("def ") && content.contains(":") && !content.contains("{") {
            Some(LanguageSupport::Python)
        } else if content.contains("function") && content.contains("{") && content.contains("}") {
            Some(LanguageSupport::JavaScript)
        } else if content.starts_with("#!/bin/bash")
            || content.starts_with("#!/usr/bin/env bash")
            || content.starts_with("#!/bin/sh")
            || content.starts_with("#!/usr/bin/env sh")
            || content.contains("#!/usr/bin/env bash")
            || content.contains("#!/usr/bin/env sh")
        {
            Some(LanguageSupport::Bash)
        } else {
            None
        }
    }
}

impl std::fmt::Display for LanguageSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let language_name = match self {
            LanguageSupport::Rust => "Rust",
            LanguageSupport::Python => "Python",
            LanguageSupport::JavaScript => "JavaScript",
            LanguageSupport::TypeScript => "TypeScript",
            LanguageSupport::Go => "Go",
            LanguageSupport::Java => "Java",
            LanguageSupport::Bash => "Bash",
            LanguageSupport::Swift => "Swift",
        };
        write!(f, "{}", language_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn create_test_analyzer() -> TreeSitterAnalyzer {
        TreeSitterAnalyzer::new().expect("Failed to create analyzer")
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert!(
            analyzer
                .supported_languages
                .contains(&LanguageSupport::Rust)
        );
        assert!(
            analyzer
                .supported_languages
                .contains(&LanguageSupport::Python)
        );
    }

    #[test]
    fn test_language_detection_from_path() {
        let analyzer = create_test_analyzer();

        // Test basic file extensions
        match analyzer.detect_language_from_path(Path::new("main.rs")) {
            Ok(lang) => assert_eq!(lang, LanguageSupport::Rust),
            Err(e) => panic!("Expected Rust language, got error: {}", e),
        }

        match analyzer.detect_language_from_path(Path::new("script.py")) {
            Ok(lang) => assert_eq!(lang, LanguageSupport::Python),
            Err(e) => panic!("Expected Python language, got error: {}", e),
        }

        // Test unknown extension should return error
        assert!(
            analyzer
                .detect_language_from_path(Path::new("file.unknown"))
                .is_err()
        );
    }

    #[test]
    fn test_language_detection_from_content() {
        let analyzer = create_test_analyzer();

        // Test Rust content
        let rust_code = r#"fn main() { println!("Hello, world!"); let x = 42; }"#;
        assert_eq!(
            analyzer.detect_language_from_content(rust_code),
            Some(LanguageSupport::Rust)
        );

        // Test Python content
        let python_code = r#"def main(): print("Hello, world!"); x = 42"#;
        assert_eq!(
            analyzer.detect_language_from_content(python_code),
            Some(LanguageSupport::Python)
        );

        // Test unknown content
        let unknown_code = "This is not code just plain text.";
        assert_eq!(analyzer.detect_language_from_content(unknown_code), None);
    }

    #[test]
    fn test_parse_rust_code() {
        let mut analyzer = create_test_analyzer();

        let rust_code = r#"fn main() { println!("Hello, world!"); let x = 42; }"#;

        let result = analyzer.parse(rust_code, LanguageSupport::Rust);
        assert!(result.is_ok());

        let tree = result.unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[cfg(feature = "swift")]
    #[test]
    fn test_parse_swift_code() {
        let mut analyzer = create_test_analyzer();
        let swift_code = "print(\"Hello, World!\")\n";
        let result = analyzer.parse(swift_code, LanguageSupport::Swift);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_injection_highlighting_integration() {
        let mut analyzer = create_test_analyzer();
        let rust_code = r#"fn main() { println!("Hello, injection highlighting!"); }"#;

        // Test injection-based highlighting
        let result = analyzer.highlight_syntax_with_injections(rust_code, LanguageSupport::Rust);
        assert!(result.is_ok());

        let highlights = result.unwrap();
        assert_eq!(highlights.main_language, LanguageSupport::Rust);
    }

    #[test]
    fn test_cross_injection_query_integration() {
        let mut analyzer = create_test_analyzer();
        let rust_code = r#"fn test() { let x = 42; }"#;

        // Test cross-injection query
        let result = analyzer.execute_cross_injection_query(
            rust_code,
            LanguageSupport::Rust,
            "(function_item) @function",
        );
        assert!(result.is_ok());

        let matches = result.unwrap();
        assert_eq!(matches.len(), 0); // Stub implementation returns empty
    }

    #[test]
    fn test_enhanced_symbol_extraction() {
        let mut analyzer = create_test_analyzer();
        let rust_code = r#"fn test_function() { let x = 42; }"#;

        // Test the enhanced symbol extraction
        let result = analyzer.extract_symbols_with_injections(rust_code, LanguageSupport::Rust);
        assert!(result.is_ok());

        let symbols = result.unwrap();
        assert_eq!(symbols.len(), 0); // Stub highlighter returns empty
    }
}
