---
type: standard-operating-procedure
id: git-commit
---

# Creating Git Commits

## When to Use

After completing work and user asks to commit changes.

## Procedure

### 1. Gather Context

Run in parallel:

- `git status` - see what's changed
- `git diff --staged` - see staged changes
- `git log -5 --oneline` - understand commit message style

### 2. Analyze

- Determine change type (feat/fix/refactor/docs/chore/test)
- Draft message < 72 chars describing *why*, not *what*
- Check for sensitive files (.env, credentials, secrets, API keys)
- Verify changes are intentional and complete

### 3. Execute

```bash
# Stage relevant files
git add <files>

# Commit with HEREDOC for proper formatting
git commit -m "$(cat <<'EOF'
<message>
EOF
)"

# Verify clean state
git status
```

### 4. Error Recovery

#### Pre-commit hook changed files?

1. Check authorship: `git log -1 --format='%an %ae'`
2. Check not pushed: `git status` shows "ahead"
3. If both safe: amend commit
4. Otherwise: create new commit

#### Commit failed?

- Report error clearly
- Don't retry automatically
- Ask user for guidance

## Commit Message Format

```
<type>: <subject>

<optional body>

<optional footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring (no behavior change)
- `docs`: Documentation only
- `test`: Adding/updating tests
- `chore`: Tooling, dependencies, config

### Subject Guidelines

- Start with lowercase verb (add, fix, update, remove, etc.)
- No period at end
- Maximum 72 characters
- Describe *why*, not *what* (code shows what)

### Examples

Good:
```
feat: add temporal decay to memory system

Implements exponential decay for older conversation turns
to prioritize recent context while preserving historical summaries.
```

Bad:
```
Update code
```

## Anti-Patterns

- ❌ Generic messages like "update code" or "fix bug"
- ❌ Committing sensitive files (.env, credentials, API keys)
- ❌ Force-amending other developers' commits
- ❌ Using `--no-verify` without explicit user permission
- ❌ Committing broken tests without user acknowledgment
- ❌ Large commits mixing multiple unrelated changes

## Security Checks

Always verify these files are NOT staged:

- `.env` or `.env.*`
- `credentials.json`
- `*_credentials.*`
- `*secret*`
- `*password*`
- `*.pem`, `*.key`, `*.p12`
- API keys or tokens in code

If found, warn user and exclude from commit.

## Best Practices

1. Atomic commits - one logical change per commit
2. Test before commit (see test-before-commit SOP)
3. Write commit messages for future maintainers
4. Link to issues/PRs when relevant
5. Keep commits focused and reviewable
