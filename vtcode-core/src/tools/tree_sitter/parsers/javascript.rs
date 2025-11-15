//! JavaScript/TypeScript-specific parser implementation

use super::{LanguageParser, SymbolInfo, SymbolKind};
use crate::tools::tree_sitter::analysis::{CodeMetrics, DependencyInfo, DependencyKind};
use crate::tools::tree_sitter::analyzer::{LanguageSupport, Position, TreeSitterError};
use anyhow::Result;
use tree_sitter::{Language, Parser, Tree};

/// JavaScript/TypeScript language parser
pub struct JavaScriptParser {
    parser: Parser,
    language_support: LanguageSupport,
}

impl JavaScriptParser {
    /// Create a new JavaScript/TypeScript parser
    pub fn new(language: LanguageSupport) -> Result<Self> {
        let mut parser = Parser::new();
        let ts_language: Language = match language {
            LanguageSupport::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            LanguageSupport::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            _ => return Err(TreeSitterError::UnsupportedLanguage(format!("{:?}", language)).into()),
        };
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

    /// Recursively extract symbols from a node
    fn extract_symbols_recursive(
        &self,
        node: tree_sitter::Node,
        source_code: &str,
        symbols: &mut Vec<SymbolInfo>,
        parent_scope: Option<String>,
    ) -> Result<()> {
        let kind = node.kind();

        // Extract JavaScript/TypeScript-specific symbols
        if kind == "function_declaration"
            || kind == "function_definition"
            || kind == "method_definition"
            || kind == "arrow_function"
        {
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
        } else if kind == "class_declaration" {
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

    /// Extract JavaScript/TypeScript dependencies from a node
    fn extract_js_dependencies_recursive(
        &self,
        node: tree_sitter::Node,
        dependencies: &mut Vec<DependencyInfo>,
    ) -> Result<()> {
        let mut cursor = node.walk();

        // Look for import statements
        if node.kind() == "import_statement" {
            // Extract the module name
            dependencies.push(DependencyInfo {
                name: "unknown_js_module".to_string(), // Would need more parsing for actual name
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
            self.extract_js_dependencies_recursive(child, dependencies)?;
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
        if kind.contains("function") || kind.contains("method") || kind.contains("arrow_function") {
            *functions_count += 1;
        } else if kind.contains("class") {
            *classes_count += 1;
        } else if kind.contains("variable")
            || kind.contains("let")
            || kind.contains("const")
            || kind.contains("assignment")
        {
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

impl LanguageParser for JavaScriptParser {
    fn language(&self) -> Language {
        match self.language_support {
            LanguageSupport::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            LanguageSupport::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            _ => tree_sitter_javascript::LANGUAGE.into(),
        }
    }

    fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None).ok_or_else(|| {
            TreeSitterError::ParseError("Failed to parse JavaScript/TypeScript code".to_string()).into()
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
        self.extract_js_dependencies_recursive(root_node, &mut dependencies)?;
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
            .filter(|l| {
                l.trim().starts_with("//") || l.trim().starts_with("/*")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_function_extraction() {
        let source = r#"
function hello() {
    console.log("Hello");
}
        "#;

        let mut parser = JavaScriptParser::new(LanguageSupport::JavaScript).unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = parser.extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "hello"));
    }

    #[test]
    fn test_javascript_class_extraction() {
        let source = r#"
class MyClass {
    constructor() {}
}
        "#;

        let mut parser = JavaScriptParser::new(LanguageSupport::JavaScript).unwrap();
        let tree = parser.parse(source).unwrap();
        let symbols = parser.extract_symbols(&tree, source).unwrap();

        assert!(symbols.iter().any(|s| s.name == "MyClass"));
    }
}
