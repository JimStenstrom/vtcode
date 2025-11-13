# VT Code Benchmarks

This directory contains benchmark results and documentation for evaluating VT Code's code generation capabilities.

> **⚠️ IMPORTANT:** This directory contains a mix of real benchmark results and placeholder/aspirational content. Files referencing models like "gpt-5-nano", "gpt-5", "grok-4", etc. contain **example/test data** for documentation purposes only. These models do not exist publicly. Always verify benchmark authenticity before making decisions based on this data.

## Overview

VT Code is evaluated on industry-standard benchmarks to measure:

-   **Code Generation Quality**: Correctness and functionality of generated code
-   **Performance**: Response latency and throughput
-   **Cost Efficiency**: Token usage and API costs across providers

## HumanEval Benchmark

[HumanEval](https://github.com/openai/human-eval) is a benchmark for evaluating code generation models on 164 hand-written programming problems. Each problem includes:

-   Function signature and docstring
-   Unit tests to verify correctness
-   Pass@1 metric (percentage of problems solved on first attempt)

### Running Benchmarks

#### Prerequisites

```bash
# Install Python dependencies
pip install datasets

# Ensure vtcode is built
cargo build --release
```

#### Basic Usage

```bash
# Run full benchmark (164 tasks)
make bench-humaneval PROVIDER=<provider> MODEL='<model>'

# Run subset for quick testing
make bench-humaneval PROVIDER=<provider> MODEL='<model>' N_HE=10

# Run with custom parameters
make bench-humaneval \
  PROVIDER=openai \
  MODEL='gpt-4o' \
  N_HE=50 \
  SEED=42 \
  SLEEP_MS=500 \
  RETRY_MAX=3
```

#### Supported Models (v0.43.6)

Real models available for benchmarking:

**OpenAI:**
- gpt-4o, gpt-4o-mini
- gpt-3.5-turbo

**Anthropic:**
- claude-3-5-sonnet-20241022
- claude-3-opus-20240229
- claude-3-haiku-20240307

**Google:**
- gemini-2.0-flash-exp
- gemini-1.5-pro
- gemini-1.5-flash

**Others:**
- DeepSeek models
- xAI Grok models
- Ollama (local models)

See [PROVIDER_GUIDES.md](../PROVIDER_GUIDES.md) for complete provider setup.

#### Environment Variables

| Variable       | Default     | Description                                    |
| -------------- | ----------- | ---------------------------------------------- |
| `PROVIDER`     | `gemini`    | LLM provider (gemini, openai, anthropic, etc.) |
| `MODEL`        | (required)  | Model identifier                               |
| `N_HE`         | `164`       | Number of tasks to run (max 164)               |
| `SEED`         | `1337`      | Random seed for reproducibility                |
| `USE_TOOLS`    | `0`         | Enable tool usage (0=disabled, 1=enabled)      |
| `TEMP`         | `0.0`       | Temperature for sampling                       |
| `MAX_OUT`      | `1024`      | Maximum output tokens                          |
| `TIMEOUT_S`    | `120`       | Timeout per task in seconds                    |
| `SLEEP_MS`     | `0`         | Sleep between tasks (ms)                       |
| `RETRY_MAX`    | `2`         | Maximum retry attempts                         |
| `BACKOFF_MS`   | `500`       | Backoff delay for retries (ms)                 |
| `INPUT_PRICE`  | `0.0`       | Cost per 1k input tokens (USD)                 |
| `OUTPUT_PRICE` | `0.0`       | Cost per 1k output tokens (USD)                |

#### Visualization

Generate charts and summaries from results:

```bash
# Generate ASCII chart and markdown summary
python3 scripts/render_benchmark_chart.py reports/HE_*.json

# View latest results
cat reports/HE_*_summary.md
```

### Results Archive

All benchmark results are stored in the `reports/` directory with the naming convention:

```
HE_YYYYMMDD-HHMMSS_<model>_tools-<0|1>_N<count>.json
```

Each report includes:

-   Metadata (model, provider, configuration)
-   Summary statistics (pass@1, latency, cost)
-   Individual task results (passed/failed, errors, timing)

### Verifying Benchmark Authenticity

To ensure benchmark results are real:

1. **Check model names** against publicly available LLMs
2. **Verify dates** are in the past and reasonable
3. **Look for JSON reports** in `reports/` directory
4. **Run benchmarks yourself** to reproduce results
5. **Be skeptical of extraordinary claims** (e.g., >90% pass rates at very low cost)

### Known Issues

1. **Token Counting**: vtcode doesn't currently report token usage from the LLM API
2. **Some benchmark docs contain placeholder data** - verify before use

### Future Work

-   [ ] Add support for more benchmarks (MBPP, CodeContests)
-   [ ] Multi-model comparison dashboard
-   [ ] Token usage tracking and reporting
-   [ ] Cost optimization analysis
-   [ ] Performance profiling and optimization
-   [ ] Replace placeholder benchmark data with real results

## Rust Performance Benchmarks

Internal performance benchmarks for VT Code are located in `/benches/` (project root), using Criterion.rs:

```bash
# Run all Rust benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench search_benchmark
```

Available benchmarks:
- `search_benchmark.rs` - Search performance across file sizes
- `system_benchmarks.rs` - System-level performance tests
- `tree_sitter_benchmark.rs` - Tree-sitter parsing performance

These measure VT Code's internal performance, not LLM code generation quality.

## Methodology

1. **Dataset**: Complete HumanEval dataset (164 problems)
2. **Evaluation**: Automated test execution with Python unittest
3. **Reproducibility**: Fixed seed for deterministic sampling
4. **Rate Limiting**: Configurable sleep between tasks to respect API limits

## Contributing

To add real benchmark results:

1. Run benchmark with an actual, publicly available model
2. Verify results are reproducible (run 2-3 times)
3. Include raw JSON reports in `reports/`
4. Document exact configuration and environment
5. **Do not submit placeholder or aspirational data**
6. Submit PR with results and analysis

## References

-   [HumanEval Paper](https://arxiv.org/abs/2107.03374) - Original benchmark paper
-   [OpenAI HumanEval](https://github.com/openai/human-eval) - Official implementation
-   [Criterion.rs](https://github.com/bheisler/criterion.rs) - Rust benchmarking library
-   [Benchmark Scripts](../../scripts/) - VT Code benchmark implementations
