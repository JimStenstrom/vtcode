//! Integration tests for SOP (Standard Operating Procedure) loading

use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use vtcode_rag::{
    Embedder, IndexingPipeline, MockEmbedder, QueryPipeline, SemanticChunker, load_sops_from_dir,
};
use vtcode_vectordb::{
    InMemoryVectorDb,
    types::{Condition, Filter},
};

#[tokio::test]
async fn test_load_sops_from_directory() {
    // Create temp directory with SOPs
    let temp_dir = TempDir::new().unwrap();
    let sop_dir = temp_dir.path().join(".vtcode/sops");
    fs::create_dir_all(&sop_dir).unwrap();

    // Create git-commit SOP
    let git_commit_sop = r#"---
type: standard-operating-procedure
id: git-commit
---
# Git Commit SOP
This is a test procedure for creating git commits.
"#;
    fs::write(sop_dir.join("git-commit.md"), git_commit_sop).unwrap();

    // Create test-before-commit SOP
    let test_sop = r#"---
type: standard-operating-procedure
id: test-before-commit
---
# Test Before Commit
Always run tests before committing code.
"#;
    fs::write(sop_dir.join("test-before-commit.md"), test_sop).unwrap();

    // Load SOPs
    let sops = load_sops_from_dir(&sop_dir).unwrap();

    // Verify count
    assert_eq!(sops.len(), 2);

    // Verify metadata
    for sop in &sops {
        assert_eq!(sop.metadata["document_type"], "sop");
        assert!(sop.metadata.get("sop_id").is_some());
        assert!(sop.metadata.get("file_path").is_some());
    }

    // Verify specific SOP
    let git_commit = sops.iter().find(|s| s.id == "sop:git-commit").unwrap();
    assert!(git_commit.content.contains("Git Commit SOP"));
    assert_eq!(git_commit.metadata["sop_id"], "git-commit");
}

#[tokio::test]
async fn test_sop_indexing_and_retrieval() {
    // Create temp directory with SOPs
    let temp_dir = TempDir::new().unwrap();
    let sop_dir = temp_dir.path().join(".vtcode/sops");
    fs::create_dir_all(&sop_dir).unwrap();

    // Create SOPs
    let commit_sop = r#"---
id: git-commit
---
# Creating Git Commits
Run git status, git diff, and git log before committing.
Use meaningful commit messages.
"#;
    fs::write(sop_dir.join("git-commit.md"), commit_sop).unwrap();

    let error_sop = r#"---
id: error-handling
---
# Error Handling
Use anyhow::Result for application code.
Add context to errors with .with_context().
"#;
    fs::write(sop_dir.join("error-handling.md"), error_sop).unwrap();

    // Load SOPs
    let sops = load_sops_from_dir(&sop_dir).unwrap();
    assert_eq!(sops.len(), 2);

    // Create indexing pipeline
    let vectordb = Arc::new(InMemoryVectorDb::new());
    let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::new(384));
    let chunker = Box::new(SemanticChunker::default());

    let indexing_pipeline = IndexingPipeline::new(
        vectordb.clone(),
        embedder.clone(),
        chunker,
        "test_sops".to_string(),
    );

    // Index SOPs
    indexing_pipeline
        .index_documents(sops)
        .await
        .expect("Failed to index SOPs");

    // Create query pipeline
    let query_pipeline = QueryPipeline::new(vectordb, embedder, "test_sops".to_string());

    // Test semantic search with SOP filter
    let filter = Filter {
        must: vec![Condition::Match {
            key: "document_type".to_string(),
            value: serde_json::json!("sop"),
        }],
        must_not: vec![],
        should: vec![],
    };

    // Query for git-related SOPs
    let results = query_pipeline
        .retrieve("how to commit code", 5, Some(filter.clone()))
        .await
        .expect("Failed to retrieve SOPs");

    assert!(!results.is_empty(), "Should find at least one SOP");
    // Check that at least one result is git-related (MockEmbedder produces deterministic but not semantic vectors)
    let has_git_related = results.iter().any(|r|
        r.content.contains("Git Commit") || r.content.contains("commit")
    );
    assert!(has_git_related, "Should find at least one git-related SOP");

    // Query for error handling
    let error_results = query_pipeline
        .retrieve("error handling in rust", 5, Some(filter))
        .await
        .expect("Failed to retrieve error handling SOP");

    assert!(!error_results.is_empty());
    // Check that at least one result is error-handling related
    let has_error_handling = error_results.iter().any(|r|
        r.content.contains("Error Handling") || r.content.contains("anyhow")
    );
    assert!(has_error_handling, "Should find at least one error handling SOP");
}

#[test]
fn test_sop_frontmatter_parsing() {
    let temp_dir = TempDir::new().unwrap();

    // SOP with frontmatter
    let with_frontmatter = r#"---
type: standard-operating-procedure
id: my-custom-id
---
# SOP Content
"#;
    fs::write(temp_dir.path().join("custom.md"), with_frontmatter).unwrap();

    // SOP without frontmatter
    let without_frontmatter = "# Simple SOP\nNo frontmatter here.";
    fs::write(temp_dir.path().join("simple.md"), without_frontmatter).unwrap();

    let sops = load_sops_from_dir(temp_dir.path()).unwrap();
    assert_eq!(sops.len(), 2);

    // Check custom ID
    let custom = sops.iter().find(|s| s.id == "sop:my-custom-id").unwrap();
    assert_eq!(custom.metadata["sop_id"], "my-custom-id");
    assert_eq!(custom.metadata["filename"], "custom");

    // Check filename-based ID
    let simple = sops.iter().find(|s| s.id == "sop:simple").unwrap();
    assert_eq!(simple.metadata["sop_id"], "simple");
    assert_eq!(simple.metadata["filename"], "simple");
}

#[test]
fn test_sop_directory_depth() {
    let temp_dir = TempDir::new().unwrap();

    // Root level
    fs::write(temp_dir.path().join("root.md"), "# Root SOP").unwrap();

    // One level deep
    let sub1 = temp_dir.path().join("category");
    fs::create_dir(&sub1).unwrap();
    fs::write(sub1.join("level1.md"), "# Level 1 SOP").unwrap();

    // Two levels deep
    let sub2 = sub1.join("subcategory");
    fs::create_dir(&sub2).unwrap();
    fs::write(sub2.join("level2.md"), "# Level 2 SOP").unwrap();

    // Three levels deep
    let sub3 = sub2.join("deep");
    fs::create_dir(&sub3).unwrap();
    fs::write(sub3.join("level3.md"), "# Level 3 SOP").unwrap();

    // Four levels deep (should be ignored by max_depth=4)
    let sub4 = sub3.join("deeper");
    fs::create_dir(&sub4).unwrap();
    fs::write(sub4.join("level4.md"), "# Level 4 SOP").unwrap();

    let sops = load_sops_from_dir(temp_dir.path()).unwrap();

    // Should find root, level1, level2, and level3, but not level4
    assert_eq!(sops.len(), 4);

    let ids: Vec<_> = sops.iter().map(|s| s.metadata["filename"].as_str().unwrap()).collect();
    assert!(ids.contains(&"root"));
    assert!(ids.contains(&"level1"));
    assert!(ids.contains(&"level2"));
    assert!(ids.contains(&"level3"));
    assert!(!ids.contains(&"level4"));
}

#[test]
fn test_load_from_nonexistent_directory() {
    let sops = load_sops_from_dir(std::path::Path::new("/nonexistent/directory")).unwrap();
    assert_eq!(sops.len(), 0, "Should return empty vec for nonexistent directory");
}

#[test]
fn test_ignore_non_markdown_files() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("valid.md"), "# Valid SOP").unwrap();
    fs::write(temp_dir.path().join("readme.txt"), "Not a SOP").unwrap();
    fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
    fs::write(temp_dir.path().join("script.sh"), "#!/bin/bash").unwrap();

    let sops = load_sops_from_dir(temp_dir.path()).unwrap();

    assert_eq!(sops.len(), 1, "Should only load .md files");
    assert_eq!(sops[0].metadata["filename"], "valid");
}

#[tokio::test]
async fn test_multiple_sop_directories() {
    // Simulate loading from multiple SOP paths
    let temp_dir = TempDir::new().unwrap();

    // Global SOPs
    let global_dir = temp_dir.path().join(".vtcode/sops");
    fs::create_dir_all(&global_dir).unwrap();
    fs::write(global_dir.join("global.md"), "# Global SOP").unwrap();

    // Project-specific SOPs
    let project_dir = temp_dir.path().join("docs/procedures");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("project.md"), "# Project SOP").unwrap();

    // Load from both
    let mut all_sops = Vec::new();
    all_sops.extend(load_sops_from_dir(&global_dir).unwrap());
    all_sops.extend(load_sops_from_dir(&project_dir).unwrap());

    assert_eq!(all_sops.len(), 2);

    let ids: Vec<_> = all_sops.iter().map(|s| s.metadata["filename"].as_str().unwrap()).collect();
    assert!(ids.contains(&"global"));
    assert!(ids.contains(&"project"));
}
