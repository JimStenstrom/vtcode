//! Python-specific parser implementation

use super::{LanguageParser, SymbolInfo, SymbolKind};
use crate::tools::tree_sitter::analysis::{CodeMetrics, DependencyInfo, DependencyKind};
use crate::tools::tree_sitter::analyzer::{LanguageSupport, Position, TreeSitterError};
use anyhow::Result;
use tree_sitter::{Language, Parser, Tree};

/// Python language parser
pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_python::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| TreeSitterError::ParseError(format!("Failed to set Python language: {}", e)))?;
        Ok(Self { parser })
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

    /// Recursively extract symbols from a node
    fn extract_symbols_recursive(
        &self,
        node: tree_sitter::Node,
        source_code: &str,
        symbols: &mut Vec<SymbolInfo>,
        parent_scope: Option<String>,
    ) -> Result<()> {
        let kind = node.kind();

        // Extract Python-specific symbols
        if kind == "function_definition" {
            // Extract function name
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
        } else if kind == "class_definition" {
            // Extract class name
            if let Some(name_node) = self.find_child_by_type(node, "identifier") {
                let name = &source_code[name_node.start_byte()..name_node.end_byte()];
                symbols.push(SymbolInfo {
                    name: name.to_string(),
                    kind: SymbolKind::Class,
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

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_recursive(child, source_code, symbols, parent_scope.clone())?;
        }

        Ok(())
    }

    /// Extract Python dependencies from a node
    fn extract_python_dependencies_recursive(
        &self,
        node: tree_sitter::Node,
        dependencies: &mut Vec<DependencyInfo>,
    ) -> Result<()> {
        let mut cursor = node.walk();

        // Look for import statements
        if node.kind() == "import_statement" || node.kind() == "import_from_statement" {
            // Extract the module name
            dependencies.push(DependencyInfo {
                name: "unknown_python_module".to_string(), // Would need more parsing for actual name
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
            self.extract_python_dependencies_recursive(child, dependencies)?;
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
        } else if kind.contains("class") {
            *classes_count += 1;
        } else if kind.contains("assignment") {
            *variables_count += 1;
        } else if kind.contains("import") {
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

impl LanguageParser for PythonParser {
    fn language(&self) -> Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| TreeSitterError::ParseError("Failed to parse Python code".to_string()).into())
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
        self.extract_python_dependencies_recursive(root_node, &mut dependencies)?;
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

        // Count comments
        let lines_of_comments = lines
            .iter()
            .filter(|l| l.trim().starts_with("#"))
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
        LanguageSupport::Python
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_function_extraction() {
        let source = r#"
def hello():
    print("Hello")
        "#;

        let mut parser = PythonParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = parser.extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "hello"));
    }

    #[test]
    fn test_python_class_extraction() {
        let source = r#"
class MyClass:
    pass
        "#;

        let mut parser = PythonParser::new().unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = parser.extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyClass"));
    }
}
