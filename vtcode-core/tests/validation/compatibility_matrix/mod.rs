//! Provider compatibility matrix testing framework
//!
//! This module provides a comprehensive testing framework for validating
//! all provider × model × feature combinations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vtcode_core::config::constants::models;
use vtcode_core::llm::provider::{LLMError, LLMProvider, LLMRequest};

// ============================================================================
// Feature Definitions
// ============================================================================

/// Features that can be tested across providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    /// Basic text generation
    Basic,
    /// Streaming responses
    Streaming,
    /// Tool/function calling
    Tools,
    /// Vision/multimodal input
    Vision,
    /// Reasoning/thinking mode
    Reasoning,
    /// Prompt caching
    PromptCache,
    /// Parallel tool calls
    ParallelTools,
    /// Custom configuration
    CustomConfig,
}

impl Feature {
    pub fn all() -> Vec<Feature> {
        vec![
            Feature::Basic,
            Feature::Streaming,
            Feature::Tools,
            Feature::Vision,
            Feature::Reasoning,
            Feature::PromptCache,
            Feature::ParallelTools,
            Feature::CustomConfig,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Feature::Basic => "Basic",
            Feature::Streaming => "Streaming",
            Feature::Tools => "Tools",
            Feature::Vision => "Vision",
            Feature::Reasoning => "Reasoning",
            Feature::PromptCache => "PromptCache",
            Feature::ParallelTools => "ParallelTools",
            Feature::CustomConfig => "CustomConfig",
        }
    }
}

// ============================================================================
// Test Expectations
// ============================================================================

/// Expected result for a compatibility test
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestExpectation {
    /// Feature is fully supported
    Supported,
    /// Feature is not supported
    Unsupported,
    /// Feature is partially supported (with limitations)
    PartiallySupported,
}

// ============================================================================
// Compatibility Test Definition
// ============================================================================

/// A single compatibility test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityTest {
    pub provider: &'static str,
    pub model: String,
    pub feature: Feature,
    pub expected: TestExpectation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl CompatibilityTest {
    pub fn new(
        provider: &'static str,
        model: impl Into<String>,
        feature: Feature,
        expected: TestExpectation,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            feature,
            expected,
            notes: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn id(&self) -> String {
        format!("{}::{}::{:?}", self.provider, self.model, self.feature)
    }
}

// ============================================================================
// Test Result
// ============================================================================

/// Result of running a compatibility test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test: CompatibilityTest,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u128,
}

impl TestResult {
    pub fn success(test: CompatibilityTest, duration_ms: u128) -> Self {
        Self {
            test,
            success: true,
            error: None,
            duration_ms,
        }
    }

    pub fn failure(test: CompatibilityTest, error: String, duration_ms: u128) -> Self {
        Self {
            test,
            success: false,
            error: Some(error),
            duration_ms,
        }
    }

    pub fn matches_expectation(&self) -> bool {
        match self.test.expected {
            TestExpectation::Supported => self.success,
            TestExpectation::Unsupported => !self.success,
            TestExpectation::PartiallySupported => true, // Partial support is acceptable either way
        }
    }
}

// ============================================================================
// Compatibility Matrix Definition
// ============================================================================

/// Generate the complete compatibility matrix for all providers
pub fn generate_compatibility_matrix() -> Vec<CompatibilityTest> {
    let mut tests = Vec::new();

    // Anthropic
    tests.extend(anthropic_tests());

    // OpenAI
    tests.extend(openai_tests());

    // Gemini
    tests.extend(gemini_tests());

    // OpenRouter
    tests.extend(openrouter_tests());

    // XAI
    tests.extend(xai_tests());

    // Moonshot
    tests.extend(moonshot_tests());

    // DeepSeek
    tests.extend(deepseek_tests());

    // ZAI
    tests.extend(zai_tests());

    // Ollama
    tests.extend(ollama_tests());

    // LMStudio
    tests.extend(lmstudio_tests());

    // Minimax
    tests.extend(minimax_tests());

    tests
}

// ============================================================================
// Provider-Specific Test Definitions
// ============================================================================

fn anthropic_tests() -> Vec<CompatibilityTest> {
    vec![
        // Claude Sonnet 4.5
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::Tools,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::Vision,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::Reasoning,
            TestExpectation::Supported,
        )
        .with_notes("Extended thinking mode supported"),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::PromptCache,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::ParallelTools,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_SONNET_4_5,
            Feature::CustomConfig,
            TestExpectation::Supported,
        ),
        // Claude Haiku 4.5
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_HAIKU_4_5,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_HAIKU_4_5,
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "anthropic",
            models::CLAUDE_HAIKU_4_5,
            Feature::Tools,
            TestExpectation::Supported,
        ),
        // Minimax via Anthropic wrapper
        CompatibilityTest::new(
            "anthropic",
            models::minimax::MINIMAX_M2,
            Feature::Basic,
            TestExpectation::Supported,
        )
        .with_notes("Minimax uses special URL routing"),
        CompatibilityTest::new(
            "anthropic",
            models::minimax::MINIMAX_M2,
            Feature::Tools,
            TestExpectation::Supported,
        )
        .with_notes("Tools use XML format"),
    ]
}

fn openai_tests() -> Vec<CompatibilityTest> {
    vec![
        // GPT-5
        CompatibilityTest::new("openai", "gpt-5", Feature::Basic, TestExpectation::Supported),
        CompatibilityTest::new("openai", "gpt-5", Feature::Streaming, TestExpectation::Supported),
        CompatibilityTest::new("openai", "gpt-5", Feature::Tools, TestExpectation::Supported),
        CompatibilityTest::new("openai", "gpt-5", Feature::Vision, TestExpectation::Supported),
        CompatibilityTest::new("openai", "gpt-5", Feature::Reasoning, TestExpectation::Supported)
            .with_notes("Responses API with reasoning support"),
        CompatibilityTest::new(
            "openai",
            "gpt-5",
            Feature::PromptCache,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "openai",
            "gpt-5",
            Feature::ParallelTools,
            TestExpectation::Supported,
        )
        .with_notes("parallel_tool_calls parameter supported"),
        CompatibilityTest::new(
            "openai",
            "gpt-5",
            Feature::CustomConfig,
            TestExpectation::Supported,
        ),
        // GPT-5-mini
        CompatibilityTest::new(
            "openai",
            "gpt-5-mini",
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "openai",
            "gpt-5-mini",
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "openai",
            "gpt-5-mini",
            Feature::Tools,
            TestExpectation::Supported,
        ),
    ]
}

fn gemini_tests() -> Vec<CompatibilityTest> {
    vec![
        // Gemini 2.5 Flash
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::Streaming,
            TestExpectation::Supported,
        )
        .with_notes("Custom StreamingProcessor pattern"),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::Tools,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::Vision,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::Reasoning,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::PromptCache,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::ParallelTools,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-flash",
            Feature::CustomConfig,
            TestExpectation::Supported,
        ),
        // Gemini 2.5 Pro
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-pro",
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-pro",
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "gemini",
            "gemini-2.5-pro",
            Feature::Tools,
            TestExpectation::Supported,
        ),
    ]
}

fn openrouter_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "openrouter",
            models::OPENROUTER_X_AI_GROK_CODE_FAST_1,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "openrouter",
            models::OPENROUTER_X_AI_GROK_CODE_FAST_1,
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "openrouter",
            models::OPENROUTER_X_AI_GROK_CODE_FAST_1,
            Feature::Tools,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "openrouter",
            models::OPENROUTER_QWEN3_CODER,
            Feature::Basic,
            TestExpectation::Supported,
        ),
    ]
}

fn xai_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new("xai", models::xai::GROK_4, Feature::Basic, TestExpectation::Supported)
            .with_notes("XAI wraps OpenAI"),
        CompatibilityTest::new(
            "xai",
            models::xai::GROK_4,
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new("xai", models::xai::GROK_4, Feature::Tools, TestExpectation::Supported),
        CompatibilityTest::new(
            "xai",
            models::xai::GROK_4_CODE,
            Feature::Basic,
            TestExpectation::Supported,
        ),
    ]
}

fn moonshot_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "moonshot",
            models::MOONSHOT_KIMI_K2_TURBO_PREVIEW,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "moonshot",
            models::MOONSHOT_KIMI_K2_TURBO_PREVIEW,
            Feature::Streaming,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "moonshot",
            models::MOONSHOT_KIMI_K2_TURBO_PREVIEW,
            Feature::Tools,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "moonshot",
            models::MOONSHOT_KIMI_K2_THINKING,
            Feature::Reasoning,
            TestExpectation::PartiallySupported,
        )
        .with_notes("Heavy mode support"),
    ]
}

fn deepseek_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "deepseek",
            models::DEEPSEEK_CHAT,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "deepseek",
            models::DEEPSEEK_CHAT,
            Feature::Streaming,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "deepseek",
            models::DEEPSEEK_CHAT,
            Feature::Tools,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "deepseek",
            models::DEEPSEEK_REASONER,
            Feature::Reasoning,
            TestExpectation::Supported,
        )
        .with_notes("reasoning_effort parameter supported"),
    ]
}

fn zai_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "zai",
            models::zai::ZAI_CHAT_L,
            Feature::Basic,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "zai",
            models::zai::ZAI_CHAT_L,
            Feature::Streaming,
            TestExpectation::Unsupported,
        ),
        CompatibilityTest::new(
            "zai",
            models::zai::ZAI_CHAT_L,
            Feature::Tools,
            TestExpectation::Supported,
        ),
    ]
}

fn ollama_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "ollama",
            models::ollama::DEFAULT_MODEL,
            Feature::Basic,
            TestExpectation::Supported,
        )
        .with_notes("Local model inference"),
        CompatibilityTest::new(
            "ollama",
            models::ollama::DEFAULT_MODEL,
            Feature::Streaming,
            TestExpectation::Supported,
        ),
        CompatibilityTest::new(
            "ollama",
            models::ollama::DEFAULT_MODEL,
            Feature::Tools,
            TestExpectation::PartiallySupported,
        )
        .with_notes("Depends on model"),
    ]
}

fn lmstudio_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "lmstudio",
            models::lmstudio::DEFAULT_MODEL,
            Feature::Basic,
            TestExpectation::Supported,
        )
        .with_notes("Local model inference"),
        CompatibilityTest::new(
            "lmstudio",
            models::lmstudio::DEFAULT_MODEL,
            Feature::Streaming,
            TestExpectation::Supported,
        )
        .with_notes("Simplified streaming"),
        CompatibilityTest::new(
            "lmstudio",
            models::lmstudio::DEFAULT_MODEL,
            Feature::Tools,
            TestExpectation::PartiallySupported,
        )
        .with_notes("Depends on model"),
    ]
}

fn minimax_tests() -> Vec<CompatibilityTest> {
    vec![
        CompatibilityTest::new(
            "minimax",
            models::minimax::MINIMAX_M2,
            Feature::Basic,
            TestExpectation::Supported,
        )
        .with_notes("Wraps Anthropic provider"),
        CompatibilityTest::new(
            "minimax",
            models::minimax::MINIMAX_M2,
            Feature::Tools,
            TestExpectation::Supported,
        )
        .with_notes("XML-based tool calls"),
        CompatibilityTest::new(
            "minimax",
            models::minimax::MINIMAX_M2,
            Feature::Streaming,
            TestExpectation::Unsupported,
        ),
    ]
}

// ============================================================================
// Report Generation
// ============================================================================

/// Generate a markdown report from test results
pub fn generate_compatibility_report(results: &[TestResult]) -> String {
    let mut report = String::from("# Provider Compatibility Matrix\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Summary statistics
    let total = results.len();
    let passed = results.iter().filter(|r| r.matches_expectation()).count();
    let failed = total - passed;
    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Total Tests: {}\n", total));
    report.push_str(&format!("- Passed: {}\n", passed));
    report.push_str(&format!("- Failed: {}\n", failed));
    report.push_str(&format!("- Pass Rate: {:.2}%\n\n", pass_rate));

    // Group by provider
    let mut by_provider: HashMap<&str, Vec<&TestResult>> = HashMap::new();
    for result in results {
        by_provider
            .entry(result.test.provider)
            .or_default()
            .push(result);
    }

    // Generate per-provider tables
    let mut providers: Vec<_> = by_provider.keys().cloned().collect();
    providers.sort();

    for provider in providers {
        let provider_results = &by_provider[provider];
        report.push_str(&format!("## {}\n\n", provider.to_uppercase()));

        // Group by model
        let mut by_model: HashMap<&str, Vec<&TestResult>> = HashMap::new();
        for result in provider_results.iter() {
            by_model
                .entry(&result.test.model)
                .or_default()
                .push(result);
        }

        report.push_str("| Model | Basic | Stream | Tools | Vision | Reasoning | Cache | Parallel | Config |\n");
        report.push_str("|-------|-------|--------|-------|--------|-----------|-------|----------|--------|\n");

        let mut models: Vec<_> = by_model.keys().cloned().collect();
        models.sort();

        for model in models {
            let model_results = &by_model[model];
            report.push_str(&format!("| {} ", model));

            for feature in Feature::all() {
                let status = model_results
                    .iter()
                    .find(|r| r.test.feature == feature)
                    .map(|r| {
                        if r.matches_expectation() {
                            match r.test.expected {
                                TestExpectation::Supported => "✅",
                                TestExpectation::Unsupported => "➖",
                                TestExpectation::PartiallySupported => "⚠️",
                            }
                        } else {
                            "❌"
                        }
                    })
                    .unwrap_or("❓");
                report.push_str(&format!("| {} ", status));
            }
            report.push_str("|\n");
        }

        report.push_str("\n");
    }

    // Failed tests section
    let failed_tests: Vec<_> = results.iter().filter(|r| !r.matches_expectation()).collect();
    if !failed_tests.is_empty() {
        report.push_str("## Failed Tests\n\n");
        for result in failed_tests {
            report.push_str(&format!(
                "- **{}**: {} - {}\n",
                result.test.id(),
                result.test.feature.name(),
                result.error.as_deref().unwrap_or("Unknown error")
            ));
        }
        report.push_str("\n");
    }

    report.push_str("---\n");
    report.push_str("Legend: ✅ Supported | ➖ Not Supported (Expected) | ⚠️ Partially Supported | ❌ Failed | ❓ Not Tested\n");

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_compatibility_matrix() {
        let matrix = generate_compatibility_matrix();
        assert!(!matrix.is_empty());

        // Verify we have tests for all providers
        let providers: std::collections::HashSet<_> =
            matrix.iter().map(|t| t.provider).collect();
        assert!(providers.contains("anthropic"));
        assert!(providers.contains("openai"));
        assert!(providers.contains("gemini"));
        assert!(providers.contains("ollama"));
        assert!(providers.contains("lmstudio"));
        assert_eq!(providers.len(), 11);
    }

    #[test]
    fn test_feature_names() {
        assert_eq!(Feature::Basic.name(), "Basic");
        assert_eq!(Feature::Streaming.name(), "Streaming");
        assert_eq!(Feature::Tools.name(), "Tools");
    }

    #[test]
    fn test_test_result_matches_expectation() {
        let test = CompatibilityTest::new(
            "test",
            "model",
            Feature::Basic,
            TestExpectation::Supported,
        );

        let success = TestResult::success(test.clone(), 100);
        assert!(success.matches_expectation());

        let failure = TestResult::failure(test.clone(), "error".to_string(), 100);
        assert!(!failure.matches_expectation());
    }
}
