//! # vtcode-procedures
//!
//! Standard Operating Procedures (SOPs) system for VTCode agents.
//!
//! ## Overview
//!
//! This crate provides functionality for loading, indexing, and retrieving
//! Standard Operating Procedures to guide LLM behavior on common workflows.
//!
//! ## Features
//!
//! - Load procedures from markdown files
//! - Semantic search via RAG (Retrieval-Augmented Generation)
//! - Configurable procedure directories
//! - Support for both project-level and user-specific procedures
//!
//! ## Example
//!
//! ```no_run
//! use vtcode_procedures::ProcedureManager;
//! use vtcode_config::ProceduresConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ProceduresConfig::default();
//!     let manager = ProcedureManager::new(config).await.unwrap();
//!
//!     // Retrieve relevant procedures
//!     let procedures = manager.get_relevant_procedures("how to edit files", 3).await.unwrap();
//!     for proc in procedures {
//!         println!("{}", proc);
//!     }
//! }
//! ```

pub mod loader;
pub mod manager;

// Re-export main types
pub use loader::load_procedures_from_dir;
pub use manager::{ProcedureManager, ProcedureStats};
