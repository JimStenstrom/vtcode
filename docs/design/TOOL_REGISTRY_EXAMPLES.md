# Tool Registry Implementation Examples

This document provides practical examples of tool registration, metadata configuration, and LLM-friendly documentation patterns.

## Table of Contents
1. [Basic Tool Registration](#basic-tool-registration)
2. [Rich Metadata Configuration](#rich-metadata-configuration)
3. [LLM-Optimized Documentation](#llm-optimized-documentation)
4. [Policy Configuration](#policy-configuration)
5. [Anti-Pattern Detection](#anti-pattern-detection)
6. [Alternative Suggestions](#alternative-suggestions)

## Basic Tool Registration

### Example 1: Simple Read-Only Tool

```rust
use vtcode_core::tools::registry::{ToolRegistration, ToolCategory, RiskLevel};
use serde_json::{json, Value};

async fn read_file_executor(registry: &mut ToolRegistry, args: Value) -> Result<Value> {
    let file_path = args["file_path"].as_str()
        .ok_or_else(|| anyhow!("file_path is required"))?;

    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(2000);

    // Implementation...
    Ok(json!({
        "content": "file contents here",
        "lines_read": 100,
        "total_lines": 150
    }))
}

// Register the tool
let registration = ToolRegistration::new(
    "read_file",
    CapabilityLevel::FileOperations,
    false, // doesn't use PTY
    read_file_executor,
)
.with_metadata(ToolMetadata {
    description: "Read file contents with optional pagination",
    usage_notes: vec![
        UsageNote::tip("Use offset/limit for large files to reduce token usage"),
        UsageNote::warning("Do not use for binary files"),
        UsageNote::important("Prefer this over 'cat' command"),
    ],
    when_to_use: vec![
        "Reading specific files with known paths".into(),
        "Viewing file contents for inspection or analysis".into(),
    ],
    when_not_to_use: vec![
        "Searching for content across files (use Grep instead)".into(),
        "Reading multiple files in sequence (use Glob + parallel reads)".into(),
        "Exploring unknown codebases (use Task agent with Explore mode)".into(),
    ],
    ..Default::default()
})
.with_risk_profile(RiskProfile {
    risk_level: RiskLevel::Low,
    risk_factors: vec![],
    requires_confirmation: false,
    audit_level: AuditLevel::Basic,
})
.with_runtime_profile(RuntimeProfile {
    expected_latency: LatencyCategory::Fast,
    cacheable: true,
    cache_ttl: Some(Duration::from_secs(60)),
    can_run_parallel: true,
    ..Default::default()
});

registry.register_tool(registration)?;
```

### Example 2: Write Tool with Risk Profile

```rust
async fn write_file_executor(registry: &mut ToolRegistry, args: Value) -> Result<Value> {
    let file_path = args["file_path"].as_str()
        .ok_or_else(|| anyhow!("file_path is required"))?;
    let content = args["content"].as_str()
        .ok_or_else(|| anyhow!("content is required"))?;

    // Check if file exists
    let file_exists = registry.file_ops_tool().file_exists(file_path)?;

    if file_exists {
        return Err(anyhow!(
            "File already exists. Use Edit tool to modify existing files. \
             This prevents accidental overwrites."
        ));
    }

    // Implementation...
    Ok(json!({
        "success": true,
        "file_path": file_path,
        "bytes_written": content.len()
    }))
}

let registration = ToolRegistration::new(
    "write_file",
    CapabilityLevel::CodeModification,
    false,
    write_file_executor,
)
.with_metadata(ToolMetadata {
    description: "Write content to a new file",
    usage_notes: vec![
        UsageNote::important("ONLY for creating NEW files. Use Edit for existing files."),
        UsageNote::warning("This will fail if file already exists (safety feature)"),
        UsageNote::tip("Parent directory must exist; use mkdir if needed"),
    ],
    when_to_use: vec![
        "Creating new files that don't exist".into(),
    ],
    when_not_to_use: vec![
        "Modifying existing files (use Edit instead)".into(),
        "Appending to files (read, modify, then Edit)".into(),
        "Creating temporary files (use dedicated temp file APIs)".into(),
    ],
    examples: vec![
        ToolExample {
            description: "Create a new configuration file".into(),
            input: json!({
                "file_path": "/path/to/config.json",
                "content": "{\n  \"setting\": \"value\"\n}"
            }),
            expected_output: Some("File created successfully".into()),
            is_anti_pattern: false,
        },
        ToolExample {
            description: "❌ ANTI-PATTERN: Trying to overwrite existing file".into(),
            input: json!({
                "file_path": "/path/to/existing.txt",
                "content": "new content"
            }),
            expected_output: Some("ERROR: File exists. Use Edit instead.".into()),
            is_anti_pattern: true,
        },
    ],
    ..Default::default()
})
.with_risk_profile(RiskProfile {
    risk_level: RiskLevel::Medium,
    risk_factors: vec![RiskFactor::ModifiesFiles],
    requires_confirmation: true,
    audit_level: AuditLevel::Full,
});
```

### Example 3: Execution Tool with PTY

```rust
async fn bash_executor(registry: &mut ToolRegistry, args: Value) -> Result<Value> {
    let command = args["command"].as_str()
        .ok_or_else(|| anyhow!("command is required"))?;
    let timeout = args.get("timeout")
        .and_then(|v| v.as_u64())
        .unwrap_or(120_000);

    // Validate command against security policies
    if is_dangerous_command(command) {
        return Err(anyhow!(
            "Command '{}' is blocked by security policy. \
             Dangerous commands require explicit user approval.",
            command
        ));
    }

    // Execute with PTY
    let output = registry.pty_manager()
        .execute_command(command, timeout)
        .await?;

    Ok(json!({
        "stdout": output.stdout,
        "stderr": output.stderr,
        "exit_code": output.exit_code
    }))
}

let registration = ToolRegistration::new(
    "bash",
    CapabilityLevel::ShellAccess,
    true, // uses PTY
    bash_executor,
)
.with_metadata(ToolMetadata {
    description: "Execute bash commands in a PTY session",
    detailed_description: Some(
        "Executes shell commands with proper terminal emulation. \
         Supports interactive commands, color output, and signal handling."
            .into(),
    ),
    usage_notes: vec![
        UsageNote::important("Use specialized tools (Read, Edit, Grep) instead of bash equivalents"),
        UsageNote::warning("Commands are subject to security policies"),
        UsageNote::tip("Chain commands with && for sequential execution"),
        UsageNote::deprecated("Don't use bash for: cat, grep, sed, awk, echo"),
    ],
    when_to_use: vec![
        "Running build commands (npm, cargo, make)".into(),
        "Executing tests, linters, formatters".into(),
        "Git operations".into(),
        "System commands without specialized tools".into(),
    ],
    when_not_to_use: vec![
        "Reading files (use Read tool)".into(),
        "Searching content (use Grep tool)".into(),
        "Editing files (use Edit tool)".into(),
        "Communication with user (output text directly)".into(),
    ],
    alternatives: vec![
        ToolAlternative {
            tool_name: "Read".into(),
            when_better: "When reading file contents".into(),
            example: "Instead of 'bash cat file.txt', use Read(file_path='file.txt')".into(),
        },
        ToolAlternative {
            tool_name: "Grep".into(),
            when_better: "When searching for content".into(),
            example: "Instead of 'bash grep pattern file', use Grep(pattern='pattern')".into(),
        },
    ],
    ..Default::default()
})
.with_risk_profile(RiskProfile {
    risk_level: RiskLevel::High,
    risk_factors: vec![
        RiskFactor::ExecutesCode,
        RiskFactor::ModifiesFiles,
        RiskFactor::DestructivePotential,
    ],
    requires_confirmation: true,
    audit_level: AuditLevel::Full,
})
.with_runtime_profile(RuntimeProfile {
    expected_latency: LatencyCategory::Medium,
    timeout_category: ToolTimeoutCategory::Pty,
    default_timeout: Some(Duration::from_secs(120)),
    cacheable: false,
    can_run_parallel: true,
    resource_usage: ResourceUsage {
        cpu_intensity: CpuIntensity::Medium,
        memory_requirement: MemoryRequirement::Moderate,
        io_intensity: IoIntensity::Moderate,
    },
});
```

## Rich Metadata Configuration

### Parameter Schema with Validation

```rust
use serde_json::json;

let parameters_schema = json!({
    "type": "object",
    "properties": {
        "pattern": {
            "type": "string",
            "description": "Regular expression pattern to search for"
        },
        "path": {
            "type": "string",
            "description": "Directory or file to search in"
        },
        "glob": {
            "type": "string",
            "description": "File pattern to filter (e.g., '*.js', '**/*.rs')"
        },
        "output_mode": {
            "type": "string",
            "enum": ["content", "files_with_matches", "count"],
            "default": "files_with_matches",
            "description": "Output format"
        },
        "case_sensitive": {
            "type": "boolean",
            "default": true,
            "description": "Case-sensitive search"
        }
    },
    "required": ["pattern"]
});

let metadata = ToolMetadata {
    parameters_schema,
    required_parameters: vec!["pattern"],
    optional_parameters: vec![
        ParameterInfo {
            name: "output_mode".into(),
            description: "Controls output verbosity. Use 'files_with_matches' \
                         (default) for discovery, 'content' to see matching lines, \
                         'count' for statistics.".into(),
            default_value: Some(json!("files_with_matches")),
            validation_rules: vec![
                ValidationRule::OneOf(vec![
                    "content".into(),
                    "files_with_matches".into(),
                    "count".into()
                ])
            ],
        },
        ParameterInfo {
            name: "glob".into(),
            description: "Filter files by pattern. More efficient than searching \
                         all files.".into(),
            default_value: None,
            validation_rules: vec![
                ValidationRule::Pattern(r"^[\w*./\-]+$".into()),
            ],
        },
    ],
    error_patterns: vec![
        ErrorPattern {
            pattern: "regex parse error".into(),
            meaning: "Invalid regular expression syntax".into(),
            suggested_action: "Check regex syntax. Use simpler pattern or escape special chars.".into(),
        },
        ErrorPattern {
            pattern: "No such file or directory".into(),
            meaning: "Specified path does not exist".into(),
            suggested_action: "Verify path is correct and file/directory exists.".into(),
        },
    ],
    ..Default::default()
};
```

## LLM-Optimized Documentation

### Decision Tree in Description

```rust
let description = r#"
Search file contents using ripgrep.

DECISION TREE:
├─ Need to find files by name? -> Use Glob instead
├─ Need to search specific file? -> Use Read + manual search
├─ Need to search across codebase? -> Use Grep ✓
└─ Need semantic code search? -> Use Task agent with CodeSearch

USAGE:
1. Discovery (default): output_mode="files_with_matches"
   → Find which files contain the pattern

2. Content inspection: output_mode="content"
   → See actual matching lines

3. Statistics: output_mode="count"
   → Count matches per file

IMPORTANT:
- Use glob parameter to filter files (faster than searching all)
- For multiline patterns, set multiline=true
- Default is case-sensitive; use -i flag for case-insensitive

ANTI-PATTERNS:
❌ Don't use bash grep (slower, harder to parse)
❌ Don't search without glob filter on large codebases
❌ Don't use for finding files by name (use Glob)
"#;
```

### Structured Usage Examples

```rust
let examples = vec![
    ToolExample {
        description: "Find all TODO comments in JavaScript files".into(),
        input: json!({
            "pattern": "TODO:",
            "glob": "**/*.js",
            "output_mode": "files_with_matches"
        }),
        expected_output: Some("List of files containing TODO comments".into()),
        is_anti_pattern: false,
    },

    ToolExample {
        description: "View matching lines with context".into(),
        input: json!({
            "pattern": "function authenticate",
            "glob": "**/*.ts",
            "output_mode": "content",
            "-C": 3  // 3 lines of context
        }),
        expected_output: Some("Matching lines with 3 lines before/after".into()),
        is_anti_pattern: false,
    },

    ToolExample {
        description: "❌ ANTI-PATTERN: Using bash grep instead".into(),
        input: json!({
            "tool": "bash",
            "command": "grep -r 'pattern' ."
        }),
        expected_output: Some("Use Grep tool instead for better performance and parsing".into()),
        is_anti_pattern: true,
    },

    ToolExample {
        description: "❌ ANTI-PATTERN: Searching without file filter".into(),
        input: json!({
            "pattern": "common_word",
            "output_mode": "content"
            // Missing glob filter - will search ALL files
        }),
        expected_output: Some("Add glob parameter to filter files and reduce search time".into()),
        is_anti_pattern: true,
    },
];
```

## Policy Configuration

### Workspace-Specific Policies

```rust
// In vtcode.toml:
[tools]
default_policy = "prompt"

[tools.policies]
# Read-only tools: auto-allow
read_file = "allow"
grep = "allow"
list_files = "allow"

# Write tools: prompt
write_file = "prompt"
edit_file = "prompt"

# Execution: prompt in untrusted, allow in trusted workspaces
bash = "prompt"

[tools.trusted_workspaces]
"/home/user/myproject" = { bash = "allow", write_file = "allow" }
```

### Conditional Policies

```rust
pub struct ConditionalPolicy {
    base_policy: ToolPolicy,
    conditions: Vec<PolicyCondition>,
}

// Example: Allow write_file in test directories without prompting
let write_file_policy = ConditionalPolicy {
    base_policy: ToolPolicy::Prompt,
    conditions: vec![
        PolicyCondition {
            condition_type: ConditionType::ParametersMatch(vec!["file_path".into()]),
            allow_if: Box::new(|ctx| {
                ctx.args["file_path"]
                    .as_str()
                    .map(|p| p.contains("/test/") || p.ends_with("_test.rs"))
                    .unwrap_or(false)
            }),
        },
    ],
};
```

## Anti-Pattern Detection

### Registry-Level Validation

```rust
impl ToolRegistry {
    pub fn validate_tool_call(
        &self,
        tool_name: &str,
        args: &Value,
        context: &ExecutionContext,
    ) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Anti-pattern: Using bash for file operations
        if tool_name == "bash" {
            if let Some(cmd) = args["command"].as_str() {
                if cmd.starts_with("cat ") {
                    warnings.push(ValidationWarning::anti_pattern(
                        "Use Read tool instead of 'cat' command",
                        "Read",
                        json!({ "file_path": extract_file_path(cmd) })
                    ));
                }

                if cmd.contains("grep ") {
                    warnings.push(ValidationWarning::anti_pattern(
                        "Use Grep tool instead of 'grep' command",
                        "Grep",
                        json!({ "pattern": extract_grep_pattern(cmd) })
                    ));
                }

                if cmd.contains("echo ") && !cmd.contains(">") {
                    warnings.push(ValidationWarning::anti_pattern(
                        "Output text directly instead of using echo",
                        "Direct output",
                        json!(null)
                    ));
                }
            }
        }

        // Anti-pattern: Reading files in loop
        if tool_name == "read_file" && context.recent_tool_calls.len() > 5 {
            let recent_reads = context.recent_tool_calls
                .iter()
                .filter(|c| c.tool == "read_file")
                .count();

            if recent_reads > 3 {
                warnings.push(ValidationWarning::performance(
                    "Reading many files sequentially. Consider using Grep or Task agent.",
                    "Grep or Task agent",
                    json!({ "pattern": "your_search_pattern" })
                ));
            }
        }

        // Anti-pattern: Write on existing file
        if tool_name == "write_file" {
            if let Some(path) = args["file_path"].as_str() {
                if self.file_ops_tool().file_exists(path)? {
                    warnings.push(ValidationWarning::error(
                        "File exists. Use Edit tool to modify existing files.",
                        "Edit",
                        json!({
                            "file_path": path,
                            "old_string": "text to replace",
                            "new_string": "new text"
                        })
                    ));
                }
            }
        }

        Ok(warnings)
    }
}

pub enum ValidationWarning {
    AntiPattern {
        message: String,
        suggested_tool: String,
        example_args: Value,
    },
    Performance {
        message: String,
        optimization: String,
        example: Value,
    },
    Error {
        message: String,
        resolution: String,
        correct_usage: Value,
    },
}
```

## Alternative Suggestions

### Context-Aware Recommendations

```rust
impl ToolRegistry {
    pub async fn suggest_tool(
        &self,
        intent: &UserIntent,
        context: &ExecutionContext,
    ) -> ToolSuggestion {
        match intent {
            UserIntent::ReadFile { count: 1, path: Some(_) } => {
                ToolSuggestion {
                    tool_name: "Read".into(),
                    reason: "Direct file read with known path".into(),
                    confidence: 0.95,
                    example_usage: json!({ "file_path": "/path/to/file" }).to_string(),
                }
            }

            UserIntent::ReadFile { count, path: None } if *count > 5 => {
                ToolSuggestion {
                    tool_name: "Task".into(),
                    reason: "Reading many files; delegate to exploration agent".into(),
                    confidence: 0.9,
                    example_usage: json!({
                        "subagent_type": "Explore",
                        "prompt": "Find and analyze files matching criteria"
                    }).to_string(),
                }
            }

            UserIntent::SearchContent { pattern, scope } => {
                ToolSuggestion {
                    tool_name: "Grep".into(),
                    reason: "Content search across files".into(),
                    confidence: 0.92,
                    example_usage: json!({
                        "pattern": pattern,
                        "glob": scope,
                        "output_mode": "files_with_matches"
                    }).to_string(),
                }
            }

            UserIntent::ModifyFile { path, exists: true } => {
                ToolSuggestion {
                    tool_name: "Edit".into(),
                    reason: "File exists; use Edit for modifications".into(),
                    confidence: 0.98,
                    example_usage: json!({
                        "file_path": path,
                        "old_string": "exact text to replace",
                        "new_string": "new text"
                    }).to_string(),
                }
            }

            UserIntent::ModifyFile { path, exists: false } => {
                ToolSuggestion {
                    tool_name: "Write".into(),
                    reason: "File doesn't exist; use Write to create".into(),
                    confidence: 0.98,
                    example_usage: json!({
                        "file_path": path,
                        "content": "file contents"
                    }).to_string(),
                }
            }

            _ => ToolSuggestion::default(),
        }
    }
}
```

## Summary

These examples demonstrate:

1. **Rich metadata** that guides LLM decision-making
2. **Clear documentation** with decision trees and anti-patterns
3. **Policy flexibility** for different trust levels
4. **Proactive validation** to prevent common mistakes
5. **Smart suggestions** based on intent and context

The goal is to make the "right thing" easy and the "wrong thing" hard, while providing clear guidance when the LLM makes mistakes.
