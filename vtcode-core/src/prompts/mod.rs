//! System prompt generation with modular architecture
//!
//! This module re-exports types from vtcode-prompts and provides
//! vtcode-core specific extensions for integration with the agent runtime.

// Re-export all types from vtcode-prompts for backward compatibility
pub use vtcode_prompts::*;

// vtcode-core specific extensions
pub mod core_extensions;

// Re-export core-specific functionality
pub use core_extensions::{
    compose_system_instruction_text_with_config,
    generate_system_instruction_content,
    generate_system_instruction_with_config_and_guidelines,
    read_agent_guidelines,
    read_instruction_hierarchy,
};
