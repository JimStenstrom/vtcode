//! Configuration merging and TOML document manipulation

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Persist configuration to a file, preserving comments in existing files
///
/// If the file already exists, this function uses `toml_edit` to preserve
/// comments and formatting. For new files, it writes a standard TOML serialization.
pub fn save_config_preserving_comments(
    path: &Path,
    config: &super::validator::VTCodeConfig,
) -> Result<()> {
    // If file exists, preserve comments by using toml_edit
    if path.exists() {
        let original_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read existing config: {}", path.display()))?;

        let mut doc = original_content
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| format!("Failed to parse existing config: {}", path.display()))?;

        // Serialize new config to TOML value
        let new_value =
            toml::to_string_pretty(config).context("Failed to serialize configuration")?;
        let new_doc: toml_edit::DocumentMut = new_value
            .parse()
            .context("Failed to parse serialized configuration")?;

        // Update values while preserving structure and comments
        merge_toml_documents(&mut doc, &new_doc);

        fs::write(path, doc.to_string())
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    } else {
        // New file, just write normally
        let content =
            toml::to_string_pretty(config).context("Failed to serialize configuration")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    }

    Ok(())
}

/// Merge TOML documents, preserving comments and structure from original
///
/// Updates `original` with values from `new` while maintaining the original's
/// comments, whitespace, and overall structure.
pub fn merge_toml_documents(original: &mut toml_edit::DocumentMut, new: &toml_edit::DocumentMut) {
    for (key, new_value) in new.iter() {
        if let Some(original_value) = original.get_mut(key) {
            merge_toml_items(original_value, new_value);
        } else {
            original[key] = new_value.clone();
        }
    }
}

/// Recursively merge TOML items
///
/// For tables, recursively merges child items. For other types, replaces the value.
pub fn merge_toml_items(original: &mut toml_edit::Item, new: &toml_edit::Item) {
    match (original, new) {
        (toml_edit::Item::Table(orig_table), toml_edit::Item::Table(new_table)) => {
            for (key, new_value) in new_table.iter() {
                if let Some(orig_value) = orig_table.get_mut(key) {
                    merge_toml_items(orig_value, new_value);
                } else {
                    orig_table[key] = new_value.clone();
                }
            }
        }
        (orig, new) => {
            *orig = new.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn save_config_preserves_comments() {
        let mut temp_file = NamedTempFile::new().expect("failed to create temp file");
        let config_with_comments = r#"# This is a test comment
[agent]
# Provider comment
provider = "openai"
default_model = "gpt-5-nano"

# Tools section comment
[tools]
default_policy = "prompt"
"#;

        write!(temp_file, "{}", config_with_comments).expect("failed to write temp config");
        temp_file.flush().expect("failed to flush");

        // Load config
        let manager = super::super::ConfigManager::load_from_file(temp_file.path())
            .expect("failed to load config");

        // Modify and save
        let mut modified_config = manager.config().clone();
        modified_config.agent.default_model = "gpt-5".to_string();

        save_config_preserving_comments(temp_file.path(), &modified_config)
            .expect("failed to save config");

        // Read back and verify comments are preserved
        let saved_content = fs::read_to_string(temp_file.path()).expect("failed to read saved config");

        assert!(
            saved_content.contains("# This is a test comment"),
            "top-level comment should be preserved"
        );
        assert!(
            saved_content.contains("# Provider comment"),
            "inline comment should be preserved"
        );
        assert!(
            saved_content.contains("# Tools section comment"),
            "section comment should be preserved"
        );
        assert!(
            saved_content.contains("gpt-5"),
            "modified value should be present"
        );
    }

    #[test]
    fn merge_toml_documents_preserves_structure() {
        let original_toml = r#"
# Original comment
[section1]
key1 = "value1"
# Keep this comment
key2 = "value2"

[section2]
nested_key = "nested_value"
"#;

        let new_toml = r#"
[section1]
key1 = "updated_value1"
key3 = "new_value3"

[section2]
nested_key = "updated_nested"
"#;

        let mut original_doc: toml_edit::DocumentMut =
            original_toml.parse().expect("failed to parse original");
        let new_doc: toml_edit::DocumentMut = new_toml.parse().expect("failed to parse new");

        merge_toml_documents(&mut original_doc, &new_doc);

        let result = original_doc.to_string();

        // Check that original comments are preserved
        assert!(result.contains("# Original comment"));
        assert!(result.contains("# Keep this comment"));

        // Check that values are updated
        assert!(result.contains("updated_value1"));
        assert!(result.contains("updated_nested"));

        // Check that new keys are added
        assert!(result.contains("key3"));
        assert!(result.contains("new_value3"));
    }

    #[test]
    fn merge_toml_items_replaces_non_table_values() {
        let mut original_doc: toml_edit::DocumentMut = r#"
[section]
key = "old_value"
"#
        .parse()
        .expect("failed to parse");

        let new_doc: toml_edit::DocumentMut = r#"
[section]
key = "new_value"
"#
        .parse()
        .expect("failed to parse");

        merge_toml_documents(&mut original_doc, &new_doc);

        let result = original_doc.to_string();
        assert!(result.contains("new_value"));
        assert!(!result.contains("old_value"));
    }
}
