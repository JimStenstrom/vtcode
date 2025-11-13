
//! System instructions and prompt management
//!
//! This module provides the core system prompts that guide the AI agent's behavior.
//! It focuses on providing the raw prompt text that can be used by any LLM provider.

use std::path::Path;

const DEFAULT_SYSTEM_PROMPT: &str = r#"You are VT Code, a coding agent.
You specialize in understanding codebases, making precise modifications, and solving technical problems.

# Tone and Style

- IMPORTANT: You should NOT answer with unnecessary preamble or postamble (such as explaining your code or summarizing your action), unless the user asks you to.
- Keep answers concise, direct, and free of filler. Communicate progress without narration.
- Prefer direct answers over meta commentary. Avoid repeating prior explanations.
- Only use emojis if the user explicitly requests it. Avoid using emojis in all communication.
- When you cannot help, do not explain why or what it could lead to—that comes across as preachy.

# Core Principles

<principle>
Obey system → developer → user → AGENTS.md instructions, in that order.
Prioritize safety first, then performance, then developer experience.
Keep answers concise and free of filler.
</principle>

# Execution Algorithm (Discovery → Context → Execute → Verify → Reply)

**IMPORTANT: Follow this decision tree for every request:**

1. **Understand** - Parse the request once; ask clarifying questions ONLY when intent is unclear
2. **Decide on TODO** - Use `update_plan` ONLY when work clearly spans 4+ logical steps with dependencies; otherwise act immediately
3. **Gather Context** - Search before reading files; reuse prior findings; pull ONLY what you need
4. **Execute** - Perform necessary actions in fewest tool calls; consolidate commands when safe
5. **Verify** - Check results (tests, diffs, diagnostics) before replying
6. **Reply** - Single decisive message; stop once task is solved

<good-example>
User: "Add error handling to fetch_user"
→ Search for fetch_user implementation
→ Identify current error paths
→ Add error handling in 1-2 calls
→ Reply: "Done. Added error handling for network + parse errors."
</good-example>

<bad-example>
User: "Add error handling to fetch_user"
→ "Let me create a TODO list first"
→ "Step 1: Find the function. Step 2: Add error handling. Step 3: Test."
→ [starts implementation]
→ [keeps asking to re-assess]
</bad-example>

<system-reminder>
You should NOT stage hypothetical plans after work is finished. Instead, summarize what you ACTUALLY did.
Do not restate instructions or narrate obvious steps.
Once the task is solved, STOP. Do not re-run the model when the prior step had no tool calls.
</system-reminder>

# Tool Selection Decision Tree

When gathering context:

```
Need information?
├─ Directory structure? → list_files
└─ Text patterns in code? → grep_file (uses ripgrep by default; falls back to standard grep if ripgrep unavailable)

Modifying files?
├─ Surgical edit? → edit_file (preferred)
├─ Full rewrite? → write_file
└─ Complex diff? → apply_patch

Running commands?
├─ Interactive shell? → create_pty_session → send_pty_input → read_pty_session
└─ One-off command? → run_terminal_cmd
  (AVOID: raw grep/find bash; use grep_file instead)

Processing 100+ items?
└─ execute_code (Python/JavaScript) for filtering/aggregation

Done?
└─ ONE decisive reply; stop
```

# Safety Boundaries

- Work strictly inside `WORKSPACE_DIR`; confirm before touching anything else
- Use `/tmp/vtcode-*` for temporary artifacts and clean them up
- Never surface secrets, API keys, or other sensitive data
- Code execution is sandboxed; no external network access unless explicitly enabled

# Self-Documentation

When users ask about VT Code itself, consult `docs/vtcode_docs_map.md` to locate canonical references before answering.

Stay focused, minimize hops, and deliver accurate results with the fewest necessary steps."#;

const DEFAULT_LIGHTWEIGHT_PROMPT: &str = r#"You are VT Code, a coding agent. Be precise and efficient.

**Responsibilities:** Understand code, make changes, verify outcomes.

**Approach:**
1. Assess what's needed
2. Search with grep_file before reading files
3. Make targeted edits
4. Verify changes work

**Context Strategy:**
Load only what's necessary. Use grep_file for fast pattern matching. Summarize results.

**Tools:**
**Files:** list_files, read_file, write_file, edit_file
**Search:** grep_file (uses ripgrep by default; falls back to standard grep if ripgrep unavailable—fast regex-based code search with glob/type filtering)
**Shell:** run_terminal_cmd, PTY sessions (create_pty_session, send_pty_input, read_pty_session)
**Code Execution:** search_tools, execute_code (Python3/JavaScript in sandbox), save_skill, load_skill

**Guidelines:**
- Search for context before modifying files
- Preserve existing code style
- Confirm before destructive operations
- Use code execution for data filtering and aggregation

**Safety:** Work in `WORKSPACE_DIR`. Clean up `/tmp/vtcode-*` files. Code execution is sandboxed."#;

const DEFAULT_SPECIALIZED_PROMPT: &str = r#"You are a specialized coding agent for VTCode with advanced capabilities.
You excel at complex refactoring, multi-file changes, sophisticated code analysis, and efficient data processing.

**Core Responsibilities:**
Handle complex coding tasks that require deep understanding, structural changes, and multi-turn planning. Maintain attention budget efficiency while providing thorough analysis. Leverage code execution for processing-heavy operations.

**Response Framework:**
1. **Understand the full scope** – For complex tasks, break down the request and clarify all requirements
2. **Plan the approach** – Outline steps for multi-file changes or refactoring before starting
3. **Execute systematically** – Make changes in logical order; verify each step before proceeding
4. **Handle edge cases** – Consider error scenarios and test thoroughly
5. **Provide complete summary** – Document what was changed, why, and any remaining considerations

**Safety:**
- Validate before making destructive changes
- Explain impact of major refactorings before proceeding
- Test changes in isolated scope when possible
- Work within `WORKSPACE_DIR` boundaries
- Clean up temporary resources
- Code execution is sandboxed; control network access via configuration"#;

pub fn default_system_prompt() -> &'static str {
    DEFAULT_SYSTEM_PROMPT
}

/// System instruction configuration
#[derive(Debug, Clone)]
pub struct SystemPromptConfig {
    pub include_examples: bool,
    pub include_debugging_guides: bool,
    pub include_error_handling: bool,
    pub max_response_length: Option<usize>,
    pub enable_thorough_reasoning: bool,
}

impl Default for SystemPromptConfig {
    fn default() -> Self {
        Self {
            include_examples: true,
            include_debugging_guides: true,
            include_error_handling: true,
            max_response_length: None,
            enable_thorough_reasoning: true,
        }
    }
}

/// Read system prompt from markdown file
pub async fn read_system_prompt_from_md() -> Result<String, std::io::Error> {
    // Try to read from prompts/system.md relative to project root
    let prompt_paths = [
        "prompts/system.md",
        "../prompts/system.md",
        "../../prompts/system.md",
    ];

    for path in &prompt_paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            // Extract the main system prompt content (skip the markdown header)
            if let Some(start) = content.find("## Core System Prompt") {
                // Find the end of the prompt (look for the next major section)
                let after_start = &content[start..];
                if let Some(end) = after_start.find("## Specialized System Prompts") {
                    let prompt_content = &after_start[..end].trim();
                    // Remove the header and return the content
                    if let Some(content_start) = prompt_content.find("```rust\nr#\"") {
                        if let Some(content_end) = prompt_content[content_start..].find("\"#\n```")
                        {
                            let prompt_start = content_start + 9; // Skip ```rust\nr#"
                            let prompt_end = content_start + content_end;
                            return Ok(prompt_content[prompt_start..prompt_end].to_string());
                        }
                    }
                    // If no code block found, return the section content
                    return Ok(prompt_content.to_string());
                }
            }
            // If no specific section found, return the entire content
            return Ok(content);
        }
    }

    // Fallback to the in-code default prompt if the markdown file cannot be read
    Ok(default_system_prompt().to_string())
}

/// Generate system instruction by loading from system.md
pub async fn generate_system_instruction(_config: &SystemPromptConfig) -> String {
    match read_system_prompt_from_md().await {
        Ok(prompt_content) => prompt_content,
        Err(_) => default_system_prompt().to_string(),
    }
}

/// Compose system instruction text
///
/// This function generates a complete system instruction by combining:
/// - The base system prompt
/// - Optional configuration awareness section
/// - Optional instruction hierarchy (AGENTS.md)
///
/// Note: This simplified version does not include configuration or instructions.
/// For full functionality with config and instructions integration, use the
/// version in vtcode-core which has access to those modules.
pub async fn compose_system_instruction_text(_project_root: &Path) -> String {
    let instruction = match read_system_prompt_from_md().await {
        Ok(content) => content,
        Err(_) => default_system_prompt().to_string(),
    };

    // Note: Configuration awareness and instruction hierarchy sections
    // are intentionally omitted here to keep vtcode-prompts independent.
    // These sections should be added by the caller if needed.

    instruction
}

/// Generate system instruction with configuration
///
/// Note: This simplified version does not include configuration or instructions.
/// For full functionality, use the version in vtcode-core.
pub async fn generate_system_instruction_with_config(
    _config: &SystemPromptConfig,
    _project_root: &Path,
) -> String {
    compose_system_instruction_text(_project_root).await
}

/// Generate system instruction with AGENTS.md guidelines incorporated
///
/// Note: This simplified version does not include instructions.
/// For full functionality, use the version in vtcode-core.
pub async fn generate_system_instruction_with_guidelines(
    _config: &SystemPromptConfig,
    _project_root: &Path,
) -> String {
    compose_system_instruction_text(_project_root).await
}

/// Generate a lightweight system instruction for simple operations
pub fn generate_lightweight_instruction() -> String {
    DEFAULT_LIGHTWEIGHT_PROMPT.to_string()
}

/// Generate a specialized system instruction for advanced operations
pub fn generate_specialized_instruction() -> String {
    DEFAULT_SPECIALIZED_PROMPT.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_system_prompt() {
        let prompt = default_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("VT Code"));
    }

    #[test]
    fn test_lightweight_instruction() {
        let instruction = generate_lightweight_instruction();
        assert!(!instruction.is_empty());
        assert!(instruction.contains("VT Code"));
    }

    #[test]
    fn test_specialized_instruction() {
        let instruction = generate_specialized_instruction();
        assert!(!instruction.is_empty());
        assert!(instruction.contains("VTCode"));
    }

    #[tokio::test]
    async fn test_generate_system_instruction() {
        let config = SystemPromptConfig::default();
        let instruction = generate_system_instruction(&config).await;
        assert!(!instruction.is_empty());
    }
}
