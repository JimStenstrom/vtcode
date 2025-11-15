//! Generic fallback parser implementation for languages without specialized parsers

use super::{LanguageParser, SymbolInfo, SymbolKind};
use crate::tools::tree_sitter::analysis::{CodeMetrics, DependencyInfo, DependencyKind};
use crate::tools::tree_sitter::analyzer::{LanguageSupport, Position, TreeSitterError};
use anyhow::Result;
use tree_sitter::{Language, Parser, Tree};

/// Generic language parser for languages without specialized implementations
pub struct GenericParser {
    parser: Parser,
    language_support: LanguageSupport,
}

impl GenericParser {
    /// Create a new generic parser for the specified language
    pub fn new(language: LanguageSupport) -> Result<Self> {
        let mut parser = Parser::new();
        let ts_language = get_language(language)?;
        parser
            .set_language(&ts_language)
            .map_err(|e| TreeSitterError::ParseError(format!("Failed to set language: {}", e)))?;
        Ok(Self {
            parser,
            language_support: language,
        })
    }

    /// Find a child node of a specific type
    fn find_child_by_type<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        type_name: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == type_name {
                return Some(child);
            }
        }
        None
    }

    /// Recursively extract symbols from a node (generic extraction)
    fn extract_symbols_recursive(
        &self,
        node: tree_sitter::Node,
        source_code: &str,
        symbols: &mut Vec<SymbolInfo>,
        parent_scope: Option<String>,
    ) -> Result<()> {
        let kind = node.kind();

        // Handle Bash-specific symbols
        if self.language_support == LanguageSupport::Bash {
            if kind == "function_definition" {
                if let Some(name_node) = self.find_child_by_type(node, "word") {
                    let name = &source_code[name_node.start_byte()..name_node.end_byte()];
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        position: Position {
                            row: node.start_position().row,
                            column: node.start_position().column,
                            byte_offset: node.start_byte(),
                        },
                        scope: parent_scope.clone(),
                        signature: None,
                        documentation: None,
                    });
                }
            } else if kind == "variable_assignment" {
                if let Some(name_node) = self.find_child_by_type(node, "word") {
                    let name = &source_code[name_node.start_byte()..name_node.end_byte()];
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        kind: SymbolKind::Variable,
                        position: Position {
                            row: node.start_position().row,
                            column: node.start_position().column,
                            byte_offset: node.start_byte(),
                        },
                        scope: parent_scope.clone(),
                        signature: None,
                        documentation: None,
                    });
                }
            }
        } else {
            // Generic extraction for other languages
            if kind.contains("function") || kind.contains("method") {
                // Try to find a name
                if let Some(name_node) = self.find_child_by_type(node, "identifier") {
                    let name = &source_code[name_node.start_byte()..name_node.end_byte()];
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        position: Position {
                            row: node.start_position().row,
                            column: node.start_position().column,
                            byte_offset: node.start_byte(),
                        },
                        scope: parent_scope.clone(),
                        signature: None,
                        documentation: None,
                    });
                }
            }
        }

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_recursive(child, source_code, symbols, parent_scope.clone())?;
        }

        Ok(())
    }

    /// Extract generic dependencies from a node
    fn extract_dependencies_recursive(
        &self,
        node: tree_sitter::Node,
        dependencies: &mut Vec<DependencyInfo>,
    ) -> Result<()> {
        let mut cursor = node.walk();

        // Look for import/include statements
        if node.kind().contains("import") || node.kind().contains("include") {
            // Extract the dependency name
            dependencies.push(DependencyInfo {
                name: "unknown_dependency".to_string(),
                kind: DependencyKind::Import,
                source: node.kind().to_string(),
                position: Position {
                    row: node.start_position().row,
                    column: node.start_position().column,
                    byte_offset: node.start_byte(),
                },
            });
        }

        // Recursively process children
        for child in node.children(&mut cursor) {
            self.extract_dependencies_recursive(child, dependencies)?;
        }

        Ok(())
    }

    /// Recursively count different types of nodes
    fn count_nodes_recursive(
        &self,
        node: tree_sitter::Node,
        functions_count: &mut usize,
        classes_count: &mut usize,
        variables_count: &mut usize,
        imports_count: &mut usize,
    ) {
        let kind = node.kind();

        // Count based on node type
        if kind.contains("function") || kind.contains("method") {
            *functions_count += 1;
        } else if kind.contains("class") || kind.contains("struct") || kind.contains("enum") {
            *classes_count += 1;
        } else if kind.contains("variable")
            || kind.contains("let")
            || kind.contains("const")
            || kind.contains("assignment")
        {
            *variables_count += 1;
        } else if kind.contains("import") || kind.contains("include") || kind.contains("use") {
            *imports_count += 1;
        }

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.count_nodes_recursive(
                child,
                functions_count,
                classes_count,
                variables_count,
                imports_count,
            );
        }
    }
}

impl LanguageParser for GenericParser {
    fn language(&self) -> Language {
        get_language(self.language_support).unwrap()
    }

    fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None).ok_or_else(|| {
            TreeSitterError::ParseError(format!(
                "Failed to parse {} code",
                self.language_support
            ))
            .into()
        })
    }

    fn extract_symbols(&self, tree: &Tree, source: &str) -> Result<Vec<SymbolInfo>> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();
        self.extract_symbols_recursive(root_node, source, &mut symbols, None)?;
        Ok(symbols)
    }

    fn extract_dependencies(&self, tree: &Tree, _source: &str) -> Result<Vec<DependencyInfo>> {
        let mut dependencies = Vec::new();
        let root_node = tree.root_node();
        self.extract_dependencies_recursive(root_node, &mut dependencies)?;
        Ok(dependencies)
    }

    fn calculate_metrics(&self, tree: &Tree, source: &str) -> Result<CodeMetrics> {
        let root_node = tree.root_node();
        let lines = source.lines().collect::<Vec<_>>();

        // Count different types of nodes
        let mut functions_count = 0;
        let mut classes_count = 0;
        let mut variables_count = 0;
        let mut imports_count = 0;

        self.count_nodes_recursive(
            root_node,
            &mut functions_count,
            &mut classes_count,
            &mut variables_count,
            &mut imports_count,
        );

        // Count comments (generic approach)
        let lines_of_comments = lines
            .iter()
            .filter(|l| {
                l.trim().starts_with("//")
                    || l.trim().starts_with("/*")
                    || l.trim().starts_with("#")
            })
            .count();

        let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
        let lines_of_code = lines.len();

        let comment_ratio = if lines_of_code > 0 {
            lines_of_comments as f64 / lines_of_code as f64
        } else {
            0.0
        };

        Ok(CodeMetrics {
            lines_of_code,
            lines_of_comments,
            blank_lines,
            functions_count,
            classes_count,
            variables_count,
            imports_count,
            comment_ratio,
        })
    }

    fn language_support(&self) -> LanguageSupport {
        self.language_support
    }
}

/// Get the tree-sitter language for a language support type
fn get_language(language: LanguageSupport) -> Result<Language> {
    let lang: Language = match language {
        LanguageSupport::Go => tree_sitter_go::LANGUAGE.into(),
        LanguageSupport::Java => tree_sitter_java::LANGUAGE.into(),
        LanguageSupport::Bash => tree_sitter_bash::LANGUAGE.into(),
        LanguageSupport::Swift => {
            #[cfg(feature = "swift")]
            {
                tree_sitter_swift::LANGUAGE.into()
            }
            #[cfg(not(feature = "swift"))]
            {
                return Err(TreeSitterError::UnsupportedLanguage("Swift".to_string()).into());
            }
        }
        _ => {
            return Err(
                TreeSitterError::UnsupportedLanguage(format!("{:?}", language)).into(),
            );
        }
    };
    Ok(lang)
}
