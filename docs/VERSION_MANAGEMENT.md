# Version Management Strategy

## Overview

VTCode uses a **centralized version management strategy** to ensure consistency across all project components. This document describes the versioning approach, tools, and workflows used to maintain version synchronization.

## Problem Statement

VTCode is a multi-component project consisting of:

- **Rust Workspace**: Main binary and 10 workspace crates
- **NPM Package**: Distribution via npm and GitHub Packages
- **VSCode Extension**: IDE integration
- **Inter-dependencies**: Workspace crates depend on each other with explicit versions

**Previous Challenge**: No automated strategy existed for synchronizing versions across all these components, leading to potential inconsistencies and manual errors during releases.

## Solution: Centralized Version Management

### Single Source of Truth

**`Cargo.toml`** (main project) is the authoritative source for the version number.

All other components synchronize to this version:
- Workspace crate `Cargo.toml` files
- Inter-workspace dependency references
- `npm/package.json`
- `vscode-extension/package.json`

### Version Management Tool

The **`scripts/sync-versions.sh`** script provides centralized version management:

```bash
# Check if all versions are in sync
./scripts/sync-versions.sh check

# Get current version
./scripts/sync-versions.sh get

# Set a new version across all components
./scripts/sync-versions.sh set 0.44.0

# Synchronize all versions to match main Cargo.toml
./scripts/sync-versions.sh sync
```

## Workflow Integration

### Pre-Release Check

Before any release, the version consistency check runs automatically:

```bash
./scripts/release.sh
```

The release script:
1. ✅ Checks that all versions are in sync
2. ✅ Runs `cargo-release` to update main version
3. ✅ Synchronizes all component versions automatically
4. ✅ Commits version changes
5. ✅ Updates `Cargo.lock`
6. ✅ Pushes to remote

### Manual Version Update

To manually update the version:

```bash
# Set new version across all components
./scripts/sync-versions.sh set 0.44.0

# Update Cargo.lock
cargo update --workspace

# Commit changes
git add -A
git commit -m "chore: bump version to 0.44.0"
```

### Version Check in CI/CD

Add version consistency checks to your CI pipeline:

```yaml
- name: Check version consistency
  run: ./scripts/sync-versions.sh check
```

## Components Managed

### Rust Components

| Component | Location | Description |
|-----------|----------|-------------|
| Main Binary | `Cargo.toml` | Primary package (source of truth) |
| vtcode-core | `vtcode-core/Cargo.toml` | Core library |
| vtcode-commons | `vtcode-commons/Cargo.toml` | Shared utilities |
| vtcode-config | `vtcode-config/Cargo.toml` | Configuration |
| vtcode-llm | `vtcode-llm/Cargo.toml` | LLM integration |
| vtcode-indexer | `vtcode-indexer/Cargo.toml` | Code indexing |
| vtcode-tools | `vtcode-tools/Cargo.toml` | Tool definitions |
| vtcode-markdown-store | `vtcode-markdown-store/Cargo.toml` | Markdown storage |
| vtcode-bash-runner | `vtcode-bash-runner/Cargo.toml` | Bash execution |
| vtcode-exec-events | `vtcode-exec-events/Cargo.toml` | Event handling |
| vtcode-acp-client | `vtcode-acp-client/Cargo.toml` | ACP client |

### Distribution Components

| Component | Location | Description |
|-----------|----------|-------------|
| NPM Package | `npm/package.json` | npm distribution |
| VSCode Extension | `vscode-extension/package.json` | IDE extension |

### Inter-Workspace Dependencies

The script also manages version references in dependencies:

```toml
[dependencies]
vtcode-core = { path = "vtcode-core", version = "0.43.6" }
vtcode-commons = { path = "../vtcode-commons", version = "0.43.6" }
```

When the version changes to `0.44.0`, these references are automatically updated:

```toml
[dependencies]
vtcode-core = { path = "vtcode-core", version = "0.44.0" }
vtcode-commons = { path = "../vtcode-commons", version = "0.44.0" }
```

## Semantic Versioning

VTCode follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR**: Breaking changes (e.g., `1.0.0` → `2.0.0`)
- **MINOR**: New features, backwards-compatible (e.g., `1.0.0` → `1.1.0`)
- **PATCH**: Bug fixes, backwards-compatible (e.g., `1.0.0` → `1.0.1`)
- **PRERELEASE**: Pre-release versions (e.g., `1.0.0-alpha.1`, `1.0.0-beta.2`)

### Release Commands

```bash
# Patch release (0.43.6 → 0.43.7)
./scripts/release.sh patch

# Minor release (0.43.6 → 0.44.0)
./scripts/release.sh minor

# Major release (0.43.6 → 1.0.0)
./scripts/release.sh major

# Pre-release (0.43.6 → 0.44.0-alpha.1)
./scripts/release.sh --pre-release
```

## Best Practices

### ✅ Do's

1. **Always use the version management script** for version updates
2. **Run version check** before creating pull requests that modify versions
3. **Use semantic versioning** conventions
4. **Keep Cargo.lock updated** after version changes
5. **Commit version changes separately** from feature changes

### ❌ Don'ts

1. **Don't manually edit** version numbers in individual files
2. **Don't skip** the version consistency check
3. **Don't mix** version updates with other changes in the same commit
4. **Don't forget** to update `Cargo.lock` after version changes

## Troubleshooting

### Version Mismatch Detected

If the version check fails:

```bash
$ ./scripts/sync-versions.sh check
WARNING: vtcode-core/Cargo.toml has version 0.43.5 (expected 0.43.6)
ERROR: Version mismatch detected in 1 file(s)
```

**Solution**: Synchronize versions:

```bash
./scripts/sync-versions.sh sync
cargo update --workspace
git add -A
git commit -m "chore: synchronize versions"
```

### Manual Version Update Failed

If manual version update fails:

1. Check file permissions
2. Verify `jq` is installed (for JSON files)
3. Check for syntax errors in TOML/JSON files
4. Run with verbose output for debugging

### Release Script Version Sync Issues

If the release script fails during version sync:

1. Check that `scripts/sync-versions.sh` exists and is executable
2. Verify git working tree is clean
3. Ensure you're on the `main` branch
4. Check for file permission issues

## Architecture Decision Records

### Why Cargo.toml as Source of Truth?

**Decision**: Use main `Cargo.toml` as the authoritative version source.

**Rationale**:
- Cargo.toml is the primary build configuration
- `cargo-release` already updates this file
- Rust workspace is the core of the project
- Simplifies integration with existing release workflow

**Alternatives Considered**:
- Separate `VERSION` file: Additional file to maintain
- Package.json as source: Would complicate Rust workflow
- Multiple sources: Risk of inconsistency

### Why Bash Script Instead of Rust Binary?

**Decision**: Implement version management as a bash script.

**Rationale**:
- Lightweight and fast
- No compilation required
- Easy to integrate with existing release scripts
- Shell scripts are standard for build/release automation

**Alternatives Considered**:
- Rust binary: Overkill for simple file updates
- Python script: Additional dependency
- Make/Makefile: Less portable, more complex syntax

## Future Enhancements

### Planned Improvements

1. **Pre-commit Hook**: Automatically check version consistency
2. **GitHub Action**: Validate versions in PR checks
3. **Version Bump Assistant**: Interactive CLI for guided version updates
4. **Changelog Integration**: Auto-update changelog with version
5. **Dependency Version Check**: Validate external dependency versions

### Extension Points

The version management system is designed to be extensible:

```bash
# Add new component to sync
# Edit scripts/sync-versions.sh and add to the component list

# Custom validation rules
# Add validation functions to check_versions()

# Integration with other tools
# Call sync-versions.sh from your custom scripts
```

## References

- [Semantic Versioning](https://semver.org/)
- [Cargo Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [Release Workflow](../RELEASING.md)

## Support

For issues or questions about version management:

1. Check this documentation
2. Review [RELEASING.md](../RELEASING.md)
3. File an issue on GitHub
4. Contact the maintainers

---

**Last Updated**: 2025-11-13
**Version**: 1.0.0
**Status**: ✅ Active
