---
type: standard-operating-procedure
id: test-before-commit
---

# Test Before Commit

## When to Use

Before creating any git commit with code changes.

## Procedure

### 1. Determine Test Scope

Based on the changes made:

- **Single module**: Run tests for that module only
- **Multiple modules**: Run full test suite
- **Documentation only**: Skip tests (but verify doc builds if applicable)
- **Configuration only**: Consider if config changes affect tests

### 2. Run Relevant Tests

#### For Rust Projects

```bash
# Single crate tests
cargo test --package <crate-name>

# Single test
cargo test --test <test-name>

# All tests (may be slow)
cargo test

# Run with output for debugging
cargo test -- --nocapture

# Check compilation without tests
cargo check
```

#### For TypeScript/Node Projects

```bash
# All tests
npm test

# Specific test file
npm test -- <test-file>

# Watch mode for development
npm test -- --watch
```

#### For Python Projects

```bash
# All tests
pytest

# Specific test
pytest tests/test_<name>.py

# With output
pytest -v
```

### 3. Check Results

✅ **All tests must pass** before committing, unless:

- User explicitly says "commit anyway"
- WIP commit on feature branch (document clearly in message)
- Tests are intentionally broken as part of TDD (document in message)

❌ **If tests fail:**

1. Report failures clearly to user
2. Show relevant error messages
3. Ask whether to:
   - Fix failing tests first
   - Commit anyway (with explanation)
   - Create WIP commit on feature branch

### 4. Check for New Warnings

Even if tests pass, check for:

- Compiler warnings
- Linter warnings
- Deprecation warnings

Report these to user, but don't block commit unless severe.

## Exceptions

You may skip tests when:

1. **Documentation-only changes**
   - README updates
   - Comment changes
   - Markdown documentation

2. **User explicitly requests it**
   - "Commit without testing"
   - "Skip tests for now"

3. **Non-code changes**
   - Configuration files (`.toml`, `.json`, `.yaml`)
   - CI/CD workflows
   - Git-related files (`.gitignore`, etc.)

4. **Test infrastructure changes**
   - Modifying test utilities
   - Adding new test fixtures
   - Updating test dependencies

## Fast Test Strategies

For large codebases:

1. **Run affected tests first**
   ```bash
   # Only test changed crates
   cargo test --package vtcode-memory --package vtcode-rag
   ```

2. **Use cargo check for quick validation**
   ```bash
   # Fast compilation check
   cargo check

   # Then run specific tests
   cargo test --test integration_memory
   ```

3. **Parallel test execution**
   ```bash
   # Rust runs tests in parallel by default
   cargo test

   # Limit parallelism if needed
   cargo test -- --test-threads=4
   ```

## Build Verification

In addition to tests, verify:

```bash
# Rust projects
cargo build

# Check for clippy warnings
cargo clippy

# Format check
cargo fmt -- --check
```

## Common Patterns

### Before Commit Checklist

- [ ] Relevant tests pass
- [ ] No new compiler warnings
- [ ] Code compiles successfully
- [ ] Linter is happy (or warnings acknowledged)
- [ ] No debug print statements left in code

### TDD Workflow

When doing Test-Driven Development:

1. Write failing test
2. Implement feature
3. **Run tests** ← Verify test now passes
4. Commit with test + implementation together

### Debugging Failed Tests

```bash
# Show test output
cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test <test-name> -- --nocapture

# Show ignored tests
cargo test -- --ignored
```

## Anti-Patterns

- ❌ Committing without running tests on code changes
- ❌ Ignoring test failures without user consent
- ❌ Running full test suite on trivial changes
- ❌ Disabling tests to make them "pass"
- ❌ Committing commented-out tests

## Integration with Git Commit SOP

This SOP should be run **before** the git-commit SOP:

1. **First**: Run tests (this SOP)
2. **Then**: Create commit (git-commit SOP)

If tests fail, pause and don't proceed to commit unless user explicitly requests it.
