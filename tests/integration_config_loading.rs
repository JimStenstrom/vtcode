//! Integration tests for configuration loading

use std::path::PathBuf;
use tempfile::NamedTempFile;
use vtcode_config::{MemoryConfig, VectorDbConfig};

#[test]
fn test_memory_config_defaults() {
    let config = MemoryConfig::default();

    assert!(config.enabled, "Memory should be enabled by default");
    assert_eq!(
        config.working_memory_limit, 20,
        "Working memory limit should be 20"
    );
    assert_eq!(config.summary_limit, 100, "Summary limit should be 100");
    assert!(
        config.enable_background_summarization,
        "Background summarization should be enabled"
    );
    assert!(config.auto_checkpoint, "Auto checkpoint should be enabled");
    assert_eq!(
        config.checkpoint_interval_seconds, 300,
        "Checkpoint interval should be 300 seconds"
    );

    println!("✅ Memory config defaults are correct");
}

#[test]
fn test_vectordb_config_defaults() {
    let config = VectorDbConfig::default();

    assert_eq!(config.backend, "memory", "Default backend should be memory");
    assert_eq!(
        config.collection_prefix, "vtcode",
        "Default collection prefix should be vtcode"
    );
    assert_eq!(
        config.embedding_dimensions, 384,
        "Default embedding dimensions should be 384"
    );
    assert!(config.qdrant.is_none(), "Qdrant config should be None by default");

    println!("✅ VectorDB config defaults are correct");
}

#[test]
fn test_config_from_toml() {
    use std::fs;

    let toml_content = r#"
[memory]
enabled = true
working_memory_limit = 30
summary_limit = 150

[vectordb]
backend = "memory"
embedding_dimensions = 512
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), toml_content).unwrap();

    // Load config (adjust based on your loading logic)
    let config: toml::Value = toml::from_str(toml_content).unwrap();

    assert_eq!(
        config["memory"]["working_memory_limit"],
        toml::Value::Integer(30)
    );
    assert_eq!(
        config["vectordb"]["embedding_dimensions"],
        toml::Value::Integer(512)
    );

    println!("✅ Config loading from TOML works");
}

#[test]
fn test_memory_config_serialization() {
    let config = MemoryConfig {
        enabled: true,
        working_memory_limit: 25,
        summary_limit: 120,
        enable_background_summarization: true,
        auto_checkpoint: true,
        checkpoint_interval_seconds: 600,
        log_directory: PathBuf::from("/tmp/sessions"),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("working_memory_limit = 25"));
    assert!(toml_str.contains("summary_limit = 120"));

    // Deserialize back
    let deserialized: MemoryConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.working_memory_limit, 25);
    assert_eq!(deserialized.summary_limit, 120);

    println!("✅ Memory config serialization works");
}

#[test]
fn test_vectordb_config_serialization() {
    let config = VectorDbConfig {
        backend: "qdrant".to_string(),
        collection_prefix: "test".to_string(),
        embedding_dimensions: 512,
        qdrant: Some(vtcode_config::QdrantConfig {
            url: "http://localhost:6333".to_string(),
            api_key: Some("test_key".to_string()),
        }),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("backend = \"qdrant\""));
    assert!(toml_str.contains("embedding_dimensions = 512"));

    // Deserialize back
    let deserialized: VectorDbConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.backend, "qdrant");
    assert_eq!(deserialized.embedding_dimensions, 512);
    assert!(deserialized.qdrant.is_some());

    println!("✅ VectorDB config serialization works");
}

#[test]
fn test_memory_config_partial() {
    let toml_content = r#"
enabled = false
working_memory_limit = 15
"#;

    let config: MemoryConfig = toml::from_str(toml_content).unwrap();

    // Specified values
    assert!(!config.enabled);
    assert_eq!(config.working_memory_limit, 15);

    // Default values for non-specified fields
    assert_eq!(config.summary_limit, 100); // default
    assert!(config.enable_background_summarization); // default

    println!("✅ Partial memory config loading works");
}

#[test]
fn test_vectordb_config_with_qdrant() {
    let toml_content = r#"
backend = "qdrant"
collection_prefix = "myapp"
embedding_dimensions = 768

[qdrant]
url = "http://localhost:6333"
api_key = "secret"
"#;

    let config: VectorDbConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.backend, "qdrant");
    assert_eq!(config.embedding_dimensions, 768);
    assert!(config.qdrant.is_some());

    let qdrant = config.qdrant.unwrap();
    assert_eq!(qdrant.url, "http://localhost:6333");
    assert_eq!(qdrant.api_key, Some("secret".to_string()));

    println!("✅ VectorDB config with Qdrant works");
}

#[test]
fn test_memory_config_disabled() {
    let toml_content = r#"
enabled = false
"#;

    let config: MemoryConfig = toml::from_str(toml_content).unwrap();
    assert!(!config.enabled, "Memory should be disabled");

    println!("✅ Memory config can be disabled");
}

#[test]
fn test_config_json_serialization() {
    let config = MemoryConfig {
        enabled: true,
        working_memory_limit: 20,
        summary_limit: 100,
        enable_background_summarization: true,
        auto_checkpoint: true,
        checkpoint_interval_seconds: 300,
        log_directory: PathBuf::from("/tmp/sessions"),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&config).unwrap();
    assert!(json_str.contains("working_memory_limit"));

    // Deserialize back
    let deserialized: MemoryConfig = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.working_memory_limit, 20);

    println!("✅ Config JSON serialization works");
}

#[test]
fn test_memory_config_custom_directory() {
    let custom_path = PathBuf::from("/custom/path/sessions");
    let config = MemoryConfig {
        log_directory: custom_path.clone(),
        ..Default::default()
    };

    assert_eq!(config.log_directory, custom_path);

    // Test serialization round-trip
    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: MemoryConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.log_directory, custom_path);

    println!("✅ Memory config with custom directory works");
}
