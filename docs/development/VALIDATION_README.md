# Phase 3 Validation System - Quick Start Guide

This document provides a quick reference for using the Phase 3 validation system.

For comprehensive details, see [VALIDATION_STRATEGY.md](./VALIDATION_STRATEGY.md).

## Overview

The validation system ensures that Phase 3 provider refactoring doesn't introduce regressions, performance degradation, or breaking changes. It consists of:

1. **Regression Tests** - 150+ tests for provider functionality
2. **Performance Benchmarks** - Measure operation performance
3. **Breaking Change Detection** - API surface analysis
4. **Compatibility Matrix** - Provider×Model×Feature validation

## Quick Start

### Before Starting Refactoring (Baseline)

Capture baseline metrics before making any changes:

```bash
# Run validation and save baseline
./scripts/validate-phase3.sh --baseline
```

This creates baseline files in `validation-results/`:
- `api_baseline.txt` - Current public API
- Benchmark baseline (stored by Criterion)
- Test coverage baseline

### During Refactoring

Run validation after each major change:

```bash
# Run full validation suite
./scripts/validate-phase3.sh
```

Or run specific test categories:

```bash
# Regression tests only
cargo test --package vtcode-core --lib llm::providers

# Validation tests only
cargo test --package vtcode-core --test validation_*

# Performance benchmarks only
cargo bench --bench provider_performance

# Compatibility matrix only
cargo test --package vtcode-core --test compatibility_matrix
```

### After Refactoring (Comparison)

Compare final results against baseline:

```bash
# Run validation and compare with baseline
./scripts/validate-phase3.sh --compare
```

## Test Organization

```
vtcode-core/
├── tests/
│   └── validation/
│       ├── mocks/              # Mock providers for testing
│       ├── fixtures/           # Test data and fixtures
│       ├── utils/              # Assertion helpers and utilities
│       ├── provider_regression/# Regression tests
│       ├── provider_edge_cases/# Provider-specific edge cases
│       ├── integration/        # Integration tests
│       └── compatibility_matrix/ # Compatibility matrix framework
└── benches/
    └── provider_performance.rs # Performance benchmarks
```

## Key Commands

### Regression Testing

```bash
# Run all provider tests
cargo test --package vtcode-core --lib llm::providers

# Run specific provider tests
cargo test --package vtcode-core --lib llm::providers::anthropic

# Run with output
cargo test --package vtcode-core --lib llm::providers -- --nocapture

# Run validation regression tests
cargo test --package vtcode-core --test validation_* -- --nocapture
```

### Performance Benchmarking

```bash
# Run all benchmarks
cargo bench --bench provider_performance

# Run specific benchmark group
cargo bench --bench provider_performance -- provider_construction

# Save baseline
cargo bench --bench provider_performance -- --save-baseline before

# Compare against baseline
cargo bench --bench provider_performance -- --baseline before

# Generate flamegraph (requires cargo-flamegraph)
cargo flamegraph --bench provider_performance
```

### Breaking Change Detection

```bash
# Install tools if needed
cargo install cargo-public-api
cargo install cargo-semver-checks

# Check public API
cargo public-api --simplified > api_current.txt

# Compare with baseline
diff api_baseline.txt api_current.txt

# Check for semver violations
cargo semver-checks check-release
```

### Compatibility Matrix

```bash
# Run full compatibility matrix
cargo test --package vtcode-core --test compatibility_matrix -- --nocapture

# Run for specific provider
cargo test --package vtcode-core --test compatibility_matrix -- anthropic --nocapture

# Generate report
cargo test --package vtcode-core --test compatibility_matrix -- --show-output > compatibility_report.md
```

### Test Coverage

```bash
# Install tarpaulin if needed
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin \
  --package vtcode-core \
  --out Lcov \
  --output-dir coverage

# Generate HTML report
cargo tarpaulin \
  --package vtcode-core \
  --out Html \
  --output-dir coverage
```

## Validation Results

All validation results are saved to `validation-results/` with timestamps:

```
validation-results/
├── validation_report_YYYYMMDD_HHMMSS.md  # Summary report
├── regression_YYYYMMDD_HHMMSS.txt        # Regression test output
├── benchmarks_YYYYMMDD_HHMMSS.txt        # Benchmark results
├── api_diff_YYYYMMDD_HHMMSS.txt          # API changes
├── semver_YYYYMMDD_HHMMSS.txt            # Semver check results
├── compatibility_YYYYMMDD_HHMMSS.md      # Compatibility matrix
└── coverage_YYYYMMDD_HHMMSS/             # Coverage reports
```

## Success Criteria

For validation to pass:

- ✅ All regression tests must pass
- ✅ No operation should be >10% slower
- ✅ Memory usage should not increase >5%
- ✅ No undocumented breaking changes
- ✅ All expected provider features still work
- ✅ Code coverage ≥ baseline

## Common Issues

### Tests Failing

1. Check test output in `validation-results/regression_*.txt`
2. Run specific failing test with `--nocapture` for more detail
3. Use `RUST_BACKTRACE=1` for stack traces

### Benchmarks Show Regression

1. Check benchmark output in `validation-results/benchmarks_*.txt`
2. Profile with `cargo flamegraph` to identify hotspots
3. Review changes in performance-critical paths

### API Changes Detected

1. Review `validation-results/api_diff_*.txt`
2. Document all breaking changes in CHANGELOG.md
3. Add deprecation warnings where possible
4. Update migration guide

### Compatibility Matrix Failures

1. Check `validation-results/compatibility_*.md`
2. Verify provider-specific edge cases
3. Test manually with real API keys if needed

## CI/CD Integration

The validation system can be integrated into CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Run Phase 3 Validation
  run: ./scripts/validate-phase3.sh

- name: Upload Validation Results
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: validation-results
    path: validation-results/
```

## Development Workflow

Recommended workflow during Phase 3:

1. **Before starting**: Run `./scripts/validate-phase3.sh --baseline`
2. **Daily**: Run `cargo test --package vtcode-core --lib llm::providers`
3. **Before PR**: Run `./scripts/validate-phase3.sh`
4. **Before merge**: Run `./scripts/validate-phase3.sh --compare`

## Getting Help

- **Full Documentation**: See [VALIDATION_STRATEGY.md](./VALIDATION_STRATEGY.md)
- **Critical Review**: See [PHASE_3_CRITICAL_REVIEW.md](./PHASE_3_CRITICAL_REVIEW.md)
- **Architecture Plan**: See [ARCHITECTURE_TRANSFORMATION.md](./ARCHITECTURE_TRANSFORMATION.md)

## Tool Installation

Install all validation tools at once:

```bash
# Core testing tools
cargo install cargo-tarpaulin
cargo install cargo-public-api
cargo install cargo-semver-checks
cargo install cargo-flamegraph
cargo install criterion

# Optional profiling tools (Linux)
sudo apt-get install valgrind

# Optional profiling tools (macOS)
# Use Instruments.app (part of Xcode)
```

## Quick Reference

| Task | Command |
|------|---------|
| Full validation | `./scripts/validate-phase3.sh` |
| Capture baseline | `./scripts/validate-phase3.sh --baseline` |
| Compare with baseline | `./scripts/validate-phase3.sh --compare` |
| Run regression tests | `cargo test --package vtcode-core --lib llm::providers` |
| Run benchmarks | `cargo bench --bench provider_performance` |
| Check breaking changes | `cargo semver-checks check-release` |
| Run compatibility matrix | `cargo test --package vtcode-core --test compatibility_matrix` |
| Generate coverage | `cargo tarpaulin --package vtcode-core --out Html` |

---

**Last Updated**: 2025-11-13
**Version**: 1.0
