# Lifecycle Hooks - Practical Examples

This guide provides detailed practical examples for using lifecycle hooks in VT Code. For core concepts and configuration reference, see [Lifecycle Hooks](./lifecycle-hooks.md).

## Table of Contents

- [Getting Started](#getting-started)
- [Example 1: Enhanced Session Context](#example-1-enhanced-session-context)
- [Example 2: Pre-Tool Validation](#example-2-pre-tool-validation)
- [Example 3: Security Policy Enforcement](#example-3-security-policy-enforcement)
- [Example 4: Development Environment Setup](#example-4-development-environment-setup)
- [Example 5: CI/CD Integration](#example-5-cicd-integration)
- [Example 6: Monitoring and Analytics](#example-6-monitoring-and-analytics)
- [Testing and Debugging](#testing-and-debugging)
- [Security Considerations](#security-considerations)
- [Performance Optimization](#performance-optimization)

## Getting Started

To start using lifecycle hooks in your project:

1. **Create a `vtcode.toml` configuration file** in your project root if you don't already have one:

```bash
touch vtcode.toml
```

2. **Add the lifecycle hooks section** to your configuration:

```toml
[hooks.lifecycle]
# Add your hooks here
```

3. **Create a hooks directory** to store your hook scripts:

```bash
mkdir -p .vtcode/hooks
```

## Example 1: Enhanced Session Context

Add project-specific context when a session starts to help the model understand your project structure.

### Create the Script

Create `.vtcode/hooks/session-context.sh`:

```bash
#!/bin/bash

# Read the JSON payload from stdin
payload=$(cat)

# Extract project info and return as context
echo "Project: $(basename $VT_PROJECT_DIR)"
echo "Files in project root:"
ls -la $VT_PROJECT_DIR | head -20

# Show git status if available
if command -v git &> /dev/null && [ -d "$VT_PROJECT_DIR/.git" ]; then
  echo ""
  echo "Git Status:"
  cd "$VT_PROJECT_DIR" && git status --short
fi
```

### Make Executable

```bash
chmod +x .vtcode/hooks/session-context.sh
```

### Configure the Hook

Add to your `vtcode.toml`:

```toml
[hooks.lifecycle]
session_start = [
  {
    hooks = [
      { command = "$VT_PROJECT_DIR/.vtcode/hooks/session-context.sh" }
    ]
  }
]
```

## Example 2: Pre-Tool Validation

Validate commands before execution to prevent dangerous operations.

### Create the Validation Script

Create `.vtcode/hooks/validate-bash.sh`:

```bash
#!/bin/bash

# Read the JSON payload from stdin
payload=$(cat)

# Extract the command being run (requires jq)
if command -v jq &> /dev/null; then
  command=$(echo "$payload" | jq -r '.tool_input.command // ""')
else
  # Fallback without jq
  command=$(echo "$payload" | grep -o '"command":"[^"]*"' | cut -d'"' -f4)
fi

# Block dangerous commands
if [[ "$command" == *"rm -rf /"* ]] || [[ "$command" == *"dd if="* ]]; then
    echo "Dangerous command blocked: $command" >&2
    exit 2  # Exit code 2 blocks the command
fi

# Warn about potentially risky commands
if [[ "$command" == *"rm -rf"* ]] || [[ "$command" == *"sudo"* ]]; then
    echo "Warning: Potentially risky command: $command" >&2
fi

# Allow safe commands
echo "Command approved: $command"
exit 0
```

### Configure the Hook

```toml
[hooks.lifecycle]
pre_tool_use = [
  {
    matcher = "Bash",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/validate-bash.sh",
        timeout_seconds = 5
      }
    ]
  }
]
```

## Example 3: Security Policy Enforcement

Implement comprehensive security checks across multiple hook events.

### Configuration

```toml
[hooks.lifecycle]
# Block prompts containing sensitive data patterns
user_prompt_submit = [
  {
    matcher = ".*password.*|.*secret.*|.*token.*|.*api.*key.*",
    hooks = [
      {
        command = '''
          python3 -c "
import sys, json
payload = json.load(sys.stdin)
prompt = payload.get('prompt', '')
print(f'Prompt blocked for security reasons: {prompt[:50]}...', file=sys.stderr)
sys.exit(2)
"
        ''',
        timeout_seconds = 5
      }
    ]
  }
]

# Validate Bash commands for dangerous patterns
pre_tool_use = [
  {
    matcher = "Bash",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/security-check.sh",
        timeout_seconds = 10
      }
    ]
  }
]

# Log completed Bash commands for audit trail
post_tool_use = [
  {
    matcher = "Bash",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/log-command.sh"
      }
    ]
  }
]
```

### Security Check Script

Create `.vtcode/hooks/security-check.sh`:

```bash
#!/bin/bash

payload=$(cat)
command=$(echo "$payload" | jq -r '.tool_input.command // ""')

# List of dangerous patterns
dangerous_patterns=(
  "rm -rf /"
  "mkfs\."
  "> /dev/sd"
  "dd if=.*of=/dev"
  "chmod 777"
  "curl.*|.*bash"
  "wget.*|.*sh"
)

# Check each pattern
for pattern in "${dangerous_patterns[@]}"; do
  if echo "$command" | grep -E "$pattern" &> /dev/null; then
    echo "BLOCKED: Command matches dangerous pattern: $pattern" >&2
    echo "Command: $command" >&2
    exit 2
  fi
done

echo "Security check passed"
exit 0
```

### Audit Log Script

Create `.vtcode/hooks/log-command.sh`:

```bash
#!/bin/bash

payload=$(cat)
log_file="$VT_PROJECT_DIR/.vtcode/command-audit.log"

# Extract command details
command=$(echo "$payload" | jq -r '.tool_input.command // "unknown"')
timestamp=$(date '+%Y-%m-%d %H:%M:%S')

# Append to audit log
echo "[$timestamp] $VT_SESSION_ID: $command" >> "$log_file"

exit 0
```

## Example 4: Development Environment Setup

Set up project-specific context and validation for code modifications.

### Configuration

```toml
[hooks.lifecycle]
# Set up project environment at session start
session_start = [
  {
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/setup-env.sh",
        timeout_seconds = 30
      }
    ]
  }
]

# Add project-specific context for code modifications
pre_tool_use = [
  {
    matcher = "Write|Edit",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/check-style.sh"
      }
    ]
  }
]

# Validate code after write operations
post_tool_use = [
  {
    matcher = "Write|Edit",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/run-linter.sh",
        timeout_seconds = 30
      }
    ]
  }
]
```

### Environment Setup Script

Create `.vtcode/hooks/setup-env.sh`:

```bash
#!/bin/bash

echo "=== Project Environment Setup ==="
echo "Project: $(basename $VT_PROJECT_DIR)"
echo "Session: $VT_SESSION_ID"

# Check for required tools
required_tools=("git" "cargo" "rustc")
missing_tools=()

for tool in "${required_tools[@]}"; do
  if ! command -v "$tool" &> /dev/null; then
    missing_tools+=("$tool")
  fi
done

if [ ${#missing_tools[@]} -gt 0 ]; then
  echo "Warning: Missing tools: ${missing_tools[*]}" >&2
fi

# Show project metadata
if [ -f "$VT_PROJECT_DIR/Cargo.toml" ]; then
  version=$(grep '^version' "$VT_PROJECT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
  echo "Rust project version: $version"
fi

exit 0
```

### Style Check Script

Create `.vtcode/hooks/check-style.sh`:

```bash
#!/bin/bash

payload=$(cat)
file_path=$(echo "$payload" | jq -r '.tool_input.file_path // .tool_input.path // ""')

if [ -z "$file_path" ]; then
  exit 0  # No file path, skip
fi

# Check file extension
ext="${file_path##*.}"

case "$ext" in
  rs)
    echo "Rust file modification detected: $file_path"
    echo "Remember to follow Rust conventions (snake_case, rustfmt)"
    ;;
  py)
    echo "Python file modification detected: $file_path"
    echo "Remember to follow PEP 8 style guidelines"
    ;;
esac

exit 0
```

### Linter Script

Create `.vtcode/hooks/run-linter.sh`:

```bash
#!/bin/bash

payload=$(cat)
file_path=$(echo "$payload" | jq -r '.tool_input.file_path // .tool_input.path // ""')

if [ -z "$file_path" ] || [ ! -f "$VT_PROJECT_DIR/$file_path" ]; then
  exit 0
fi

ext="${file_path##*.}"

case "$ext" in
  rs)
    if command -v cargo &> /dev/null; then
      echo "Running cargo check..."
      cd "$VT_PROJECT_DIR" && cargo check --quiet 2>&1 | head -20
    fi
    ;;
  py)
    if command -v flake8 &> /dev/null; then
      echo "Running flake8..."
      flake8 "$VT_PROJECT_DIR/$file_path" 2>&1 | head -20
    fi
    ;;
esac

exit 0
```

## Example 5: CI/CD Integration

Integrate with your development workflow for automated testing and validation.

### Configuration

```toml
[hooks.lifecycle]
# Run tests when code is modified
post_tool_use = [
  {
    matcher = "Write|Edit",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/run-tests.sh",
        timeout_seconds = 120
      }
    ]
  }
]

# Validate commit messages
pre_tool_use = [
  {
    matcher = "Bash",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/validate-commit.sh"
      }
    ]
  }
]
```

### Test Runner Script

Create `.vtcode/hooks/run-tests.sh`:

```bash
#!/bin/bash

payload=$(cat)
file_path=$(echo "$payload" | jq -r '.tool_input.file_path // .tool_input.path // ""')

# Only run tests for code files
if [[ ! "$file_path" =~ \.(rs|py|js|ts)$ ]]; then
  exit 0
fi

echo "Running tests after modification to $file_path..."

cd "$VT_PROJECT_DIR"

# Detect project type and run appropriate tests
if [ -f "Cargo.toml" ]; then
  echo "Running Rust tests..."
  cargo test --quiet 2>&1 | tail -20
elif [ -f "package.json" ]; then
  echo "Running Node.js tests..."
  npm test 2>&1 | tail -20
elif [ -f "pytest.ini" ] || [ -f "setup.py" ]; then
  echo "Running Python tests..."
  pytest --quiet 2>&1 | tail -20
fi

exit 0
```

## Example 6: Monitoring and Analytics

Track agent usage and performance for analysis.

### Configuration

```toml
[hooks.lifecycle]
# Log session start
session_start = [
  {
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/log-session-start.sh"
      }
    ]
  }
]

# Log tool usage
post_tool_use = [
  {
    matcher = ".*",
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/log-tool-usage.sh"
      }
    ]
  }
]

# Log session end
session_end = [
  {
    hooks = [
      {
        command = "$VT_PROJECT_DIR/.vtcode/hooks/log-session-end.sh"
      }
    ]
  }
]
```

### Session Logging Scripts

Create `.vtcode/hooks/log-session-start.sh`:

```bash
#!/bin/bash

payload=$(cat)
log_file="$VT_PROJECT_DIR/.vtcode/session-analytics.log"
timestamp=$(date '+%Y-%m-%d %H:%M:%S')

echo "[$timestamp] SESSION_START $VT_SESSION_ID" >> "$log_file"

exit 0
```

Create `.vtcode/hooks/log-tool-usage.sh`:

```bash
#!/bin/bash

payload=$(cat)
tool_name=$(echo "$payload" | jq -r '.tool_name // "unknown"')
log_file="$VT_PROJECT_DIR/.vtcode/session-analytics.log"
timestamp=$(date '+%Y-%m-%d %H:%M:%S')

echo "[$timestamp] TOOL_USE $VT_SESSION_ID $tool_name" >> "$log_file"

exit 0
```

## Testing and Debugging

### Validate Configuration

```bash
vtcode config validate
```

### Manual Hook Testing

Create a test payload:

```bash
cat > test-payload.json << 'EOF'
{
  "session_id": "test-session-123",
  "cwd": "/path/to/project",
  "hook_event_name": "SessionStart",
  "source": "startup",
  "transcript_path": null
}
EOF
```

Test your hook:

```bash
cat test-payload.json | .vtcode/hooks/session-context.sh
```

### Check Script Permissions

```bash
chmod +x .vtcode/hooks/*.sh
```

### Debugging Tips

1. **Check hook execution** by looking at stderr output in the VT Code UI
2. **Use `jq`** for robust JSON payload parsing in your scripts
3. **Set shorter timeouts** during development to avoid hanging processes
4. **Log to files** for debugging complex hook logic:

```bash
echo "$(date): Processing hook for $VT_HOOK_EVENT" >> /tmp/vtcode-hooks.log
```

5. **Test exit codes** carefully - remember that exit code 2 blocks execution

## Security Considerations

1. **Sandbox your scripts** - avoid running potentially malicious code
2. **Validate all inputs** - never trust user input or tool parameters without validation
3. **Use relative paths** - prefer `$VT_PROJECT_DIR` over hardcoded paths
4. **Minimize permissions** - run hooks with minimal required privileges
5. **Audit script content** - regularly review hook scripts for security issues
6. **Escape shell variables** - use proper quoting to prevent injection:

```bash
# Good
command=$(echo "$payload" | jq -r '.tool_input.command')

# Bad - potential injection
command=$(echo $payload | grep command)
```

## Performance Optimization

### Optimize Timeout Values

Set appropriate timeouts for different operations:

- Fast validations: 1-5 seconds
- Code analysis: 10-30 seconds
- Full project scans: 30-60 seconds
- Long-running processes: 120+ seconds (use sparingly)

### Cache Expensive Operations

```bash
# Example: cache git status results
cache_file="/tmp/vtcode_git_status_$VT_SESSION_ID"
if [[ ! -f "$cache_file" ]] || [[ $(find "$cache_file" -mmin +5 2>/dev/null) ]]; then
  git status --porcelain > "$cache_file"
fi
cat "$cache_file"
```

### Parallel Execution Considerations

Hooks in the same group run sequentially, but multiple matching groups might run in parallel. Design your hooks to be thread-safe if needed:

```bash
# Use session-specific temp files
temp_file="/tmp/hook_${VT_SESSION_ID}_$(date +%s).tmp"
```

## Common Use Cases Summary

| Use Case | Events | Purpose |
|----------|--------|---------|
| Security Validation | `pre_tool_use` | Validate dangerous commands before execution |
| Context Enrichment | `session_start`, `user_prompt_submit` | Add project-specific information |
| Policy Enforcement | `user_prompt_submit` | Block prompts containing sensitive keywords |
| Logging | `post_tool_use`, `session_end` | Track agent activity and tool usage |
| Environment Setup | `session_start` | Configure project-specific settings |
| Code Quality | `post_tool_use` (Write/Edit) | Run linters and tests after modifications |
| Audit Trail | All events | Comprehensive logging for compliance |

## See Also

* **[Lifecycle Hooks (Core Concepts)](./lifecycle-hooks.md)** - Configuration and event reference
* **[Security Guide](./security.md)** - Security best practices
* **[Configuration Reference](../config.md)** - Complete configuration options
