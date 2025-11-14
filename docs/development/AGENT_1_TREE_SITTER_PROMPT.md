# Agent 1: Tree-sitter Extraction - Task Brief

**Your Role:** Extract vtcode-tree-sitter crate from vtcode-core
**Estimated Time:** 4-5 hours
**Branch Name:** `phase-4-tree-sitter`
**Working in Parallel With:** Agent 2 (patch extraction), Agent 3 (tools design prep)

---

## Your Mission

You are **Agent 1** in a 3-agent parallel execution of Phase 4. Your job is to extract all tree-sitter code analysis functionality from the vtcode-core monolith into a new, optional, independently reusable crate called **vtcode-tree-sitter**.

This extraction will:
- Remove 2,700 LOC from vtcode-core
- Make code analysis optional (faster builds for users who don't need it)
- Create a reusable tree-sitter integration for other Rust projects

---

## Context: The Big Picture

VTCode is transforming from a 95K LOC monolith to a modular architecture. You're in **Phase 4: Modularize Tools**.

**Current State:**
- Phase 1-2: ✅ Complete (circular deps broken, large subsystems extracted)
- Phase 3: 🔄 In progress (LLM providers being modularized)
- Phase 4: → Your task (tools modularization)

**Your Part:**
- Extract tree-sitter code analysis into standalone crate
- Make it an optional feature in vtcode-core
- Ensure zero impact when disabled

---

## Required Reading

**IMPORTANT:** Read these documents before starting:

1. **docs/development/PHASE_4_AND_5_IMPLEMENTATION.md**
   - Section 4.1: Extract vtcode-tree-sitter (your detailed task)
   - Complete implementation steps
   - Code examples and API designs

2. **docs/development/ARCHITECTURE_TRANSFORMATION.md**
   - Phase 4 overview
   - Understand the target architecture

3. **docs/development/PHASE_4_PARALLELIZATION_STRATEGY.md**
   - Your role in the 3-agent strategy
   - Coordination requirements
   - Merge strategy

---

## Your Tasks (4-5 hours)

### ✅ Pre-Flight Checklist (15 minutes)

Before you start coding:

- [ ] Read all three documentation files above
- [ ] Attend 30-minute kickoff meeting with Agents 2 & 3
- [ ] Confirm branch name: `phase-4-tree-sitter`
- [ ] Verify communication channel is set up
- [ ] Understand Cargo.toml coordination with Agent 2

**Cargo.toml Coordination:**
You and Agent 2 will both modify `vtcode-core/Cargo.toml`. Pre-agree on this structure:

```toml
# vtcode-core/Cargo.toml

[features]
default = ["tree-sitter"]  # ← You add this
tree-sitter = ["vtcode-tree-sitter"]  # ← You add this

[dependencies]
vtcode-tree-sitter = { path = "../vtcode-tree-sitter", optional = true }  # ← You add this
vtcode-patch = { path = "../vtcode-patch" }  # ← Agent 2 adds this
```

---

### Task 4.1.1: Create Crate Structure (30 minutes)

**Goal:** Set up the new vtcode-tree-sitter crate with proper structure.

```bash
# Create branch
git checkout -b phase-4-tree-sitter

# Create directory structure
mkdir -p vtcode-tree-sitter/src/languages
mkdir -p vtcode-tree-sitter/tests
cd vtcode-tree-sitter

# Initialize crate
cat > Cargo.toml << 'EOF'
[package]
name = "vtcode-tree-sitter"
version = "0.1.0"
edition = "2021"
authors = ["VTCode Contributors"]
description = "Tree-sitter based code analysis for VTCode"
license = "MIT OR Apache-2.0"

[dependencies]
vtcode-tool-traits = { path = "../vtcode-tool-traits" }
tree-sitter = "0.22"
tree-sitter-rust = "0.21"
tree-sitter-python = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-typescript = "0.21"
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.8"
EOF

# Create basic README
cat > README.md << 'EOF'
# vtcode-tree-sitter

Tree-sitter based code analysis for VTCode.

Provides multi-language parsing, symbol extraction, and code analysis utilities.

## Features

- Multi-language support (Rust, Python, JavaScript, TypeScript)
- Symbol extraction (functions, classes, methods, variables)
- AST-based code analysis
- Fast and accurate parsing

## Status

⚠️ Under development as part of Phase 4 extraction.
EOF

# Verify structure
tree vtcode-tree-sitter
```

**Checkpoint:** Crate structure created, Cargo.toml compiles.

---

### Task 4.1.2: Define Public API (1 hour)

**Goal:** Create the public API and type definitions.

**Create src/lib.rs:**

```rust
//! Tree-sitter based code analysis for VTCode
//!
//! This crate provides language parsing, symbol extraction, and code analysis
//! utilities using tree-sitter parsers.
//!
//! # Features
//!
//! - Multi-language parsing (Rust, Python, JavaScript, TypeScript)
//! - Symbol extraction (functions, classes, methods, variables)
//! - AST-based code analysis
//! - Fast incremental parsing
//!
//! # Example
//!
//! ```rust
//! use vtcode_tree_sitter::{TreeSitterAnalyzer, Language};
//! use std::path::Path;
//!
//! let analyzer = TreeSitterAnalyzer::new();
//! let code = "fn hello() { println!(\"Hello\"); }";
//!
//! let result = analyzer.analyze_file(
//!     Path::new("example.rs"),
//!     code
//! )?;
//!
//! for symbol in result.symbols {
//!     println!("Found {}: {}", symbol.kind, symbol.name);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod parser;
pub mod query;
pub mod analysis;
pub mod languages;
mod error;

pub use error::{Error, Result};

use std::collections::HashMap;
use std::path::Path;

/// Main tree-sitter analyzer for code analysis
pub struct TreeSitterAnalyzer {
    parsers: HashMap<Language, tree_sitter::Parser>,
}

impl TreeSitterAnalyzer {
    /// Create a new tree-sitter analyzer
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Initialize parsers for each language
        for lang in [Language::Rust, Language::Python, Language::JavaScript, Language::TypeScript] {
            if let Ok(parser) = Self::create_parser(lang) {
                parsers.insert(lang, parser);
            }
        }

        Self { parsers }
    }

    /// Analyze a file and extract symbols
    pub fn analyze_file(&self, path: &Path, content: &str) -> Result<AnalysisResult> {
        let language = Self::detect_language(path)?;
        let parser = self.get_parser(language)?;

        let tree = parser.parse(content, None)
            .ok_or_else(|| Error::ParseError(format!("Failed to parse {}", path.display())))?;

        let symbols = self.extract_symbols_from_tree(&tree, content, language)?;

        Ok(AnalysisResult {
            tree,
            language,
            symbols,
        })
    }

    /// Extract symbols from code content
    pub fn extract_symbols(&self, content: &str, language: Language) -> Result<Vec<Symbol>> {
        let parser = self.get_parser(language)?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| Error::ParseError("Failed to parse content".to_string()))?;

        self.extract_symbols_from_tree(&tree, content, language)
    }

    fn detect_language(path: &Path) -> Result<Language> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Ok(Language::Rust),
            Some("py") => Ok(Language::Python),
            Some("js") => Ok(Language::JavaScript),
            Some("ts") => Ok(Language::TypeScript),
            _ => Err(Error::UnsupportedLanguage(
                path.display().to_string()
            )),
        }
    }

    fn get_parser(&self, language: Language) -> Result<&tree_sitter::Parser> {
        self.parsers.get(&language)
            .ok_or_else(|| Error::ParserNotFound(language))
    }

    fn create_parser(language: Language) -> Result<tree_sitter::Parser> {
        let mut parser = tree_sitter::Parser::new();

        let lang = match language {
            Language::Rust => tree_sitter_rust::language(),
            Language::Python => tree_sitter_python::language(),
            Language::JavaScript => tree_sitter_javascript::language(),
            Language::TypeScript => tree_sitter_typescript::language_typescript(),
        };

        parser.set_language(lang)
            .map_err(|e| Error::LanguageError(format!("Failed to set language: {}", e)))?;

        Ok(parser)
    }

    fn extract_symbols_from_tree(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        language: Language,
    ) -> Result<Vec<Symbol>> {
        // Implementation will be moved from vtcode-core
        todo!("To be implemented in next step")
    }
}

impl Default for TreeSitterAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
}

/// Analysis result containing AST and metadata
#[derive(Debug)]
pub struct AnalysisResult {
    pub tree: tree_sitter::Tree,
    pub language: Language,
    pub symbols: Vec<Symbol>,
}

/// Code symbol (function, class, variable, etc.)
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub children: Vec<Symbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Variable,
    Constant,
    Module,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Constant => write!(f, "constant"),
            SymbolKind::Module => write!(f, "module"),
        }
    }
}

/// Range in source code
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Position in source code (line and column)
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
```

**Create src/error.rs:**

```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse: {0}")]
    ParseError(String),

    #[error("Unsupported language for file: {0}")]
    UnsupportedLanguage(String),

    #[error("Parser not found for language: {0:?}")]
    ParserNotFound(super::Language),

    #[error("Language configuration error: {0}")]
    LanguageError(String),
}
```

**Create placeholder modules:**

```bash
# src/parser.rs
echo "// Parser implementation - to be moved from vtcode-core" > src/parser.rs

# src/query.rs
echo "// Query implementation - to be moved from vtcode-core" > src/query.rs

# src/analysis.rs
echo "// Analysis utilities - to be moved from vtcode-core" > src/analysis.rs

# src/languages/mod.rs
mkdir -p src/languages
echo "// Language-specific implementations" > src/languages/mod.rs
```

**Checkpoint:** Run `cargo check` in vtcode-tree-sitter. Should compile (with todo! warnings).

---

### Task 4.1.3: Move Tree-sitter Logic (1.5 hours)

**Goal:** Move existing tree-sitter code from vtcode-core to vtcode-tree-sitter.

**Find the code:**

```bash
# From vtcode-core root
find vtcode-core/src -name "*tree*" -o -name "*sitter*" | grep -v target
```

**Expected locations:**
- `vtcode-core/src/code_analysis/tree_sitter/`
- `vtcode-core/src/tools/code_analysis.rs` (parts of it)

**Move strategy:**

1. **Copy the files** (don't delete yet - safer)
   ```bash
   cp -r vtcode-core/src/code_analysis/tree_sitter/* vtcode-tree-sitter/src/
   ```

2. **Update imports** in the copied files
   - Change `crate::` to appropriate paths
   - Change `vtcode_core::` to `vtcode_tree_sitter::`
   - Update use statements

3. **Implement the extraction methods**
   - Move symbol extraction logic
   - Move query logic
   - Move language-specific parsing

4. **Test compilation**
   ```bash
   cd vtcode-tree-sitter
   cargo check
   cargo build
   ```

**Key code to move/implement:**

Look for these functions in vtcode-core:
- Symbol extraction logic
- Tree traversal code
- Query string definitions
- Language-specific parsers

**Checkpoint:** vtcode-tree-sitter compiles and contains all tree-sitter logic.

---

### Task 4.1.4: Implement Tests (30 minutes)

**Goal:** Write comprehensive tests to ensure extraction works.

**Create tests/integration_tests.rs:**

```rust
use vtcode_tree_sitter::*;
use std::path::Path;

#[test]
fn test_rust_function_parsing() {
    let analyzer = TreeSitterAnalyzer::new();
    let code = r#"
        fn hello_world() {
            println!("Hello, world!");
        }

        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    let result = analyzer.analyze_file(
        Path::new("test.rs"),
        code
    ).unwrap();

    assert_eq!(result.language, Language::Rust);
    assert_eq!(result.symbols.len(), 2);

    let names: Vec<_> = result.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"hello_world"));
    assert!(names.contains(&"add"));

    for symbol in &result.symbols {
        assert_eq!(symbol.kind, SymbolKind::Function);
    }
}

#[test]
fn test_python_parsing() {
    let analyzer = TreeSitterAnalyzer::new();
    let code = r#"
def greet(name):
    print(f"Hello, {name}!")

class Person:
    def __init__(self, name):
        self.name = name

    def say_hello(self):
        print(f"Hello, I'm {self.name}")
    "#;

    let result = analyzer.analyze_file(
        Path::new("test.py"),
        code
    ).unwrap();

    assert_eq!(result.language, Language::Python);
    assert!(result.symbols.len() >= 2); // function + class
}

#[test]
fn test_javascript_parsing() {
    let analyzer = TreeSitterAnalyzer::new();
    let code = r#"
        function hello() {
            console.log("Hello!");
        }

        class MyClass {
            constructor() {
                this.value = 0;
            }
        }
    "#;

    let result = analyzer.analyze_file(
        Path::new("test.js"),
        code
    ).unwrap();

    assert_eq!(result.language, Language::JavaScript);
    assert!(result.symbols.len() >= 2);
}

#[test]
fn test_unsupported_language() {
    let analyzer = TreeSitterAnalyzer::new();
    let code = "some code";

    let result = analyzer.analyze_file(
        Path::new("test.xyz"),
        code
    );

    assert!(result.is_err());
}

#[test]
fn test_extract_symbols_directly() {
    let analyzer = TreeSitterAnalyzer::new();
    let code = "fn main() {}";

    let symbols = analyzer.extract_symbols(code, Language::Rust).unwrap();

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "main");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}
```

**Run tests:**

```bash
cd vtcode-tree-sitter
cargo test
```

**Checkpoint:** All tests pass.

---

### Task 4.1.5: Add Feature Flag to vtcode-core (45 minutes)

**Goal:** Make tree-sitter optional in vtcode-core.

**Step 1: Update vtcode-core/Cargo.toml**

Add to the file (coordinate with Agent 2 on exact placement):

```toml
[features]
default = ["tree-sitter"]
tree-sitter = ["vtcode-tree-sitter"]

[dependencies]
vtcode-tree-sitter = { path = "../vtcode-tree-sitter", optional = true }
```

**Step 2: Update vtcode-core code to use feature flag**

Find where tree-sitter is used in vtcode-core (likely `src/tools/code_analysis.rs`):

```rust
// vtcode-core/src/tools/code_analysis.rs

#[cfg(feature = "tree-sitter")]
use vtcode_tree_sitter::{TreeSitterAnalyzer, Language};

pub struct CodeAnalysisTool {
    #[cfg(feature = "tree-sitter")]
    analyzer: TreeSitterAnalyzer,
}

impl CodeAnalysisTool {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "tree-sitter")]
            analyzer: TreeSitterAnalyzer::new(),
        }
    }

    pub fn analyze(&self, path: &Path, content: &str) -> Result<Option<AnalysisResult>> {
        #[cfg(feature = "tree-sitter")]
        {
            Ok(Some(self.analyzer.analyze_file(path, content)?))
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            // Graceful degradation when tree-sitter is disabled
            Ok(None)
        }
    }
}
```

**Step 3: Remove old tree-sitter code from vtcode-core**

```bash
# Remove the old tree_sitter directory (now that code is in vtcode-tree-sitter)
git rm -r vtcode-core/src/code_analysis/tree_sitter/
```

**Step 4: Test both configurations**

```bash
# Test with tree-sitter enabled (default)
cd vtcode-core
cargo build --features tree-sitter
cargo test --features tree-sitter

# Test without tree-sitter
cargo build --no-default-features
cargo test --no-default-features
```

**Checkpoint:** vtcode-core compiles with and without tree-sitter feature.

---

### Task 4.1.6: Write Documentation (30 minutes)

**Goal:** Complete README and crate documentation.

**Update vtcode-tree-sitter/README.md:**

```markdown
# vtcode-tree-sitter

Tree-sitter based code analysis for VTCode.

## Features

- **Multi-language parsing**: Rust, Python, JavaScript, TypeScript
- **Symbol extraction**: Functions, classes, methods, variables, constants
- **AST-based analysis**: Fast and accurate code structure analysis
- **Incremental parsing**: Efficient re-parsing for code changes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vtcode-tree-sitter = "0.1"
```

## Usage

### Basic Analysis

```rust
use vtcode_tree_sitter::{TreeSitterAnalyzer, Language};
use std::path::Path;

let analyzer = TreeSitterAnalyzer::new();
let code = r#"
    fn calculate(x: i32, y: i32) -> i32 {
        x + y
    }
"#;

let result = analyzer.analyze_file(
    Path::new("example.rs"),
    code
)?;

println!("Language: {:?}", result.language);
println!("Found {} symbols:", result.symbols.len());

for symbol in result.symbols {
    println!("  {} {}", symbol.kind, symbol.name);
}
```

### Extract Symbols

```rust
let symbols = analyzer.extract_symbols(code, Language::Rust)?;

for symbol in symbols {
    println!("{} at line {}", symbol.name, symbol.range.start.line);
}
```

## Supported Languages

| Language | File Extensions | Symbol Types |
|----------|----------------|--------------|
| Rust | `.rs` | Functions, Structs, Traits, Impls, Constants |
| Python | `.py` | Functions, Classes, Methods, Variables |
| JavaScript | `.js` | Functions, Classes, Methods, Variables |
| TypeScript | `.ts` | Functions, Classes, Interfaces, Types |

## Performance

Tree-sitter is designed for:
- **Speed**: Parses large files in milliseconds
- **Incrementality**: Re-parses only changed portions
- **Robustness**: Handles syntax errors gracefully

## Integration with VTCode

This crate is optional in vtcode-core. Enable with:

```toml
vtcode-core = { version = "0.44", features = ["tree-sitter"] }
```

## License

MIT OR Apache-2.0
```

**Checkpoint:** Documentation is complete and clear.

---

### Task 4.1.7: Final Testing & Commit (30 minutes)

**Goal:** Ensure everything works and commit your changes.

**Final testing checklist:**

```bash
# 1. Test vtcode-tree-sitter standalone
cd vtcode-tree-sitter
cargo clean
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# 2. Test vtcode-core with tree-sitter
cd ../vtcode-core
cargo test --features tree-sitter

# 3. Test vtcode-core without tree-sitter
cargo test --no-default-features

# 4. Test full workspace
cd ..
cargo test --all
```

**Commit your work:**

```bash
git add .
git commit -m "feat(phase4): Extract vtcode-tree-sitter crate

Extract tree-sitter code analysis functionality from vtcode-core into
standalone vtcode-tree-sitter crate.

Changes:
- Created vtcode-tree-sitter crate with multi-language support
- Supports Rust, Python, JavaScript, TypeScript parsing
- Symbol extraction for functions, classes, methods, variables
- Made tree-sitter optional in vtcode-core via feature flag
- Removed 2,700 LOC from vtcode-core
- Full test coverage for all languages
- Comprehensive documentation

Testing:
- All vtcode-tree-sitter tests pass
- vtcode-core works with and without tree-sitter feature
- No regressions in existing functionality

Part of Phase 4: Modularize Tools"

git push -u origin phase-4-tree-sitter
```

---

## Coordination Points

### With Agent 2 (Patch Extraction)

**Cargo.toml Coordination:**
- ✅ Confirm structure before starting (30 min kickoff)
- ✅ You add `tree-sitter` feature and dependency
- ✅ Agent 2 adds `vtcode-patch` dependency
- ✅ No conflicts if pre-coordinated

**Checkpoint 1 (Hour 1):**
- Share: "Crate structure created, API defined"
- Check: Agent 2 should be at similar stage

**Checkpoint 2 (Hour 3):**
- Share: "Code moved, tests passing"
- Check: Agent 2 should be implementing patch application

**Checkpoint 3 (Hour 5 - Wave 1 Complete):**
- Share: "Feature flag working, all tests pass, ready to merge"
- Coordinate: Who merges first? (Recommend Agent 2 first, then you rebase)

### With Agent 3 (Tools Enhancement)

**Design Review:**
- Agent 3 may ask questions about your trait design during their prep phase
- Be available to review their plugin interface design
- Share: Your public API once finalized (hour 2)

---

## Success Criteria

Your task is complete when:

- [ ] ✅ vtcode-tree-sitter crate exists and compiles
- [ ] ✅ All 4 languages supported (Rust, Python, JS, TS)
- [ ] ✅ Symbol extraction works correctly
- [ ] ✅ All tests pass (100% for core functionality)
- [ ] ✅ Feature flag works in vtcode-core
- [ ] ✅ vtcode-core compiles with AND without tree-sitter
- [ ] ✅ No regressions (existing tests still pass)
- [ ] ✅ Documentation complete (README, rustdoc)
- [ ] ✅ Code passes `cargo clippy` and `cargo fmt`
- [ ] ✅ Changes committed and pushed to `phase-4-tree-sitter` branch

---

## Troubleshooting

### Problem: Can't find tree-sitter code in vtcode-core

**Solution:**
```bash
# Search for tree-sitter usage
rg "tree_sitter" vtcode-core/src
rg "TreeSitter" vtcode-core/src

# Look for code analysis files
find vtcode-core/src -name "*analysis*"
find vtcode-core/src -name "*parse*"
```

### Problem: Tree-sitter dependencies don't compile

**Solution:**
- Check tree-sitter version compatibility
- Ensure all language bindings are same version series
- Try: `cargo update tree-sitter`

### Problem: Tests fail when tree-sitter is disabled

**Solution:**
- Ensure all tree-sitter usage is behind `#[cfg(feature = "tree-sitter")]`
- Check that disabled code path returns gracefully (None or default)
- Test: `cargo test --no-default-features`

### Problem: Merge conflict in Cargo.toml

**Solution:**
- This is expected if Agent 2 merged first
- Simply rebase: `git rebase main`
- Resolve conflict: keep both your changes and Agent 2's changes
- Continue: `git rebase --continue`

---

## Timeline Expectations

| Hour | Activity | Checkpoint |
|------|----------|------------|
| 0-0.5 | Setup, branch creation | Crate structure exists |
| 0.5-1.5 | Define API and types | API compiles |
| 1.5-3 | Move tree-sitter logic | Code moved, compiling |
| 3-3.5 | Write tests | Tests pass |
| 3.5-4 | Feature flag integration | vtcode-core works |
| 4-4.5 | Documentation | Docs complete |
| 4.5-5 | Final testing, commit | Ready to merge |

**Total: 4-5 hours**

---

## Communication

**Slack/Discord Channel:** [Insert channel name]

**Status Updates:**
- Post brief updates at each checkpoint
- Report blockers immediately
- Share design decisions that might affect others

**Format:**
```
Agent 1 Update (Hour 3):
✅ Crate structure created
✅ API defined and compiling
✅ Moving tree-sitter logic now
⏳ ETA for completion: 2 hours
```

---

## Questions?

**Before starting:**
- Clarify any questions in the kickoff meeting
- Review all three documentation files
- Confirm Cargo.toml strategy with Agent 2

**During execution:**
- Ask in the coordination channel
- Tag other agents if your question affects them
- Don't hesitate to ask for help!

---

## Final Notes

**You are Agent 1:** Your work is critical for the Phase 4 transformation. Tree-sitter is a valuable capability that many Rust projects could use - by extracting it, you're creating value beyond VTCode.

**Quality over speed:** Take the time to write good tests and documentation. This crate might be open-sourced independently.

**Have fun!** This is a great opportunity to create a clean, well-designed crate from scratch.

---

**Ready to extract vtcode-tree-sitter? Let's go! 🚀**
