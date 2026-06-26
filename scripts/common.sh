#!/usr/bin/env bash

# Common utilities and configuration for VT Code scripts

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Cargo configuration to prevent timeouts
export CARGO_HTTP_TIMEOUT=${CARGO_HTTP_TIMEOUT:-300}
export CARGO_NET_RETRY=${CARGO_NET_RETRY:-5}

# Disable OS keyring access during checks/tests so test binaries do not trigger
# repeated macOS Keychain authorization prompts. Credential code falls back to
# encrypted-file storage. Override by exporting VTCODE_DISABLE_KEYRING=0 first.
export VTCODE_DISABLE_KEYRING=${VTCODE_DISABLE_KEYRING:-1}

# Logging functions
print_info() {
    printf '%b\n' "${BLUE}INFO:${NC} $1"
}

print_status() {
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

# Compatibility logging functions (from various scripts)
log_info() {
    printf '%b\n' "${BLUE}ℹ${NC} $1" >&2
}

log_success() {
    printf '%b\n' "${GREEN}✓${NC} $1" >&2
}

log_warning() {
    printf '%b\n' "${YELLOW}⚠${NC} $1" >&2
}

log_error() {
    printf '%b\n' "${RED}✗${NC} $1" >&2
}

# Version detection
get_current_version() {
    local line
    # Try current directory first
    if [[ -f "Cargo.toml" ]]; then
        line=$(grep '^version = ' Cargo.toml | head -1 2>/dev/null || echo "")
    fi
    
    if [[ -z "$line" ]]; then
        # Try to find version in workspace members if not in root
        local root_toml="$(dirname "${BASH_SOURCE[0]}")/../Cargo.toml"
        if [[ -f "$root_toml" ]]; then
            line=$(grep '^version = ' "$root_toml" | head -1 2>/dev/null || echo "")
        fi
    fi
    
    if [[ -z "$line" ]]; then
        # Last resort: find any Cargo.toml
        line=$(find . -maxdepth 2 -name Cargo.toml -exec grep '^version = ' {} + | head -1 2>/dev/null || echo "")
    fi
    
    if [[ -z "$line" ]]; then
        echo "0.82.1" # Fallback to latest known
    else
        echo "${line#*\"}" | sed 's/\".*//'
    fi
}

# Package a release binary into a .tar.gz archive
# Usage: package_release_archive <target> <binary_name> <archive_path>
package_release_archive() {
    local target=$1
    local binary_name=$2
    local archive_path=$3
    local release_dir="target/$target/release"

    tar -C "$release_dir" -czf "$archive_path" "$binary_name"
}

# Update Homebrew formula file with new version and checksums
# Usage: update_homebrew_formula_file <formula_path> <version> <x86_64_sha> <aarch64_sha> [aarch64_linux_sha]
update_homebrew_formula_file() {
    local formula_path=$1
    local version=$2
    local x86_64_macos_sha=$3
    local aarch64_macos_sha=$4
    local aarch64_linux_sha=${5:-}

    FORMULA_PATH="$formula_path" \
    FORMULA_VERSION="$version" \
    FORMULA_X86_64_MACOS_SHA="$x86_64_macos_sha" \
    FORMULA_AARCH64_MACOS_SHA="$aarch64_macos_sha" \
    FORMULA_AARCH64_LINUX_SHA="$aarch64_linux_sha" \
        python3 <<'PYTHON_SCRIPT'
import os
import re
from pathlib import Path

formula_path = Path(os.environ["FORMULA_PATH"])
version = os.environ["FORMULA_VERSION"]
x86_64_macos_sha = os.environ["FORMULA_X86_64_MACOS_SHA"]
aarch64_macos_sha = os.environ["FORMULA_AARCH64_MACOS_SHA"]
aarch64_linux_sha = os.environ.get("FORMULA_AARCH64_LINUX_SHA", "")

content = formula_path.read_text()
content = re.sub(r'version\s+"[^"]*"', f'version "{version}"', content)
content = re.sub(
    r'(aarch64-apple-darwin\.tar\.gz"\s+sha256\s+")([^"]*)(")',
    lambda match: f'{match.group(1)}{aarch64_macos_sha}{match.group(3)}',
    content,
)
content = re.sub(
    r'(x86_64-apple-darwin\.tar\.gz"\s+sha256\s+")([^"]*)(")',
    lambda match: f'{match.group(1)}{x86_64_macos_sha}{match.group(3)}',
    content,
)
if aarch64_linux_sha:
    content = re.sub(
        r'(aarch64-unknown-linux-gnu\.tar\.gz"\s+sha256\s+")([^"]*)(")',
        lambda match: f'{match.group(1)}{aarch64_linux_sha}{match.group(3)}',
        content,
    )

formula_path.write_text(content)
PYTHON_SCRIPT
}
