//! # vtcode-memory
//!
//! Temporal-decay-based conversation memory system for VTCode agents.
//!
//! ## Overview
//!
//! This crate provides a three-tier memory architecture for managing conversation
//! history in AI agents:
//!
//! 1. **Working Memory (Hot)** - Recent conversation turns with full fidelity
//! 2. **Recent Summaries (Warm)** - Compressed summaries of older turns
//! 3. **Session Logs (Cold)** - On-disk persistence for historical search
//!
//! ## Features
//!
//! - Temporal decay-based relevance scoring
//! - Background summarization (async)
//! - Configurable memory limits
//! - Session persistence and restoration
//! - Historical query detection and retrieval
//!
//! ## Example
//!
//! ```rust
//! use vtcode_memory::{SimpleMemory, MemoryManager, MemoryConfig, ConversationTurn};
//! use vtcode_llm_types::Message;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a memory manager
//!     let mut memory = SimpleMemory::with_defaults(None);
//!
//!     // Add a conversation turn
//!     let turn = ConversationTurn::new(
//!         0,
//!         vec![
//!             Message::user("Hello!".to_string()),
//!             Message::assistant("Hi there!".to_string()),
//!         ],
//!     );
//!     memory.add_turn(turn).await.unwrap();
//!
//!     // Build context for next request
//!     let context = memory.build_context("What's next?");
//!
//!     // Save session
//!     let path = memory.save().await.unwrap();
//!     println!("Session saved to: {:?}", path);
//! }
//! ```

pub mod error;
pub mod manager;
pub mod session_log;
pub mod simple;
pub mod types;

// Re-export main types
pub use error::{MemoryError, Result};
pub use manager::MemoryManager;
pub use session_log::SessionLog;
pub use simple::SimpleMemory;
pub use types::{
    ConversationTurn, GoalStatus, MemoryConfig, MemoryStats, SessionMetadata, TurnSummary,
};
