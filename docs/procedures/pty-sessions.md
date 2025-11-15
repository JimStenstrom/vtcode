---
type: standard-operating-procedure
id: pty-sessions
---

# Managing PTY Terminal Sessions

## When to Use

When you need to run shell commands, especially:
- Long-running processes (servers, watchers)
- Interactive commands
- Commands that require persistent sessions
- Multiple concurrent operations

## Tools Available

- **`create_pty_session`** - Start a new terminal session
- **`send_pty_input`** - Send commands to a session
- **`read_pty_session`** - Read output from a session
- **`list_pty_sessions`** - List all active sessions
- **`close_pty_session`** - Terminate a session
- **`resize_pty_session`** - Adjust terminal size

## Why PTY Sessions?

### PTY vs run_command (deprecated)

```
❌ run_command (old way):
- One-shot execution
- No session persistence
- Can't interact with running process
- Deprecated

✅ PTY sessions (modern way):
- Persistent sessions
- Can interact with running processes
- Better for long-running commands
- Session management
```

## Basic Workflow

### 1. Create Session

```json
{
  "tool": "create_pty_session",
  "parameters": {
    "session_id": "build",
    "shell": "/bin/bash"
  }
}
```

### 2. Send Commands

```json
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "build",
    "input": "cargo build\n"
  }
}
```

Note: Always include `\n` (newline) to execute the command!

### 3. Read Output

```json
{
  "tool": "read_pty_session",
  "parameters": {
    "session_id": "build"
  }
}
```

### 4. Close When Done

```json
{
  "tool": "close_pty_session",
  "parameters": {
    "session_id": "build"
  }
}
```

## Session Naming Strategy

Use descriptive, purpose-based names:

```
✅ Good names:
- "build" - Running build process
- "test" - Running tests
- "dev-server" - Development server
- "watch" - File watcher
- "git" - Git operations

❌ Bad names:
- "session1" - Not descriptive
- "temp" - Unclear purpose
- "x" - Too vague
```

## Common Patterns

### Pattern 1: Quick Command Execution

For simple, fast commands:

```
1. create_pty_session(session_id="quick")
2. send_pty_input(session_id="quick", input="ls -la\n")
3. read_pty_session(session_id="quick")  # Wait for output
4. close_pty_session(session_id="quick")
```

### Pattern 2: Long-Running Process

For servers, watchers, etc.:

```
1. create_pty_session(session_id="server")
2. send_pty_input(session_id="server", input="npm run dev\n")
3. read_pty_session(session_id="server")  # Check it started
4. # Keep session open
5. Later: close_pty_session(session_id="server")
```

### Pattern 3: Multiple Commands in Sequence

```
1. create_pty_session(session_id="setup")
2. send_pty_input(session_id="setup", input="cd project\n")
3. read_pty_session(session_id="setup")
4. send_pty_input(session_id="setup", input="npm install\n")
5. read_pty_session(session_id="setup")
6. send_pty_input(session_id="setup", input="npm test\n")
7. read_pty_session(session_id="setup")
8. close_pty_session(session_id="setup")
```

### Pattern 4: Parallel Operations

Run multiple things at once:

```
# Session 1: Build
create_pty_session(session_id="build")
send_pty_input(session_id="build", input="cargo build\n")

# Session 2: Tests (in parallel)
create_pty_session(session_id="test")
send_pty_input(session_id="test", input="cargo test\n")

# Session 3: Linter (in parallel)
create_pty_session(session_id="lint")
send_pty_input(session_id="lint", input="cargo clippy\n")

# Read all results
read_pty_session(session_id="build")
read_pty_session(session_id="test")
read_pty_session(session_id="lint")

# Cleanup
close_pty_session(session_id="build")
close_pty_session(session_id="test")
close_pty_session(session_id="lint")
```

## Specific Use Cases

### Running Tests

```json
1. Create session:
{
  "tool": "create_pty_session",
  "parameters": {"session_id": "test"}
}

2. Run tests:
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "test",
    "input": "cargo test --test integration_test\n"
  }
}

3. Wait and read:
{
  "tool": "read_pty_session",
  "parameters": {"session_id": "test"}
}

4. Close:
{
  "tool": "close_pty_session",
  "parameters": {"session_id": "test"}
}
```

### Running Build

```
Rust:
send_pty_input(input="cargo build\n")
send_pty_input(input="cargo build --release\n")

TypeScript:
send_pty_input(input="npm run build\n")

Python:
send_pty_input(input="python setup.py build\n")
```

### Starting Development Server

```json
{
  "tool": "create_pty_session",
  "parameters": {"session_id": "dev"}
}

{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "dev",
    "input": "npm run dev\n"
  }
}

# Server keeps running...
# Later when user wants to stop:
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "dev",
    "input": "\u0003"  # Ctrl+C
  }
}
```

### Git Operations

```
1. Create git session:
create_pty_session(session_id="git")

2. Multiple git commands:
send_pty_input(session_id="git", input="git status\n")
read_pty_session(session_id="git")

send_pty_input(session_id="git", input="git diff\n")
read_pty_session(session_id="git")

send_pty_input(session_id="git", input="git log -5 --oneline\n")
read_pty_session(session_id="git")

3. Close:
close_pty_session(session_id="git")
```

### Interactive Commands

For commands that prompt for input:

```json
1. Start interactive command:
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "interactive",
    "input": "cargo init my-project\n"
  }
}

2. Read prompt:
{
  "tool": "read_pty_session",
  "parameters": {"session_id": "interactive"}
}

3. Send response:
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "interactive",
    "input": "yes\n"
  }
}
```

## Best Practices

### 1. Always Include Newline

```
❌ Wrong (command won't execute):
send_pty_input(input="cargo build")

✅ Correct:
send_pty_input(input="cargo build\n")
```

### 2. Read After Sending

Always read output after sending commands:

```
✅ Proper flow:
send_pty_input(input="ls\n")
read_pty_session()  # Get output

❌ Wrong (you miss output):
send_pty_input(input="ls\n")
send_pty_input(input="pwd\n")  # Overwrites previous output
```

### 3. Use Unique Session IDs

```
✅ Good:
create_pty_session(session_id="build-2024")
create_pty_session(session_id="test-integration")

❌ Risky:
create_pty_session(session_id="session")  # Generic!
```

### 4. Clean Up Sessions

Always close sessions when done:

```
✅ Cleanup:
close_pty_session(session_id="build")

❌ Leak (wastes resources):
# Created session but never closed it
```

### 5. Check Active Sessions

Before creating, check what's active:

```json
{
  "tool": "list_pty_sessions",
  "parameters": {}
}
```

### 6. Handle Errors Gracefully

```
If session doesn't exist:
- list_pty_sessions() to verify
- create_pty_session() to start new one

If command fails:
- read_pty_session() to see error
- Decide next action based on error
```

## Timing Considerations

### Fast Commands

For quick commands, read immediately:

```
send_pty_input(input="echo 'hello'\n")
read_pty_session()  # Output ready instantly
```

### Slow Commands

For long operations, may need to wait:

```
send_pty_input(input="cargo build --release\n")
# ... wait a bit ...
read_pty_session()  # May not be complete yet

# Can read multiple times to see progress:
read_pty_session()  # Check progress
# ... wait more ...
read_pty_session()  # Check again
```

### Background Processes

For servers/watchers that run indefinitely:

```
send_pty_input(input="npm run dev\n")
read_pty_session()  # Check startup messages
# Leave running
# User continues working
# Later: close_pty_session() to stop
```

## Special Characters

### Sending Ctrl+C

To interrupt a running command:

```json
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "server",
    "input": "\u0003"
  }
}
```

### Sending Ctrl+D (EOF)

To signal end of input:

```json
{
  "tool": "send_pty_input",
  "parameters": {
    "session_id": "repl",
    "input": "\u0004"
  }
}
```

## Common Workflows

### Build and Test

```
1. Create session:
create_pty_session(session_id="ci")

2. Build:
send_pty_input(session_id="ci", input="cargo build\n")
read_pty_session(session_id="ci")
# Check for errors

3. Test:
send_pty_input(session_id="ci", input="cargo test\n")
read_pty_session(session_id="ci")
# Check for failures

4. Cleanup:
close_pty_session(session_id="ci")
```

### Git Workflow

```
1. Create session:
create_pty_session(session_id="git")

2. Status:
send_pty_input(session_id="git", input="git status\n")
read_pty_session(session_id="git")

3. Diff:
send_pty_input(session_id="git", input="git diff\n")
read_pty_session(session_id="git")

4. Add and commit:
send_pty_input(session_id="git", input="git add .\n")
send_pty_input(session_id="git", input='git commit -m "message"\n')
read_pty_session(session_id="git")

5. Close:
close_pty_session(session_id="git")
```

### Development Server Management

```
1. Start server:
create_pty_session(session_id="dev-server")
send_pty_input(session_id="dev-server", input="npm run dev\n")
read_pty_session(session_id="dev-server")
# Server is now running

2. User works on code...

3. Restart server:
send_pty_input(session_id="dev-server", input="\u0003")  # Ctrl+C
send_pty_input(session_id="dev-server", input="npm run dev\n")

4. When done:
send_pty_input(session_id="dev-server", input="\u0003")
close_pty_session(session_id="dev-server")
```

## Error Handling

### Session Already Exists

```json
Error: "Session 'build' already exists"

Solutions:
1. list_pty_sessions() to see active sessions
2. close_pty_session(session_id="build") first
3. Use different session_id
```

### Session Not Found

```json
Error: "Session 'test' not found"

Solutions:
1. list_pty_sessions() to verify name
2. create_pty_session() first
3. Check for typos in session_id
```

### Command Timeout

```json
Command seems to hang

Solutions:
1. Send Ctrl+C: send_pty_input(input="\u0003")
2. Read to see current state
3. Close and restart session
```

## Anti-Patterns

❌ **Forgetting newline**
```
send_pty_input(input="cargo build")  # Won't execute!
```

❌ **Not reading output**
```
send_pty_input(input="command\n")
# No read_pty_session() - missed output
```

❌ **Session leak**
```
create_pty_session(...)
# Never close_pty_session()
```

❌ **Reusing session IDs carelessly**
```
create_pty_session(session_id="temp")
# ... later in code ...
create_pty_session(session_id="temp")  # Error: already exists!
```

❌ **Not handling long commands**
```
send_pty_input(input="cargo build --release\n")
read_pty_session()  # Immediately - build might not be done!
```

## Quick Reference

| Task | Tools | Pattern |
|------|-------|---------|
| Quick command | create → send → read → close | One-shot execution |
| Long process | create → send → read → keep open | Server/watcher |
| Sequential commands | create → send → read → send → read → close | Multiple steps |
| Parallel operations | Multiple create/send | Concurrent tasks |
| List active | list_pty_sessions | Check what's running |

## Summary

1. **PTY for persistent sessions** - Better than one-shot commands
2. **Always include newline** - `\n` executes the command
3. **Read after send** - Don't miss output
4. **Descriptive session IDs** - "build", "test", "dev-server"
5. **Clean up sessions** - close when done
6. **Handle timing** - Fast vs slow vs background commands
7. **Use Ctrl+C to interrupt** - `\u0003` character
