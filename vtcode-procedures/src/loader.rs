//! Procedure document loader
//!
//! Loads procedure markdown files from directories and converts them to RAG Documents.
//! Procedures are simple markdown files with optional YAML frontmatter.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use vtcode_rag::types::Document;
use walkdir::WalkDir;

/// Load procedures from a directory
///
/// Scans the directory for `.md` files (max depth 4) and converts them to Documents
/// with `document_type: "procedure"` metadata.
///
/// # Frontmatter Format
///
/// ```yaml
/// ---
/// type: standard-operating-procedure
/// id: git-commit
/// ---
/// ```
///
/// # Arguments
///
/// * `dir` - Directory to scan for `.md` files
///
/// # Returns
///
/// Vector of Documents representing procedures
///
/// # Example
///
/// ```no_run
/// use vtcode_procedures::load_procedures_from_dir;
/// use std::path::Path;
///
/// let procedures = load_procedures_from_dir(Path::new("docs/procedures")).unwrap();
/// println!("Loaded {} procedures", procedures.len());
/// ```
pub fn load_procedures_from_dir(dir: &Path) -> Result<Vec<Document>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut documents = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process .md files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read procedure file at {}", path.display())
        })?;

        // Extract ID from filename (without .md extension)
        let filename_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse frontmatter for explicit procedure ID (if present)
        let procedure_id =
            extract_frontmatter_id(&content).unwrap_or_else(|| filename_id.clone());

        // Create document with procedure metadata
        let doc = Document::new(format!("procedure:{}", procedure_id), content).with_metadata(
            serde_json::json!({
                "document_type": "procedure",
                "procedure_id": procedure_id,
                "file_path": path.to_string_lossy(),
                "filename": filename_id,
            }),
        );

        documents.push(doc);
    }

    Ok(documents)
}

/// Extract ID from YAML frontmatter
///
/// Looks for simple `id: value` pattern in frontmatter.
/// Does not require full YAML parser - just simple line matching.
///
/// # Format
///
/// ```yaml
/// ---
/// id: my-procedure-id
/// type: standard-operating-procedure
/// ---
/// ```
fn extract_frontmatter_id(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    // Find end of frontmatter
    let rest = &content[3..];
    let end_pos = rest.find("\n---")?;
    let frontmatter = &rest[..end_pos];

    // Simple YAML parsing - just look for "id: value"
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(id_val) = trimmed.strip_prefix("id:") {
            let id = id_val.trim().to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_frontmatter_id() {
        let content = r#"---
type: standard-operating-procedure
id: git-commit
---
# Git Commit Procedure
"#;
        assert_eq!(
            extract_frontmatter_id(content),
            Some("git-commit".to_string())
        );
    }

    #[test]
    fn test_extract_frontmatter_id_no_frontmatter() {
        let content = "# Just a regular markdown file";
        assert_eq!(extract_frontmatter_id(content), None);
    }

    #[test]
    fn test_extract_frontmatter_id_missing_id() {
        let content = r#"---
type: standard-operating-procedure
---
# Procedure without ID
"#;
        assert_eq!(extract_frontmatter_id(content), None);
    }

    #[test]
    fn test_load_procedures_from_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let procedures = load_procedures_from_dir(temp_dir.path()).unwrap();
        assert_eq!(procedures.len(), 0);
    }

    #[test]
    fn test_load_procedures_from_nonexistent_dir() {
        let procedures = load_procedures_from_dir(Path::new("/nonexistent/path")).unwrap();
        assert_eq!(procedures.len(), 0);
    }

    #[test]
    fn test_load_procedures_with_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let procedure_content = r#"---
type: standard-operating-procedure
id: test-procedure
---
# Test Procedure
This is a test procedure.
"#;
        fs::write(temp_dir.path().join("test.md"), procedure_content).unwrap();

        let procedures = load_procedures_from_dir(temp_dir.path()).unwrap();
        assert_eq!(procedures.len(), 1);

        let proc = &procedures[0];
        assert_eq!(proc.id, "procedure:test-procedure");
        assert_eq!(proc.metadata["document_type"], "procedure");
        assert_eq!(proc.metadata["procedure_id"], "test-procedure");
        assert_eq!(proc.metadata["filename"], "test");
    }

    #[test]
    fn test_load_procedures_without_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let procedure_content = "# Simple Procedure\nJust markdown content.";
        fs::write(temp_dir.path().join("simple.md"), procedure_content).unwrap();

        let procedures = load_procedures_from_dir(temp_dir.path()).unwrap();
        assert_eq!(procedures.len(), 1);

        let proc = &procedures[0];
        assert_eq!(proc.id, "procedure:simple"); // Uses filename
        assert_eq!(proc.metadata["procedure_id"], "simple");
    }

    #[test]
    fn test_load_procedures_ignores_non_markdown() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.md"), "# Valid Procedure").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "Not a procedure").unwrap();
        fs::write(temp_dir.path().join("test.json"), "{}").unwrap();

        let procedures = load_procedures_from_dir(temp_dir.path()).unwrap();
        assert_eq!(procedures.len(), 1); // Only .md file
    }

    #[test]
    fn test_load_procedures_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Root level
        fs::write(temp_dir.path().join("root.md"), "# Root").unwrap();

        // One level deep
        let sub1 = temp_dir.path().join("sub1");
        fs::create_dir(&sub1).unwrap();
        fs::write(sub1.join("level1.md"), "# Level 1").unwrap();

        // Two levels deep
        let sub2 = sub1.join("sub2");
        fs::create_dir(&sub2).unwrap();
        fs::write(sub2.join("level2.md"), "# Level 2").unwrap();

        // Three levels deep
        let sub3 = sub2.join("sub3");
        fs::create_dir(&sub3).unwrap();
        fs::write(sub3.join("level3.md"), "# Level 3").unwrap();

        // Four levels deep (at max_depth=4)
        let sub4 = sub3.join("sub4");
        fs::create_dir(&sub4).unwrap();
        fs::write(sub4.join("level4.md"), "# Level 4").unwrap();

        let procedures = load_procedures_from_dir(temp_dir.path()).unwrap();
        assert_eq!(procedures.len(), 4); // root, level1, level2, level3 (level4 excluded by max_depth)
    }
}
