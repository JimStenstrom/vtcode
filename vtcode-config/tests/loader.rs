use anyhow::Result;
use assert_fs::TempDir;
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vtcode_commons::paths::WorkspacePaths;
use vtcode_config::ConfigManager;
use vtcode_config::constants::defaults;
use vtcode_config::defaults::provider::with_config_defaults_provider_for_test;
use vtcode_config::defaults::{ConfigDefaultsProvider, WorkspacePathsDefaults};

#[derive(Clone)]
struct TestPaths {
    root: PathBuf,
    config_dir: PathBuf,
}

impl TestPaths {
    fn new(root: PathBuf, config_dir: PathBuf) -> Self {
        Self { root, config_dir }
    }
}

impl WorkspacePaths for TestPaths {
    fn workspace_root(&self) -> &Path {
        &self.root
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
}

fn with_test_defaults<T>(
    workspace_root: &Path,
    config_dir: PathBuf,
    home_paths: Vec<PathBuf>,
    action: impl FnOnce() -> T,
) -> T {
    let workspace_paths = TestPaths::new(workspace_root.to_path_buf(), config_dir);
    let provider = WorkspacePathsDefaults::new(Arc::new(workspace_paths))
        .with_home_paths(home_paths)
        .build();
    let provider: Arc<dyn ConfigDefaultsProvider> = provider.into();

    with_config_defaults_provider_for_test(provider, action)
}

fn write_config(path: &Path, provider: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = format!(
        "[agent]\nprovider = \"{}\"\nmax_conversation_turns = 5\n",
        provider
    );
    fs::write(path, contents)?;
    Ok(())
}

#[test]
#[serial]
fn loads_config_from_workspace_root_before_config_dir() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let root_config = workspace_root.join("vtcode.toml");
    let config_dir_config = config_dir.join("vtcode.toml");
    let home_config = workspace_root.join("home").join("vtcode.toml");

    write_config(&root_config, "workspace-root")?;
    write_config(&config_dir_config, "config-dir")?;
    write_config(&home_config, "home")?;

    let manager = with_test_defaults(
        workspace_root,
        config_dir,
        vec![home_config.clone()],
        || ConfigManager::load_from_workspace(workspace_root),
    )?;

    assert_eq!(manager.config().agent.provider, "workspace-root");
    assert_eq!(manager.config_path(), Some(root_config.as_path()));

    Ok(())
}

#[test]
#[serial]
fn loads_config_from_config_dir_when_root_missing() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_dir_config = config_dir.join("vtcode.toml");
    let home_config = workspace_root.join("home").join("vtcode.toml");

    write_config(&config_dir_config, "config-dir")?;
    write_config(&home_config, "home")?;

    let manager = with_test_defaults(
        workspace_root,
        config_dir,
        vec![home_config.clone()],
        || ConfigManager::load_from_workspace(workspace_root),
    )?;

    assert_eq!(manager.config().agent.provider, "config-dir");
    assert_eq!(manager.config_path(), Some(config_dir_config.as_path()));

    Ok(())
}

#[test]
#[serial]
fn loads_config_from_home_directory_when_workspace_missing() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let home_config = workspace_root.join("home").join("vtcode.toml");
    write_config(&home_config, "home")?;

    let manager = with_test_defaults(
        workspace_root,
        config_dir,
        vec![home_config.clone()],
        || ConfigManager::load_from_workspace(workspace_root),
    )?;

    assert_eq!(manager.config().agent.provider, "home");
    assert_eq!(manager.config_path(), Some(home_config.as_path()));

    Ok(())
}

#[test]
#[serial]
fn falls_back_to_default_config_when_no_files_found() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let manager = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    })?;

    assert!(manager.config_path().is_none());
    assert_eq!(manager.config().agent.provider, defaults::DEFAULT_PROVIDER);

    Ok(())
}

#[test]
#[serial]
fn loads_memory_config_with_defaults() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    // Create config with memory settings
    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[memory]
working_memory_limit = 30
summary_limit = 150
enabled = true
"#;
    fs::write(&config_path, contents)?;

    let manager = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    })?;

    // Check memory config loaded correctly
    assert_eq!(manager.config().memory.working_memory_limit, 30);
    assert_eq!(manager.config().memory.summary_limit, 150);
    assert!(manager.config().memory.enabled);

    // Check defaults are applied
    assert!(manager.config().memory.enable_background_summarization);
    assert_eq!(manager.config().memory.checkpoint_interval_seconds, 300);

    Ok(())
}

#[test]
#[serial]
fn loads_vectordb_config_with_defaults() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    // Create config with vectordb settings
    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[vectordb]
backend = "qdrant"
embedding_dimensions = 512
"#;
    fs::write(&config_path, contents)?;

    let manager = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    })?;

    // Check vectordb config loaded correctly
    assert_eq!(manager.config().vectordb.backend, "qdrant");
    assert_eq!(manager.config().vectordb.embedding_dimensions, 512);

    // Check defaults are applied
    assert_eq!(manager.config().vectordb.collection_prefix, "vtcode");

    Ok(())
}

#[test]
#[serial]
fn applies_memory_defaults_when_not_specified() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    // Create minimal config without memory section
    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"
"#;
    fs::write(&config_path, contents)?;

    let manager = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    })?;

    // Check all defaults are applied
    assert!(manager.config().memory.enabled);
    assert_eq!(manager.config().memory.working_memory_limit, 20);
    assert_eq!(manager.config().memory.summary_limit, 100);
    assert!(manager.config().memory.enable_background_summarization);
    assert!(manager.config().memory.auto_checkpoint);
    assert_eq!(manager.config().memory.checkpoint_interval_seconds, 300);
    assert_eq!(manager.config().vectordb.backend, "memory");
    assert_eq!(manager.config().vectordb.embedding_dimensions, 384);

    Ok(())
}

#[test]
#[serial]
fn rejects_invalid_memory_config_zero_working_limit() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[memory]
working_memory_limit = 0
"#;
    fs::write(&config_path, contents)?;

    let result = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    });

    // Should fail validation
    match result {
        Err(e) => {
            // Use {:#} to get full error chain
            let err_msg = format!("{:#}", e);
            assert!(err_msg.contains("working_memory_limit"), "Error message: {}", err_msg);
        }
        Ok(_) => panic!("Expected validation error for zero working_memory_limit"),
    }

    Ok(())
}

#[test]
#[serial]
fn rejects_invalid_memory_config_excessive_working_limit() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[memory]
working_memory_limit = 2000
"#;
    fs::write(&config_path, contents)?;

    let result = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    });

    // Should fail validation
    let err_msg = match result {
        Err(e) => format!("{:#}", e),
        Ok(_) => panic!("Expected validation to fail"),
    };
    assert!(err_msg.contains("working_memory_limit"));
    assert!(err_msg.contains("1000"));

    Ok(())
}

#[test]
#[serial]
fn rejects_invalid_vectordb_backend() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[vectordb]
backend = "postgres"
"#;
    fs::write(&config_path, contents)?;

    let result = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    });

    // Should fail validation
    let err_msg = match result {
        Err(e) => format!("{:#}", e),
        Ok(_) => panic!("Expected validation to fail"),
    };
    assert!(err_msg.contains("backend"));
    assert!(err_msg.contains("memory") || err_msg.contains("qdrant"));

    Ok(())
}

#[test]
#[serial]
fn rejects_invalid_vectordb_empty_prefix() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[vectordb]
collection_prefix = ""
"#;
    fs::write(&config_path, contents)?;

    let result = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    });

    // Should fail validation
    let err_msg = match result {
        Err(e) => format!("{:#}", e),
        Ok(_) => panic!("Expected validation to fail"),
    };
    assert!(err_msg.contains("collection_prefix"));

    Ok(())
}

#[test]
#[serial]
fn rejects_invalid_qdrant_url() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[vectordb]
backend = "qdrant"

[vectordb.qdrant]
url = "invalid-url"
"#;
    fs::write(&config_path, contents)?;

    let result = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    });

    // Should fail validation
    let err_msg = match result {
        Err(e) => format!("{:#}", e),
        Ok(_) => panic!("Expected validation to fail"),
    };
    assert!(err_msg.contains("url"));
    assert!(err_msg.contains("http"));

    Ok(())
}

#[test]
#[serial]
fn accepts_valid_qdrant_config() -> Result<()> {
    let workspace = TempDir::new()?;
    let workspace_root = workspace.path();
    let config_dir = workspace_root.join(".vtcode");
    fs::create_dir_all(&config_dir)?;

    let config_path = workspace_root.join("vtcode.toml");
    let contents = r#"
[agent]
provider = "openai"

[vectordb]
backend = "qdrant"

[vectordb.qdrant]
url = "http://localhost:6333"
api_key = "test-key"
"#;
    fs::write(&config_path, contents)?;

    let manager = with_test_defaults(workspace_root, config_dir, Vec::new(), || {
        ConfigManager::load_from_workspace(workspace_root)
    })?;

    assert_eq!(manager.config().vectordb.backend, "qdrant");
    assert!(manager.config().vectordb.qdrant.is_some());
    let qdrant = manager.config().vectordb.qdrant.as_ref().unwrap();
    assert_eq!(qdrant.url, "http://localhost:6333");
    assert_eq!(qdrant.api_key, Some("test-key".to_string()));

    Ok(())
}
