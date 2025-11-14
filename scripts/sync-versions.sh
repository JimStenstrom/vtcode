#!/usr/bin/env bash

# VTCode Version Management Script
#
# This script provides centralized version management across all components:
# - Main Cargo.toml
# - All workspace crate Cargo.toml files
# - Inter-workspace dependency references
# - npm/package.json
# - vscode-extension/package.json
#
# Usage:
#   ./scripts/sync-versions.sh check          # Check if all versions are in sync
#   ./scripts/sync-versions.sh get            # Get the current version
#   ./scripts/sync-versions.sh set <version>  # Set version across all components
#   ./scripts/sync-versions.sh sync           # Sync all versions to match main Cargo.toml

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Helper functions
print_info() {
    printf '%b\n' "${BLUE}INFO:${NC} $1"
}

print_success() {
    printf '%b\n' "${GREEN}SUCCESS:${NC} $1"
}

print_warning() {
    printf '%b\n' "${YELLOW}WARNING:${NC} $1"
}

print_error() {
    printf '%b\n' "${RED}ERROR:${NC} $1"
}

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Get current version from main Cargo.toml
get_current_version() {
    local line
    line=$(grep '^version = ' Cargo.toml | head -n 1)
    echo "${line#*\"}" | sed 's/\".*//'
}

# Get version from a specific file
get_file_version() {
    local file=$1
    local version=""

    if [[ "$file" == *.toml ]]; then
        local line
        line=$(grep '^version = ' "$file" | head -n 1)
        version="${line#*\"}"
        version=$(echo "$version" | sed 's/\".*//')
    elif [[ "$file" == *.json ]]; then
        if command -v jq >/dev/null 2>&1; then
            version=$(jq -r '.version' "$file")
        else
            version=$(grep -o '"version"[[:space:]]*:[[:space:]]*"[^"]*"' "$file" | sed 's/.*"\([^"]*\)".*/\1/')
        fi
    fi

    echo "$version"
}

# Update version in main Cargo.toml
update_main_cargo_version() {
    local new_version=$1

    print_info "Updating main Cargo.toml to version $new_version"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "0,/^version = .*/{s/^version = .*/version = \"$new_version\"/;}" Cargo.toml
    else
        # Linux
        sed -i "0,/^version = .*/{s/^version = .*/version = \"$new_version\"/;}" Cargo.toml
    fi
}

# Update version in workspace crate Cargo.toml
update_workspace_crate_version() {
    local crate_toml=$1
    local new_version=$2

    print_info "Updating $crate_toml to version $new_version"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "0,/^version = .*/{s/^version = .*/version = \"$new_version\"/;}" "$crate_toml"
    else
        # Linux
        sed -i "0,/^version = .*/{s/^version = .*/version = \"$new_version\"/;}" "$crate_toml"
    fi
}

# Update inter-workspace dependency versions
update_workspace_dependencies() {
    local new_version=$1

    print_info "Updating inter-workspace dependency references to version $new_version"

    # List of workspace crates
    local workspace_crates=(
        "vtcode-acp-client"
        "vtcode-core"
        "vtcode-commons"
        "vtcode-config"
        "vtcode-llm"
        "vtcode-markdown-store"
        "vtcode-indexer"
        "vtcode-tools"
        "vtcode-bash-runner"
        "vtcode-exec-events"
    )

    # Update main Cargo.toml dependencies
    for crate in "${workspace_crates[@]}"; do
        if grep -q "$crate.*version = " Cargo.toml; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/\($crate.*version = \)\"[^\"]*\"/\1\"$new_version\"/" Cargo.toml
            else
                sed -i "s/\($crate.*version = \)\"[^\"]*\"/\1\"$new_version\"/" Cargo.toml
            fi
        fi
    done

    # Update workspace crate Cargo.toml files
    for workspace_toml in vtcode-*/Cargo.toml; do
        if [[ -f "$workspace_toml" ]]; then
            for crate in "${workspace_crates[@]}"; do
                if grep -q "$crate.*version = " "$workspace_toml"; then
                    if [[ "$OSTYPE" == "darwin"* ]]; then
                        sed -i '' "s/\($crate.*version = \)\"[^\"]*\"/\1\"$new_version\"/" "$workspace_toml"
                    else
                        sed -i "s/\($crate.*version = \)\"[^\"]*\"/\1\"$new_version\"/" "$workspace_toml"
                    fi
                fi
            done
        fi
    done
}

# Update npm package.json version
update_npm_version() {
    local new_version=$1
    local npm_package="npm/package.json"

    if [[ ! -f "$npm_package" ]]; then
        print_warning "npm package.json not found at $npm_package - skipping"
        return 0
    fi

    print_info "Updating npm/package.json to version $new_version"

    if command -v jq >/dev/null 2>&1; then
        jq --arg new_version "$new_version" '.version = $new_version' "$npm_package" > "$npm_package.tmp"
        mv "$npm_package.tmp" "$npm_package"
    else
        print_warning "jq not found - using sed fallback for npm package.json"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\":[[:space:]]*\"[^\"]*\"/\"version\": \"$new_version\"/" "$npm_package"
        else
            sed -i "s/\"version\":[[:space:]]*\"[^\"]*\"/\"version\": \"$new_version\"/" "$npm_package"
        fi
    fi
}

# Update vscode-extension package.json version
update_vscode_version() {
    local new_version=$1
    local vscode_package="vscode-extension/package.json"

    if [[ ! -f "$vscode_package" ]]; then
        print_warning "vscode-extension package.json not found at $vscode_package - skipping"
        return 0
    fi

    print_info "Updating vscode-extension/package.json to version $new_version"

    if command -v jq >/dev/null 2>&1; then
        jq --arg new_version "$new_version" '.version = $new_version' "$vscode_package" > "$vscode_package.tmp"
        mv "$vscode_package.tmp" "$vscode_package"
    else
        print_warning "jq not found - using sed fallback for vscode-extension package.json"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\":[[:space:]]*\"[^\"]*\"/\"version\": \"$new_version\"/" "$vscode_package"
        else
            sed -i "s/\"version\":[[:space:]]*\"[^\"]*\"/\"version\": \"$new_version\"/" "$vscode_package"
        fi
    fi
}

# Check if all versions are in sync
check_versions() {
    print_info "Checking version consistency across all components..."

    local current_version
    current_version=$(get_current_version)
    print_info "Main Cargo.toml version: $current_version"

    local all_in_sync=true
    local files_checked=0
    local files_out_of_sync=()

    # Check workspace crate Cargo.toml files
    for workspace_toml in vtcode-*/Cargo.toml; do
        if [[ -f "$workspace_toml" ]]; then
            local crate_version
            crate_version=$(get_file_version "$workspace_toml")
            files_checked=$((files_checked + 1))

            if [[ "$crate_version" != "$current_version" ]]; then
                print_warning "$workspace_toml has version $crate_version (expected $current_version)"
                all_in_sync=false
                files_out_of_sync+=("$workspace_toml")
            fi
        fi
    done

    # Check npm package.json
    if [[ -f "npm/package.json" ]]; then
        local npm_version
        npm_version=$(get_file_version "npm/package.json")
        files_checked=$((files_checked + 1))

        if [[ "$npm_version" != "$current_version" ]]; then
            print_warning "npm/package.json has version $npm_version (expected $current_version)"
            all_in_sync=false
            files_out_of_sync+=("npm/package.json")
        fi
    fi

    # Check vscode-extension package.json
    if [[ -f "vscode-extension/package.json" ]]; then
        local vscode_version
        vscode_version=$(get_file_version "vscode-extension/package.json")
        files_checked=$((files_checked + 1))

        if [[ "$vscode_version" != "$current_version" ]]; then
            print_warning "vscode-extension/package.json has version $vscode_version (expected $current_version)"
            all_in_sync=false
            files_out_of_sync+=("vscode-extension/package.json")
        fi
    fi

    print_info "Checked $files_checked files"

    if [[ "$all_in_sync" == true ]]; then
        print_success "All versions are in sync at $current_version"
        return 0
    else
        print_error "Version mismatch detected in ${#files_out_of_sync[@]} file(s):"
        for file in "${files_out_of_sync[@]}"; do
            echo "  - $file"
        done
        print_info "Run './scripts/sync-versions.sh sync' to synchronize all versions"
        return 1
    fi
}

# Sync all versions to match main Cargo.toml
sync_versions() {
    local current_version
    current_version=$(get_current_version)

    print_info "Synchronizing all versions to $current_version (from main Cargo.toml)"

    # Update all workspace crate Cargo.toml files
    for workspace_toml in vtcode-*/Cargo.toml; do
        if [[ -f "$workspace_toml" ]]; then
            update_workspace_crate_version "$workspace_toml" "$current_version"
        fi
    done

    # Update inter-workspace dependencies
    update_workspace_dependencies "$current_version"

    # Update npm package.json
    update_npm_version "$current_version"

    # Update vscode-extension package.json
    update_vscode_version "$current_version"

    print_success "All versions synchronized to $current_version"
}

# Set a new version across all components
set_version() {
    local new_version=$1

    # Validate version format (basic semantic versioning)
    if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        print_error "Invalid version format: $new_version"
        print_info "Expected format: MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-PRERELEASE"
        return 1
    fi

    local current_version
    current_version=$(get_current_version)

    print_info "Updating version from $current_version to $new_version"

    # Update main Cargo.toml
    update_main_cargo_version "$new_version"

    # Update all workspace crate Cargo.toml files
    for workspace_toml in vtcode-*/Cargo.toml; do
        if [[ -f "$workspace_toml" ]]; then
            update_workspace_crate_version "$workspace_toml" "$new_version"
        fi
    done

    # Update inter-workspace dependencies
    update_workspace_dependencies "$new_version"

    # Update npm package.json
    update_npm_version "$new_version"

    # Update vscode-extension package.json
    update_vscode_version "$new_version"

    print_success "All versions updated to $new_version"
    print_info "Remember to run 'cargo update' to update Cargo.lock"
}

# Show usage
show_usage() {
    cat <<'USAGE'
VTCode Version Management Script

Usage: ./scripts/sync-versions.sh <command> [args]

Commands:
  check           Check if all versions are in sync
  get             Get the current version from main Cargo.toml
  set <version>   Set version across all components
  sync            Sync all versions to match main Cargo.toml
  help            Show this help message

Examples:
  ./scripts/sync-versions.sh check
  ./scripts/sync-versions.sh get
  ./scripts/sync-versions.sh set 0.44.0
  ./scripts/sync-versions.sh sync

Source of Truth:
  The main Cargo.toml is the authoritative source for the version.
  All other files are synchronized to match this version.

Files Managed:
  - Cargo.toml (main package)
  - vtcode-*/Cargo.toml (all workspace crates)
  - Inter-workspace dependency references
  - npm/package.json
  - vscode-extension/package.json
USAGE
}

# Main command dispatcher
main() {
    local command=${1:-help}

    case "$command" in
        check)
            check_versions
            ;;
        get)
            get_current_version
            ;;
        set)
            if [[ -z "${2:-}" ]]; then
                print_error "Version argument required"
                echo "Usage: ./scripts/sync-versions.sh set <version>"
                exit 1
            fi
            set_version "$2"
            ;;
        sync)
            sync_versions
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            print_error "Unknown command: $command"
            echo
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
