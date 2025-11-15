//! Theme loading and caching for syntax highlighting.
//!
//! This module manages syntect themes with caching to avoid repeated loading.

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use syntect::highlighting::{Theme, ThemeSet};
use tracing::warn;

const MAX_THEME_CACHE_SIZE: usize = 32;

/// Global theme cache with parking_lot RwLock for concurrent access.
static THEME_CACHE: Lazy<RwLock<HashMap<String, Theme>>> = Lazy::new(|| {
    let defaults = ThemeSet::load_defaults();
    let mut entries: Vec<(String, Theme)> = defaults.themes.into_iter().collect();
    if entries.len() > MAX_THEME_CACHE_SIZE {
        entries.truncate(MAX_THEME_CACHE_SIZE);
    }
    let themes: HashMap<_, _> = entries.into_iter().collect();
    RwLock::new(themes)
});

/// Load a theme by name with caching support.
///
/// If the theme is already in the cache, returns it immediately.
/// Otherwise, loads from ThemeSet defaults and optionally caches it.
///
/// # Arguments
///
/// * `theme_name` - Name of the theme to load
/// * `cache` - Whether to cache the loaded theme
///
/// # Returns
///
/// The requested theme, or a fallback theme if not found.
pub fn load_theme(theme_name: &str, cache: bool) -> Theme {
    // Try cache first
    if let Some(theme) = THEME_CACHE.read().get(theme_name).cloned() {
        return theme;
    }

    // Load from defaults
    let defaults = ThemeSet::load_defaults();
    if let Some(theme) = defaults.themes.get(theme_name).cloned() {
        if cache {
            let mut guard = THEME_CACHE.write();
            // Evict if cache is full (simple strategy: remove first)
            if guard.len() >= MAX_THEME_CACHE_SIZE {
                if let Some(first_key) = guard.keys().next().cloned() {
                    guard.remove(&first_key);
                }
            }
            guard.insert(theme_name.to_string(), theme.clone());
        }
        theme
    } else {
        warn!(
            theme = theme_name,
            "Falling back to default syntax highlighting theme"
        );
        defaults
            .themes
            .into_iter()
            .next()
            .map(|(_, theme)| theme)
            .unwrap_or_default()
    }
}

/// List available theme names.
pub fn available_themes() -> Vec<String> {
    let theme_set = ThemeSet::load_defaults();
    theme_set.themes.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_theme() {
        let theme = load_theme("base16-ocean.dark", true);
        assert!(!theme.settings.background.is_none());
    }

    #[test]
    fn test_fallback_theme() {
        let theme = load_theme("nonexistent-theme-xyz", false);
        // Should not panic, should return a fallback
        assert!(!theme.settings.background.is_none());
    }
}
