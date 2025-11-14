#!/bin/bash
#
# Phase 3 Validation Script
#
# This script runs the complete validation suite for Phase 3 provider refactoring.
# It includes regression tests, performance benchmarks, breaking change detection,
# and compatibility matrix generation.
#
# Usage:
#   ./scripts/validate-phase3.sh [--baseline|--compare]
#
# Options:
#   --baseline  Capture baseline metrics (run before refactoring)
#   --compare   Compare against baseline (run after refactoring)
#   (no args)   Run full validation suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VALIDATION_DIR="$PROJECT_ROOT/validation-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Create validation results directory
mkdir -p "$VALIDATION_DIR"

# Function to run regression tests
run_regression_tests() {
    print_header "Running Regression Tests"

    cd "$PROJECT_ROOT"

    if cargo test --package vtcode-core --lib llm::providers -- --nocapture > "$VALIDATION_DIR/regression_${TIMESTAMP}.txt" 2>&1; then
        print_success "Regression tests passed"
        REGRESSION_PASSED=true
    else
        print_error "Regression tests failed"
        print_warning "See $VALIDATION_DIR/regression_${TIMESTAMP}.txt for details"
        REGRESSION_PASSED=false
    fi

    # Run validation-specific tests
    if cargo test --package vtcode-core --test validation_* -- --nocapture >> "$VALIDATION_DIR/regression_${TIMESTAMP}.txt" 2>&1; then
        print_success "Validation tests passed"
    else
        print_error "Validation tests failed"
        REGRESSION_PASSED=false
    fi

    return $([ "$REGRESSION_PASSED" = true ] && echo 0 || echo 1)
}

# Function to run performance benchmarks
run_benchmarks() {
    print_header "Running Performance Benchmarks"

    cd "$PROJECT_ROOT"

    local baseline_flag=""
    if [ "${1:-}" = "--baseline" ]; then
        baseline_flag="--save-baseline before"
        print_warning "Saving baseline metrics"
    elif [ "${1:-}" = "--compare" ]; then
        baseline_flag="--baseline before"
        print_warning "Comparing against baseline"
    fi

    if cargo bench --bench provider_performance -- $baseline_flag --noplot > "$VALIDATION_DIR/benchmarks_${TIMESTAMP}.txt" 2>&1; then
        print_success "Benchmarks completed"
        BENCHMARKS_PASSED=true
    else
        print_error "Benchmarks failed"
        print_warning "See $VALIDATION_DIR/benchmarks_${TIMESTAMP}.txt for details"
        BENCHMARKS_PASSED=false
    fi

    return $([ "$BENCHMARKS_PASSED" = true ] && echo 0 || echo 1)
}

# Function to check for breaking changes
check_breaking_changes() {
    print_header "Checking for Breaking Changes"

    cd "$PROJECT_ROOT"

    # Check if cargo-public-api is installed
    if ! command -v cargo-public-api &> /dev/null; then
        print_warning "cargo-public-api not installed, skipping API analysis"
        print_warning "Install with: cargo install cargo-public-api"
        return 0
    fi

    # Capture current API surface
    cargo public-api --simplified > "$VALIDATION_DIR/api_surface_${TIMESTAMP}.txt" 2>&1

    # Compare with baseline if it exists
    if [ -f "$VALIDATION_DIR/api_baseline.txt" ]; then
        if diff "$VALIDATION_DIR/api_baseline.txt" "$VALIDATION_DIR/api_surface_${TIMESTAMP}.txt" > "$VALIDATION_DIR/api_diff_${TIMESTAMP}.txt" 2>&1; then
            print_success "No API changes detected"
            API_CHANGES=false
        else
            print_warning "API changes detected"
            print_warning "See $VALIDATION_DIR/api_diff_${TIMESTAMP}.txt for details"
            API_CHANGES=true
        fi
    else
        print_warning "No baseline API surface found"
        print_warning "Saving current API as baseline"
        cp "$VALIDATION_DIR/api_surface_${TIMESTAMP}.txt" "$VALIDATION_DIR/api_baseline.txt"
        API_CHANGES=false
    fi

    # Check if cargo-semver-checks is installed
    if ! command -v cargo-semver-checks &> /dev/null; then
        print_warning "cargo-semver-checks not installed, skipping semver analysis"
        print_warning "Install with: cargo install cargo-semver-checks"
        return 0
    fi

    # Run semver checks
    if cargo semver-checks check-release > "$VALIDATION_DIR/semver_${TIMESTAMP}.txt" 2>&1; then
        print_success "No breaking changes detected by semver-checks"
    else
        print_error "Breaking changes detected by semver-checks"
        print_warning "See $VALIDATION_DIR/semver_${TIMESTAMP}.txt for details"
        API_CHANGES=true
    fi

    return $([ "$API_CHANGES" = false ] && echo 0 || echo 1)
}

# Function to run compatibility matrix tests
run_compatibility_matrix() {
    print_header "Running Compatibility Matrix"

    cd "$PROJECT_ROOT"

    if cargo test --package vtcode-core --test compatibility_matrix -- --nocapture --show-output > "$VALIDATION_DIR/compatibility_${TIMESTAMP}.md" 2>&1; then
        print_success "Compatibility matrix generated"
        COMPAT_PASSED=true
    else
        print_error "Compatibility matrix generation failed"
        print_warning "See $VALIDATION_DIR/compatibility_${TIMESTAMP}.md for details"
        COMPAT_PASSED=false
    fi

    return $([ "$COMPAT_PASSED" = true ] && echo 0 || echo 1)
}

# Function to generate test coverage report
generate_coverage() {
    print_header "Generating Test Coverage Report"

    cd "$PROJECT_ROOT"

    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not installed, skipping coverage"
        print_warning "Install with: cargo install cargo-tarpaulin"
        return 0
    fi

    local coverage_dir="$VALIDATION_DIR/coverage_${TIMESTAMP}"
    mkdir -p "$coverage_dir"

    if cargo tarpaulin \
        --package vtcode-core \
        --out Lcov \
        --output-dir "$coverage_dir" \
        > "$VALIDATION_DIR/coverage_${TIMESTAMP}.txt" 2>&1; then
        print_success "Coverage report generated"

        # Extract coverage percentage
        local coverage=$(grep -oP 'Coverage: \K[0-9.]+' "$VALIDATION_DIR/coverage_${TIMESTAMP}.txt" | head -1)
        if [ -n "$coverage" ]; then
            print_success "Coverage: ${coverage}%"
            echo "$coverage" > "$VALIDATION_DIR/coverage_${TIMESTAMP}.percent"
        fi
    else
        print_warning "Coverage generation failed"
    fi
}

# Function to generate validation report
generate_report() {
    print_header "Generating Validation Report"

    local report_file="$VALIDATION_DIR/validation_report_${TIMESTAMP}.md"

    cat > "$report_file" << EOF
# Phase 3 Validation Report

## Summary
- **Date**: $(date '+%Y-%m-%d %H:%M:%S')
- **Status**: ${VALIDATION_STATUS:-UNKNOWN}
- **Duration**: ${VALIDATION_DURATION:-N/A}

## Test Results

### Regression Tests
- **Status**: $([ "${REGRESSION_PASSED:-false}" = true ] && echo "✅ PASS" || echo "❌ FAIL")
- **Details**: See \`regression_${TIMESTAMP}.txt\`

### Performance Benchmarks
- **Status**: $([ "${BENCHMARKS_PASSED:-false}" = true ] && echo "✅ PASS" || echo "⚠️ WARN")
- **Details**: See \`benchmarks_${TIMESTAMP}.txt\`

### API Stability
- **Changes Detected**: $([ "${API_CHANGES:-false}" = true ] && echo "Yes" || echo "No")
- **Details**: See \`api_diff_${TIMESTAMP}.txt\` and \`semver_${TIMESTAMP}.txt\`

### Compatibility Matrix
- **Status**: $([ "${COMPAT_PASSED:-false}" = true ] && echo "✅ PASS" || echo "❌ FAIL")
- **Details**: See \`compatibility_${TIMESTAMP}.md\`

### Test Coverage
- **Coverage**: $(cat "$VALIDATION_DIR/coverage_${TIMESTAMP}.percent" 2>/dev/null || echo "N/A")%
- **Details**: See \`coverage_${TIMESTAMP}/\`

## Recommendation

EOF

    if [ "${REGRESSION_PASSED:-false}" = true ] && \
       [ "${BENCHMARKS_PASSED:-false}" = true ] && \
       [ "${API_CHANGES:-false}" = false ] && \
       [ "${COMPAT_PASSED:-false}" = true ]; then
        echo "✅ **APPROVED FOR MERGE**" >> "$report_file"
        echo "" >> "$report_file"
        echo "All validation checks passed successfully." >> "$report_file"
    else
        echo "❌ **REQUIRES FIXES**" >> "$report_file"
        echo "" >> "$report_file"
        echo "One or more validation checks failed. Review the failures above." >> "$report_file"
    fi

    echo "" >> "$report_file"
    echo "## Files Generated" >> "$report_file"
    echo "" >> "$report_file"
    ls -1 "$VALIDATION_DIR" | grep "$TIMESTAMP" | while read file; do
        echo "- \`$file\`" >> "$report_file"
    done

    print_success "Report generated: $report_file"

    # Print report summary
    echo ""
    cat "$report_file"
}

# Main execution
main() {
    local mode="${1:-full}"

    print_header "Phase 3 Validation Suite"
    echo "Mode: $mode"
    echo "Results directory: $VALIDATION_DIR"

    START_TIME=$(date +%s)

    case "$mode" in
        --baseline)
            print_warning "Baseline mode: Capturing baseline metrics"
            run_regression_tests
            run_benchmarks --baseline
            check_breaking_changes
            run_compatibility_matrix
            generate_coverage
            ;;
        --compare)
            print_warning "Compare mode: Comparing against baseline"
            run_regression_tests
            run_benchmarks --compare
            check_breaking_changes
            run_compatibility_matrix
            generate_coverage
            ;;
        *)
            print_warning "Full validation mode"
            run_regression_tests
            run_benchmarks
            check_breaking_changes
            run_compatibility_matrix
            generate_coverage
            ;;
    esac

    END_TIME=$(date +%s)
    VALIDATION_DURATION=$((END_TIME - START_TIME))

    if [ "${REGRESSION_PASSED:-false}" = true ] && [ "${COMPAT_PASSED:-false}" = true ]; then
        VALIDATION_STATUS="PASS"
    else
        VALIDATION_STATUS="FAIL"
    fi

    generate_report

    print_header "Validation Complete"
    print_success "Duration: ${VALIDATION_DURATION}s"
    print_success "Results saved to: $VALIDATION_DIR"

    # Exit with appropriate code
    if [ "$VALIDATION_STATUS" = "PASS" ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"
