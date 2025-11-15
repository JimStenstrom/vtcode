# VTCode Troubleshooting Guide

This guide covers common issues and solutions when using VTCode with your IDE.

## Prerequisites Not Found

**Issue**: VTCode extension can't find the VTCode CLI.

**Solution**:
1. Ensure VTCode CLI is installed:
   ```bash
   # Install with Cargo (recommended)
   cargo install vtcode
   
   # Or with Homebrew
   brew install vtcode
   
   # Or with NPM

   ```
2. Check that VTCode is in your PATH:
   ```bash
   vtcode --version
   ```
3. If VTCode is installed in a custom location, update your IDE settings to point to the correct path:
   - VS Code: Set `vtcode.commandPath` in settings to the full path of the VTCode executable
   - Cursor/Windsurf: Look for similar extension settings to specify the VTCode executable path

## Extension Not Working

**Issue**: VTCode commands are not responding or showing errors.

**Solution**:
1. Restart your IDE after installing the extension
2. Check that your workspace contains a `vtcode.toml` configuration file
3. Verify the extension is enabled in your IDE
4. Check the IDE's output panel for error messages

## AI Provider Not Working

**Issue**: VTCode can't connect to AI providers (OpenAI, Anthropic, etc.).

**Solution**:
1. Ensure you have valid API keys in your `vtcode.toml` configuration file
2. Check that your API key has sufficient permissions
3. Verify your internet connection
4. Check if the AI provider has any service interruptions

## Slow Performance

**Issue**: VTCode is taking a long time to respond or analyze code.

**Solution**:
1. For large codebases, consider excluding large directories in your `vtcode.toml`
2. Check that your system has sufficient memory and CPU resources
3. Ensure your internet connection is stable if using cloud-based AI providers
4. Consider switching to a faster AI model in your configuration

## Configuration Issues

**Issue**: VTCode isn't using expected configuration settings.

**Solution**:
1. Verify your `vtcode.toml` file is in the root of your workspace
2. Check the syntax of your configuration file
3. Restart your IDE after making configuration changes
4. Use the "VTCode: Open Configuration" command to edit your config directly

## VS Code-Compatible Editors

**Issue**: Using VTCode with Cursor, Windsurf, or other VS Code-compatible editors.

**Solution**:
VTCode works with any VS Code-compatible editor through the Open VSX registry:
1. Ensure the VTCode CLI is installed separately on your system
2. Install the extension from the Open VSX registry or via VSIX file
3. The extension behavior should be identical to VS Code
4. Configuration settings may be located in different places depending on the editor

## Memory System

### Session Logs Not Saving

**Issue**: No files appearing in `~/.vtcode/sessions/`

**Solution**:
1. Check that `log_directory` is writable:
   ```bash
   ls -la ~/.vtcode/
   mkdir -p ~/.vtcode/sessions
   ```
2. Ensure memory system is enabled:
   ```toml
   [memory]
   enabled = true
   auto_checkpoint = true
   ```
3. Check logs for write errors:
   ```bash
   RUST_LOG=vtcode_memory=debug vtcode
   ```
4. Try manual save on exit if auto-checkpoint isn't working

### Out of Memory Errors

**Issue**: VTCode crashes with OOM (Out of Memory) errors.

**Solution**:
Reduce memory limits in `vtcode.toml`:
```toml
[memory]
working_memory_limit = 10  # Reduce from default 20
summary_limit = 50         # Reduce from default 100
```

For very resource-constrained environments:
```toml
[memory]
working_memory_limit = 5
summary_limit = 20
enable_background_summarization = false  # Disable async processing
```

### Historical Queries Not Working

**Issue**: "Remember when..." queries don't retrieve past context.

**Diagnosis**:
1. Check if memory system is enabled:
   ```bash
   # In vtcode.toml
   [memory]
   enabled = true
   ```
2. Verify summaries exist:
   ```bash
   ls ~/.vtcode/sessions/
   # Should show .json files
   ```
3. Enable debug logging to see query processing:
   ```bash
   RUST_LOG=vtcode_memory=debug vtcode
   ```

**Solution**:
Ensure background summarization is enabled:
```toml
[memory]
enable_background_summarization = true
summary_limit = 100  # Must be > 0
```

### Slow Turn Completion

**Issue**: Long delay after each message.

**Diagnosis**:
Check if synchronous summarization is enabled (blocking).

**Solution**:
Enable background summarization to eliminate delays:
```toml
[memory]
enable_background_summarization = true  # Must be true!
```

If performance is still slow:
```toml
[memory]
working_memory_limit = 10  # Reduce to speed up context building
```

### Session Files Too Large

**Issue**: Session JSON files consuming too much disk space.

**Solution**:
1. Reduce retention limits:
   ```toml
   [memory]
   working_memory_limit = 15
   summary_limit = 75
   ```

2. Clean old sessions periodically:
   ```bash
   # Remove sessions older than 30 days
   find ~/.vtcode/sessions -name "*.json" -mtime +30 -delete
   ```

3. Archive important sessions:
   ```bash
   # Move to archive
   mkdir -p ~/.vtcode/archives
   mv ~/.vtcode/sessions/202411*.json ~/.vtcode/archives/
   ```

### Memory Not Restoring Between Sessions

**Issue**: New sessions don't have context from previous work.

**Note**: This is expected behavior in Phase 1. Cross-session context restoration is planned for Phase 5.

**Current Workaround**:
Search past sessions manually:
```bash
# Find relevant sessions
grep -r "authentication" ~/.vtcode/sessions/

# Extract specific content
jq '.messages[] | select(.content | contains("JWT"))' \
   ~/.vtcode/sessions/20251115_*.json
```

## Need More Help?

If you're still experiencing issues:

1. Check the [main documentation](../README.md)
2. Review the [Cursor and Windsurf Setup Guide](./cursor-windsurf-setup.md) for editor-specific instructions
3. Join our [community Discord](https://discord.gg/vtcode)
4. Open an issue on our [GitHub repository](https://github.com/vinhnx/vtcode/issues)
5. Provide detailed information about your setup, the issue you're experiencing, and any error messages