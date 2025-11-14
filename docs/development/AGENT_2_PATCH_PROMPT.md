# Agent 2: Patch Extraction - Task Brief

**Your Role:** Extract vtcode-patch crate from vtcode-core
**Estimated Time:** 3-4 hours
**Branch Name:** `phase-4-patch`
**Working in Parallel With:** Agent 1 (tree-sitter extraction), Agent 3 (tools design prep)

---

## Your Mission

You are **Agent 2** in a 3-agent parallel execution of Phase 4. Your job is to extract all patch/diff functionality from the vtcode-core monolith into a new, core, independently reusable crate called **vtcode-patch**.

This extraction will:
- Remove 1,500 LOC from vtcode-core
- Create a reusable diff/patch library for Rust projects
- Power the Edit and MultiEdit tools with clean abstractions

---

## Context: The Big Picture

VTCode is transforming from a 95K LOC monolith to a modular architecture. You're in **Phase 4: Modularize Tools**.

**Current State:**
- Phase 1-2: ✅ Complete (circular deps broken, large subsystems extracted)
- Phase 3: 🔄 In progress (LLM providers being modularized)
- Phase 4: → Your task (tools modularization)

**Your Part:**
- Extract patch generation and application into standalone crate
- Make it usable by Edit and MultiEdit tools
- Provide clean diff/patch API for code modifications

---

## Required Reading

**IMPORTANT:** Read these documents before starting:

1. **docs/development/PHASE_4_AND_5_IMPLEMENTATION.md**
   - Section 4.2: Extract vtcode-patch (your detailed task)
   - Complete implementation steps
   - Code examples and API designs

2. **docs/development/ARCHITECTURE_TRANSFORMATION.md**
   - Phase 4 overview
   - Understand the target architecture

3. **docs/development/PHASE_4_PARALLELIZATION_STRATEGY.md**
   - Your role in the 3-agent strategy
   - Coordination requirements
   - Merge strategy (you merge FIRST!)

---

## Your Tasks (3-4 hours)

### ✅ Pre-Flight Checklist (15 minutes)

Before you start coding:

- [ ] Read all three documentation files above
- [ ] Attend 30-minute kickoff meeting with Agents 1 & 3
- [ ] Confirm branch name: `phase-4-patch`
- [ ] Verify communication channel is set up
- [ ] Understand Cargo.toml coordination with Agent 1

**Cargo.toml Coordination:**
You and Agent 1 will both modify `vtcode-core/Cargo.toml`. Pre-agree on this structure:

```toml
# vtcode-core/Cargo.toml

[features]
default = ["tree-sitter"]  # ← Agent 1 adds this
tree-sitter = ["vtcode-tree-sitter"]  # ← Agent 1 adds this

[dependencies]
vtcode-tree-sitter = { path = "../vtcode-tree-sitter", optional = true }  # ← Agent 1 adds this
vtcode-patch = { path = "../vtcode-patch" }  # ← YOU add this (NOT optional)
```

**You merge FIRST:** Your task is faster (3-4h vs 4-5h), so you'll create the first PR and merge. Agent 1 will rebase on your changes.

---

### Task 4.2.1: Create Crate Structure (20 minutes)

**Goal:** Set up the new vtcode-patch crate with proper structure.

```bash
# Create branch
git checkout -b phase-4-patch

# Create directory structure
mkdir -p vtcode-patch/src
mkdir -p vtcode-patch/tests
cd vtcode-patch

# Initialize crate
cat > Cargo.toml << 'EOF'
[package]
name = "vtcode-patch"
version = "0.1.0"
edition = "2021"
authors = ["VTCode Contributors"]
description = "Diff generation and patch application for code modifications"
license = "MIT OR Apache-2.0"

[dependencies]
vtcode-tool-traits = { path = "../vtcode-tool-traits" }
similar = "2.3"  # Text diffing library
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.8"
indoc = "2.0"  # For testing with indented strings
EOF

# Create basic README
cat > README.md << 'EOF'
# vtcode-patch

Diff generation and patch application for VTCode.

Provides utilities for generating diffs, creating patches, and applying code modifications.

## Features

- Diff generation (unified diff format)
- Patch creation and validation
- Multiple patch operations (replace, insert, delete, range replace)
- Fuzzy matching for resilient patching
- Indentation preservation

## Status

⚠️ Under development as part of Phase 4 extraction.
EOF

# Verify structure
tree vtcode-patch
```

**Checkpoint:** Crate structure created, Cargo.toml compiles.

---

### Task 4.2.2: Define Types and API (30 minutes)

**Goal:** Create the type definitions for patches.

**Create src/lib.rs:**

```rust
//! Diff generation and patch application for code modifications
//!
//! This crate provides utilities for:
//! - Generating diffs between old and new code
//! - Creating patch operations
//! - Applying patches to code
//! - Validating patches before application
//!
//! # Example
//!
//! ```rust
//! use vtcode_patch::{Patch, PatchOperation, PatchApplicator, PatchConfig};
//! use std::path::PathBuf;
//!
//! let config = PatchConfig::default();
//! let applicator = PatchApplicator::new(config);
//!
//! let content = "Hello, world!\nThis is a test.";
//! let patch = Patch {
//!     file_path: PathBuf::from("test.txt"),
//!     operation: PatchOperation::Replace {
//!         old_text: "world".to_string(),
//!         new_text: "Rust".to_string(),
//!         context_lines: 0,
//!     },
//! };
//!
//! let result = applicator.apply_patch(content, &patch)?;
//! assert_eq!(result.modified_content.unwrap(), "Hello, Rust!\nThis is a test.");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod types;
pub mod diff;
pub mod apply;
pub mod validation;
mod error;

pub use types::*;
pub use diff::DiffGenerator;
pub use apply::PatchApplicator;
pub use validation::{PatchValidator, ValidationResult};
pub use error::{Error, Result};
```

**Create src/types.rs:**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A patch operation representing a change to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub file_path: PathBuf,
    pub operation: PatchOperation,
}

/// Type of patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchOperation {
    /// Replace old text with new text
    Replace {
        old_text: String,
        new_text: String,
        context_lines: usize,
    },
    /// Insert text at a position
    Insert {
        position: Position,
        text: String,
    },
    /// Delete text between positions
    Delete {
        start: Position,
        end: Position,
    },
    /// Replace text in a range
    ReplaceRange {
        start: Position,
        end: Position,
        new_text: String,
    },
}

/// Position in source code (line and column)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Result of applying a patch
#[derive(Debug)]
pub struct PatchResult {
    pub success: bool,
    pub modified_content: Option<String>,
    pub error: Option<String>,
}

impl PatchResult {
    pub fn success(content: String) -> Self {
        Self {
            success: true,
            modified_content: Some(content),
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            modified_content: None,
            error: Some(error),
        }
    }
}

/// Configuration for patch operations
#[derive(Debug, Clone)]
pub struct PatchConfig {
    /// Enable fuzzy matching for replace operations
    pub fuzzy_matching: bool,
    /// Number of context lines for diffs
    pub context_lines: usize,
    /// Preserve indentation when applying patches
    pub preserve_indentation: bool,
    /// Maximum fuzz factor for fuzzy matching (0-3)
    pub max_fuzz: usize,
}

impl Default for PatchConfig {
    fn default() -> Self {
        Self {
            fuzzy_matching: false,
            context_lines: 3,
            preserve_indentation: true,
            max_fuzz: 0,
        }
    }
}
```

**Create src/error.rs:**

```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to apply patch: {0}")]
    PatchApplicationError(String),

    #[error("Old text not found in content: {0}")]
    OldTextNotFound(String),

    #[error("Invalid position: line {0}, column {1}")]
    InvalidPosition(usize, usize),

    #[error("Invalid range: start line {0} > end line {1}")]
    InvalidRange(usize, usize),

    #[error("Patch validation failed: {0}")]
    ValidationError(String),
}
```

**Checkpoint:** Run `cargo check` in vtcode-patch. Should compile.

---

### Task 4.2.3: Implement Diff Generation (1 hour)

**Goal:** Create diff generation functionality.

**Create src/diff.rs:**

```rust
use crate::types::{Patch, PatchOperation, PatchConfig};
use crate::Result;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub struct DiffGenerator {
    config: PatchConfig,
}

impl DiffGenerator {
    pub fn new(config: PatchConfig) -> Self {
        Self { config }
    }

    /// Generate a patch from old and new file contents
    pub fn generate_patch(
        &self,
        old_content: &str,
        new_content: &str,
        file_path: impl Into<PathBuf>,
    ) -> Result<Patch> {
        let diff = TextDiff::from_lines(old_content, new_content);

        // For now, create a simple replace-all patch
        // TODO: Optimize into smaller hunks
        Ok(Patch {
            file_path: file_path.into(),
            operation: PatchOperation::Replace {
                old_text: old_content.to_string(),
                new_text: new_content.to_string(),
                context_lines: self.config.context_lines,
            },
        })
    }

    /// Create a replace patch
    pub fn create_replace_patch(
        &self,
        file_path: impl Into<PathBuf>,
        old_text: &str,
        new_text: &str,
    ) -> Patch {
        Patch {
            file_path: file_path.into(),
            operation: PatchOperation::Replace {
                old_text: old_text.to_string(),
                new_text: new_text.to_string(),
                context_lines: self.config.context_lines,
            },
        }
    }

    /// Create an insert patch
    pub fn create_insert_patch(
        &self,
        file_path: impl Into<PathBuf>,
        position: crate::types::Position,
        text: &str,
    ) -> Patch {
        Patch {
            file_path: file_path.into(),
            operation: PatchOperation::Insert {
                position,
                text: text.to_string(),
            },
        }
    }

    /// Create a delete patch
    pub fn create_delete_patch(
        &self,
        file_path: impl Into<PathBuf>,
        start: crate::types::Position,
        end: crate::types::Position,
    ) -> Patch {
        Patch {
            file_path: file_path.into(),
            operation: PatchOperation::Delete { start, end },
        }
    }
}

impl Default for DiffGenerator {
    fn default() -> Self {
        Self::new(PatchConfig::default())
    }
}
```

**Checkpoint:** Run `cargo check`. Should compile.

---

### Task 4.2.4: Implement Patch Application (1 hour)

**Goal:** Implement the core patch application logic.

**Create src/apply.rs:**

```rust
use crate::types::{Patch, PatchOperation, PatchConfig, PatchResult, Position};
use crate::{Error, Result};

pub struct PatchApplicator {
    config: PatchConfig,
}

impl PatchApplicator {
    pub fn new(config: PatchConfig) -> Self {
        Self { config }
    }

    /// Apply a patch to file content
    pub fn apply_patch(
        &self,
        content: &str,
        patch: &Patch,
    ) -> Result<PatchResult> {
        match &patch.operation {
            PatchOperation::Replace { old_text, new_text, .. } => {
                self.apply_replace(content, old_text, new_text)
            }
            PatchOperation::Insert { position, text } => {
                self.apply_insert(content, *position, text)
            }
            PatchOperation::Delete { start, end } => {
                self.apply_delete(content, *start, *end)
            }
            PatchOperation::ReplaceRange { start, end, new_text } => {
                self.apply_range_replace(content, *start, *end, new_text)
            }
        }
    }

    fn apply_replace(
        &self,
        content: &str,
        old_text: &str,
        new_text: &str,
    ) -> Result<PatchResult> {
        // Find old_text in content
        if let Some(pos) = content.find(old_text) {
            let mut result = String::with_capacity(
                content.len() - old_text.len() + new_text.len()
            );
            result.push_str(&content[..pos]);
            result.push_str(new_text);
            result.push_str(&content[pos + old_text.len()..]);

            Ok(PatchResult::success(result))
        } else if self.config.fuzzy_matching {
            // Try fuzzy matching
            self.fuzzy_replace(content, old_text, new_text)
        } else {
            Err(Error::OldTextNotFound(
                old_text.chars().take(50).collect::<String>() + "..."
            ))
        }
    }

    fn fuzzy_replace(
        &self,
        content: &str,
        old_text: &str,
        new_text: &str,
    ) -> Result<PatchResult> {
        // Simple fuzzy matching: ignore leading/trailing whitespace
        let old_trimmed = old_text.trim();

        // Search for trimmed version
        if let Some(pos) = content.find(old_trimmed) {
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..pos]);
            result.push_str(new_text);
            result.push_str(&content[pos + old_trimmed.len()..]);

            Ok(PatchResult::success(result))
        } else {
            Err(Error::OldTextNotFound(
                old_text.chars().take(50).collect::<String>() + "..."
            ))
        }
    }

    fn apply_insert(
        &self,
        content: &str,
        position: Position,
        text: &str,
    ) -> Result<PatchResult> {
        let lines: Vec<&str> = content.lines().collect();

        if position.line >= lines.len() {
            return Err(Error::InvalidPosition(position.line, position.column));
        }

        let mut result = String::new();

        // Copy lines before insertion point
        for (i, line) in lines.iter().enumerate() {
            if i == position.line {
                // Insert at column position
                let line_chars: Vec<char> = line.chars().collect();
                if position.column > line_chars.len() {
                    return Err(Error::InvalidPosition(position.line, position.column));
                }

                result.push_str(&line_chars[..position.column].iter().collect::<String>());
                result.push_str(text);
                result.push_str(&line_chars[position.column..].iter().collect::<String>());
            } else {
                result.push_str(line);
            }

            if i < lines.len() - 1 || content.ends_with('\n') {
                result.push('\n');
            }
        }

        Ok(PatchResult::success(result))
    }

    fn apply_delete(
        &self,
        content: &str,
        start: Position,
        end: Position,
    ) -> Result<PatchResult> {
        if start.line > end.line || (start.line == end.line && start.column > end.column) {
            return Err(Error::InvalidRange(start.line, end.line));
        }

        let lines: Vec<&str> = content.lines().collect();

        if end.line >= lines.len() {
            return Err(Error::InvalidPosition(end.line, end.column));
        }

        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i < start.line || i > end.line {
                // Lines outside the deletion range
                result.push_str(line);
                if i < lines.len() - 1 {
                    result.push('\n');
                }
            } else if i == start.line && i == end.line {
                // Deletion within a single line
                let line_chars: Vec<char> = line.chars().collect();
                result.push_str(&line_chars[..start.column].iter().collect::<String>());
                result.push_str(&line_chars[end.column..].iter().collect::<String>());
                if i < lines.len() - 1 {
                    result.push('\n');
                }
            } else if i == start.line {
                // Start of multi-line deletion
                let line_chars: Vec<char> = line.chars().collect();
                result.push_str(&line_chars[..start.column].iter().collect::<String>());
            } else if i == end.line {
                // End of multi-line deletion
                let line_chars: Vec<char> = line.chars().collect();
                result.push_str(&line_chars[end.column..].iter().collect::<String>());
                if i < lines.len() - 1 {
                    result.push('\n');
                }
            }
            // else: middle lines are skipped (deleted)
        }

        Ok(PatchResult::success(result))
    }

    fn apply_range_replace(
        &self,
        content: &str,
        start: Position,
        end: Position,
        new_text: &str,
    ) -> Result<PatchResult> {
        // Delete the range first
        let delete_result = self.apply_delete(content, start, end)?;

        // Then insert at the start position
        let content_after_delete = delete_result.modified_content.unwrap();
        self.apply_insert(&content_after_delete, start, new_text)
    }
}

impl Default for PatchApplicator {
    fn default() -> Self {
        Self::new(PatchConfig::default())
    }
}
```

**Checkpoint:** Run `cargo check`. Should compile.

---

### Task 4.2.5: Add Validation (30 minutes)

**Goal:** Implement patch validation.

**Create src/validation.rs:**

```rust
use crate::types::{Patch, PatchOperation, Position};

pub struct PatchValidator;

impl PatchValidator {
    /// Validate that a patch can be applied
    pub fn validate(content: &str, patch: &Patch) -> ValidationResult {
        match &patch.operation {
            PatchOperation::Replace { old_text, .. } => {
                Self::validate_replace(content, old_text)
            }
            PatchOperation::Insert { position, .. } => {
                Self::validate_position(content, *position)
            }
            PatchOperation::Delete { start, end } => {
                Self::validate_range(content, *start, *end)
            }
            PatchOperation::ReplaceRange { start, end, .. } => {
                Self::validate_range(content, *start, *end)
            }
        }
    }

    fn validate_replace(content: &str, old_text: &str) -> ValidationResult {
        let occurrences = content.matches(old_text).count();

        ValidationResult {
            valid: occurrences > 0,
            warnings: if occurrences > 1 {
                vec![format!("Old text appears {} times in file", occurrences)]
            } else {
                vec![]
            },
            errors: if occurrences == 0 {
                vec!["Old text not found in file".to_string()]
            } else {
                vec![]
            },
        }
    }

    fn validate_position(content: &str, position: Position) -> ValidationResult {
        let lines: Vec<&str> = content.lines().collect();

        if position.line >= lines.len() {
            return ValidationResult {
                valid: false,
                warnings: vec![],
                errors: vec![format!(
                    "Line {} does not exist (file has {} lines)",
                    position.line,
                    lines.len()
                )],
            };
        }

        let line = lines[position.line];
        let line_len = line.chars().count();

        if position.column > line_len {
            return ValidationResult {
                valid: false,
                warnings: vec![],
                errors: vec![format!(
                    "Column {} exceeds line length {}",
                    position.column, line_len
                )],
            };
        }

        ValidationResult {
            valid: true,
            warnings: vec![],
            errors: vec![],
        }
    }

    fn validate_range(content: &str, start: Position, end: Position) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            warnings: vec![],
            errors: vec![],
        };

        // Validate start position
        let start_result = Self::validate_position(content, start);
        if !start_result.valid {
            result.valid = false;
            result.errors.extend(start_result.errors);
            return result;
        }

        // Validate end position
        let end_result = Self::validate_position(content, end);
        if !end_result.valid {
            result.valid = false;
            result.errors.extend(end_result.errors);
            return result;
        }

        // Validate range order
        if start.line > end.line || (start.line == end.line && start.column > end.column) {
            result.valid = false;
            result.errors.push(format!(
                "Invalid range: start ({},{}) comes after end ({},{})",
                start.line, start.column, end.line, end.column
            ));
        }

        result
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
```

**Checkpoint:** Run `cargo check`. Should compile.

---

### Task 4.2.6: Update vtcode-core Tools (45 minutes)

**Goal:** Make Edit and MultiEdit tools use the new vtcode-patch crate.

**Step 1: Add dependency to vtcode-core/Cargo.toml**

```toml
[dependencies]
vtcode-patch = { path = "../vtcode-patch" }
```

**Step 2: Update Edit tool (vtcode-core/src/tools/edit.rs)**

Find the Edit tool implementation and update it:

```rust
use vtcode_patch::{Patch, PatchOperation, PatchApplicator, PatchConfig};
use std::path::PathBuf;

pub struct EditTool {
    applicator: PatchApplicator,
}

impl EditTool {
    pub fn new() -> Self {
        let config = PatchConfig {
            fuzzy_matching: false,
            context_lines: 3,
            preserve_indentation: true,
            max_fuzz: 0,
        };

        Self {
            applicator: PatchApplicator::new(config),
        }
    }

    pub async fn execute(&self, request: EditRequest) -> Result<EditResponse> {
        // Read file
        let content = tokio::fs::read_to_string(&request.file_path).await?;

        // Create patch
        let patch = Patch {
            file_path: request.file_path.clone(),
            operation: PatchOperation::Replace {
                old_text: request.old_string,
                new_text: request.new_string,
                context_lines: 3,
            },
        };

        // Apply patch
        let result = self.applicator.apply_patch(&content, &patch)?;

        if result.success {
            // Write modified content
            if let Some(new_content) = result.modified_content {
                tokio::fs::write(&request.file_path, new_content).await?;
            }
        }

        Ok(EditResponse {
            success: result.success,
            error: result.error,
        })
    }
}
```

**Step 3: Remove old patch code from vtcode-core**

```bash
# Find and remove old patch implementation
# (it might be in vtcode-core/src/tools/ or vtcode-core/src/patch/)
git rm -r vtcode-core/src/patch/  # if this directory exists
```

**Step 4: Test the integration**

```bash
cd vtcode-core
cargo test --test edit_tool_test
```

**Checkpoint:** Edit tool uses vtcode-patch and tests pass.

---

### Task 4.2.7: Write Comprehensive Tests (30 minutes)

**Goal:** Ensure vtcode-patch works correctly.

**Create tests/integration_tests.rs:**

```rust
use vtcode_patch::*;
use std::path::PathBuf;
use indoc::indoc;

#[test]
fn test_simple_replace() {
    let config = PatchConfig::default();
    let applicator = PatchApplicator::new(config);

    let content = "Hello, world!\nThis is a test.";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Replace {
            old_text: "world".to_string(),
            new_text: "Rust".to_string(),
            context_lines: 0,
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();

    assert!(result.success);
    assert_eq!(result.modified_content.unwrap(), "Hello, Rust!\nThis is a test.");
}

#[test]
fn test_multiline_replace() {
    let applicator = PatchApplicator::default();

    let content = indoc! {"
        fn main() {
            println!(\"Hello\");
        }
    "};

    let patch = Patch {
        file_path: PathBuf::from("test.rs"),
        operation: PatchOperation::Replace {
            old_text: "println!(\"Hello\");".to_string(),
            new_text: "println!(\"Goodbye\");".to_string(),
            context_lines: 0,
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();
    assert!(result.success);
    assert!(result.modified_content.unwrap().contains("Goodbye"));
}

#[test]
fn test_insert_at_position() {
    let applicator = PatchApplicator::default();

    let content = "Line 1\nLine 2\nLine 3";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Insert {
            position: Position::new(1, 0),
            text: "INSERTED ".to_string(),
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();
    assert!(result.success);

    let lines: Vec<&str> = result.modified_content.unwrap().lines().collect();
    assert_eq!(lines[1], "INSERTED Line 2");
}

#[test]
fn test_delete_range() {
    let applicator = PatchApplicator::default();

    let content = "Line 1\nLine 2\nLine 3\nLine 4";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Delete {
            start: Position::new(1, 0),
            end: Position::new(2, 6),
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();
    assert!(result.success);

    let modified = result.modified_content.unwrap();
    assert!(modified.contains("Line 1"));
    assert!(modified.contains("Line 4"));
    assert!(!modified.contains("Line 2"));
}

#[test]
fn test_replace_range() {
    let applicator = PatchApplicator::default();

    let content = "fn old_name() {}";
    let patch = Patch {
        file_path: PathBuf::from("test.rs"),
        operation: PatchOperation::ReplaceRange {
            start: Position::new(0, 3),
            end: Position::new(0, 11),
            new_text: "new_name".to_string(),
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();
    assert!(result.success);
    assert_eq!(result.modified_content.unwrap(), "fn new_name() {}");
}

#[test]
fn test_validation_old_text_not_found() {
    let content = "Hello, world!";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Replace {
            old_text: "goodbye".to_string(),
            new_text: "hi".to_string(),
            context_lines: 0,
        },
    };

    let validation = PatchValidator::validate(content, &patch);
    assert!(!validation.valid);
    assert!(!validation.errors.is_empty());
}

#[test]
fn test_validation_invalid_position() {
    let content = "Line 1\nLine 2";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Insert {
            position: Position::new(10, 0),
            text: "test".to_string(),
        },
    };

    let validation = PatchValidator::validate(content, &patch);
    assert!(!validation.valid);
}

#[test]
fn test_fuzzy_matching() {
    let config = PatchConfig {
        fuzzy_matching: true,
        context_lines: 3,
        preserve_indentation: true,
        max_fuzz: 1,
    };
    let applicator = PatchApplicator::new(config);

    let content = "  Hello, world!  ";
    let patch = Patch {
        file_path: PathBuf::from("test.txt"),
        operation: PatchOperation::Replace {
            old_text: "Hello, world!".to_string(),  // No spaces
            new_text: "Hi, Rust!".to_string(),
            context_lines: 0,
        },
    };

    let result = applicator.apply_patch(content, &patch).unwrap();
    assert!(result.success);
}
```

**Run tests:**

```bash
cd vtcode-patch
cargo test
```

**Checkpoint:** All tests pass.

---

### Task 4.2.8: Documentation and Final Polish (30 minutes)

**Goal:** Complete documentation.

**Update vtcode-patch/README.md:**

```markdown
# vtcode-patch

Diff generation and patch application for code modifications in VTCode.

## Features

- **Multiple patch operations**: Replace, Insert, Delete, ReplaceRange
- **Fuzzy matching**: Resilient patching with whitespace tolerance
- **Validation**: Pre-validate patches before application
- **Diff generation**: Create patches from old/new content
- **Indentation preservation**: Maintains code formatting

## Installation

```toml
[dependencies]
vtcode-patch = "0.1"
```

## Usage

### Basic Replace

```rust
use vtcode_patch::*;

let applicator = PatchApplicator::default();

let content = "Hello, world!";
let patch = Patch {
    file_path: "test.txt".into(),
    operation: PatchOperation::Replace {
        old_text: "world".to_string(),
        new_text: "Rust".to_string(),
        context_lines: 0,
    },
};

let result = applicator.apply_patch(content, &patch)?;
if result.success {
    println!("Patched: {}", result.modified_content.unwrap());
}
```

### Generate Diff

```rust
let generator = DiffGenerator::default();

let old_code = "fn old_name() {}";
let new_code = "fn new_name() {}";

let patch = generator.generate_patch(old_code, new_code, "example.rs")?;
```

### Validate Before Applying

```rust
let validation = PatchValidator::validate(content, &patch);

if !validation.valid {
    for error in &validation.errors {
        eprintln!("Error: {}", error);
    }
}
```

### Fuzzy Matching

```rust
let config = PatchConfig {
    fuzzy_matching: true,
    max_fuzz: 1,
    ..Default::default()
};

let applicator = PatchApplicator::new(config);
// Will match old_text even if whitespace differs
```

## Integration with VTCode

Used by:
- Edit tool (single file modifications)
- MultiEdit tool (batch modifications)
- Refactoring tools

## License

MIT OR Apache-2.0
```

**Checkpoint:** Documentation complete.

---

### Task 4.2.9: Final Testing and Commit (30 minutes)

**Goal:** Verify everything works and commit.

**Final testing:**

```bash
# Test vtcode-patch
cd vtcode-patch
cargo clean
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# Test vtcode-core with patch
cd ../vtcode-core
cargo test --test edit_tool_test
cargo test --test multi_edit_tool_test

# Test workspace
cd ..
cargo test --all
```

**Commit and push:**

```bash
git add .
git commit -m "feat(phase4): Extract vtcode-patch crate

Extract diff/patch functionality from vtcode-core into standalone
vtcode-patch crate.

Changes:
- Created vtcode-patch crate with diff/patch operations
- Support for Replace, Insert, Delete, ReplaceRange operations
- Fuzzy matching for resilient patching
- Patch validation before application
- Updated Edit and MultiEdit tools to use vtcode-patch
- Removed 1,500 LOC from vtcode-core
- Full test coverage for all patch operations
- Comprehensive documentation

Testing:
- All vtcode-patch tests pass
- Edit and MultiEdit tools work correctly
- No regressions in existing functionality

Part of Phase 4: Modularize Tools"

git push -u origin phase-4-patch
```

---

## Coordination Points

### With Agent 1 (Tree-sitter Extraction)

**Cargo.toml Coordination:**
- ✅ You add `vtcode-patch` dependency
- ✅ Agent 1 adds `tree-sitter` feature and dependency
- ✅ Pre-coordinate structure in kickoff meeting

**You Merge FIRST:**
- Your task is faster (3-4h vs 4-5h)
- Create PR when done
- Merge to main
- Agent 1 will rebase on your changes

**Checkpoints:**
- Hour 1: Share progress
- Hour 3: Share that you're testing
- Hour 3.5: Notify you're ready to merge

### With Agent 3 (Tools Enhancement)

**Communication:**
- Agent 3 needs your patch API to be stable
- Share your public API once finalized (hour 2)
- Agent 3 will use vtcode-patch in the tool registry

---

## Success Criteria

Your task is complete when:

- [ ] ✅ vtcode-patch crate exists and compiles
- [ ] ✅ All patch operations implemented (Replace, Insert, Delete, ReplaceRange)
- [ ] ✅ Fuzzy matching works
- [ ] ✅ Validation implemented
- [ ] ✅ All tests pass (100% core functionality)
- [ ] ✅ Edit tool uses vtcode-patch
- [ ] ✅ MultiEdit tool uses vtcode-patch
- [ ] ✅ No regressions
- [ ] ✅ Documentation complete
- [ ] ✅ Code passes clippy and fmt
- [ ] ✅ Committed and pushed to `phase-4-patch`
- [ ] ✅ **MERGED TO MAIN FIRST**

---

## Timeline

| Hour | Activity | Checkpoint |
|------|----------|------------|
| 0-0.3 | Setup, crate creation | Structure exists |
| 0.3-1 | Define types and API | Types compile |
| 1-2 | Implement diff generation | Diff works |
| 2-3 | Implement patch application | Apply works |
| 3-3.5 | Add validation | Validation works |
| 3.5-4 | Update tools, tests | Integration done |
| 4-4.5 | Documentation, final test | Ready to merge |

**Total: 3-4 hours**

---

## Questions?

Ask in the coordination channel or the kickoff meeting!

---

**Ready to extract vtcode-patch? You've got this! 🚀**
