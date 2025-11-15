//! Search operations

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;

use crate::{FileIndex, SearchResult};

/// Search files using regex pattern.
pub fn search(
    index_cache: &HashMap<String, FileIndex>,
    pattern: &str,
    path_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    search_with_matcher(index_cache, pattern, path_filter, |regex, line| {
        regex
            .find_iter(line)
            .map(|m| m.as_str().to_string())
            .collect()
    })
}

/// Grep-like search (like grep command).
pub fn grep(
    index_cache: &HashMap<String, FileIndex>,
    pattern: &str,
    file_pattern: Option<&str>,
) -> Result<Vec<SearchResult>> {
    search_with_matcher(index_cache, pattern, file_pattern, |_regex, line| {
        vec![line.to_string()]
    })
}

/// Find files by name pattern.
pub fn find_files(index_cache: &HashMap<String, FileIndex>, pattern: &str) -> Result<Vec<String>> {
    let regex = Regex::new(pattern)?;
    let mut results = Vec::new();

    for file_path in index_cache.keys() {
        if regex.is_match(file_path) {
            results.push(file_path.clone());
        }
    }

    Ok(results)
}

/// Get all indexed files without pattern matching.
/// This is more efficient than using find_files(".*").
pub fn all_files(index_cache: &HashMap<String, FileIndex>) -> Vec<String> {
    index_cache.keys().cloned().collect()
}

/// Get file content with line numbers.
pub fn get_file_content(
    file_path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<String> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let start = start_line.unwrap_or(1).saturating_sub(1);
    let end = end_line.unwrap_or(lines.len());

    let selected_lines = &lines[start..end.min(lines.len())];

    let mut result = String::new();
    for (i, line) in selected_lines.iter().enumerate() {
        result.push_str(&format!("{}: {}\n", start + i + 1, line));
    }

    Ok(result)
}

/// List files in directory (like ls).
pub fn list_files(dir_path: &str, show_hidden: bool) -> Result<Vec<String>> {
    use std::path::Path;

    let path = Path::new(dir_path);
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && crate::filter::is_path_hidden(&entry_path) {
            continue;
        }

        files.push(file_name);
    }

    Ok(files)
}

/// Common search implementation used by both search() and grep().
fn search_with_matcher<F>(
    index_cache: &HashMap<String, FileIndex>,
    pattern: &str,
    path_filter: Option<&str>,
    extract_matches: F,
) -> Result<Vec<SearchResult>>
where
    F: Fn(&Regex, &str) -> Vec<String>,
{
    let regex = Regex::new(pattern)?;
    let mut results = Vec::new();

    for file_path in index_cache.keys() {
        if path_filter.is_some_and(|filter| !file_path.contains(filter)) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(file_path) {
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    let matches = extract_matches(&regex, line);

                    results.push(SearchResult {
                        file_path: file_path.clone(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        matches,
                    });
                }
            }
        }
    }

    Ok(results)
}
