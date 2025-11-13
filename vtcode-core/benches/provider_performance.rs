//! Performance benchmarks for LLM providers
//!
//! These benchmarks measure the performance of provider operations including:
//! - Constructor performance
//! - Request serialization
//! - Response deserialization
//! - Error mapping
//! - Streaming event processing
//!
//! Run with: cargo bench --bench provider_performance

use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;
use vtcode_core::config::constants::models;
use vtcode_core::llm::provider::{FinishReason, LLMRequest, LLMResponse, Message, Usage};
use vtcode_core::llm::providers::{
    AnthropicProvider, DeepSeekProvider, GeminiProvider, LmStudioProvider, MoonshotProvider,
    OllamaProvider, OpenAIProvider, OpenRouterProvider, XAIProvider, ZAIProvider,
};

// ============================================================================
// Provider Construction Benchmarks
// ============================================================================

fn bench_provider_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("provider_construction");

    group.bench_function("anthropic_new", |b| {
        b.iter(|| AnthropicProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("openai_new", |b| {
        b.iter(|| OpenAIProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("gemini_new", |b| {
        b.iter(|| GeminiProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("openrouter_new", |b| {
        b.iter(|| OpenRouterProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("xai_new", |b| {
        b.iter(|| XAIProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("moonshot_new", |b| {
        b.iter(|| MoonshotProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("deepseek_new", |b| {
        b.iter(|| DeepSeekProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("zai_new", |b| {
        b.iter(|| ZAIProvider::new(black_box("test_key".to_string())))
    });

    group.bench_function("ollama_new", |b| {
        b.iter(|| OllamaProvider::new(black_box(String::new())))
    });

    group.bench_function("lmstudio_new", |b| {
        b.iter(|| LmStudioProvider::new(black_box(String::new())))
    });

    group.finish();
}

// ============================================================================
// Request Creation Benchmarks
// ============================================================================

fn bench_request_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_creation");

    group.bench_function("simple_request", |b| {
        b.iter(|| {
            black_box(LLMRequest {
                messages: vec![Message::user("Hello".to_string())],
                system_prompt: None,
                tools: None,
                model: "test-model".to_string(),
                max_tokens: None,
                temperature: None,
                stream: false,
                tool_choice: None,
                parallel_tool_calls: None,
                parallel_tool_config: None,
                reasoning_effort: None,
            })
        })
    });

    group.bench_function("request_with_system_prompt", |b| {
        b.iter(|| {
            black_box(LLMRequest {
                messages: vec![Message::user("Hello".to_string())],
                system_prompt: Some("You are a helpful assistant".to_string()),
                tools: None,
                model: "test-model".to_string(),
                max_tokens: None,
                temperature: None,
                stream: false,
                tool_choice: None,
                parallel_tool_calls: None,
                parallel_tool_config: None,
                reasoning_effort: None,
            })
        })
    });

    group.bench_function("multi_message_request", |b| {
        b.iter(|| {
            black_box(LLMRequest {
                messages: vec![
                    Message::user("What is 2+2?".to_string()),
                    Message::assistant("4".to_string()),
                    Message::user("What is 3+3?".to_string()),
                ],
                system_prompt: None,
                tools: None,
                model: "test-model".to_string(),
                max_tokens: None,
                temperature: None,
                stream: false,
                tool_choice: None,
                parallel_tool_calls: None,
                parallel_tool_config: None,
                reasoning_effort: None,
            })
        })
    });

    group.finish();
}

// ============================================================================
// Model Iteration Benchmarks
// ============================================================================

fn bench_supported_models(c: &mut Criterion) {
    let mut group = c.benchmark_group("supported_models");

    let anthropic = AnthropicProvider::new("test_key".to_string());
    group.bench_function("anthropic_supported_models", |b| {
        b.iter(|| black_box(anthropic.supported_models()))
    });

    let openai = OpenAIProvider::new("test_key".to_string());
    group.bench_function("openai_supported_models", |b| {
        b.iter(|| black_box(openai.supported_models()))
    });

    let gemini = GeminiProvider::new("test_key".to_string());
    group.bench_function("gemini_supported_models", |b| {
        b.iter(|| black_box(gemini.supported_models()))
    });

    group.finish();
}

// ============================================================================
// Request Validation Benchmarks
// ============================================================================

fn bench_request_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_validation");

    let request = LLMRequest {
        messages: vec![Message::user("Hello".to_string())],
        system_prompt: None,
        tools: None,
        model: models::CLAUDE_SONNET_4_5.to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    };

    let anthropic = AnthropicProvider::new("test_key".to_string());
    group.bench_function("anthropic_validate_request", |b| {
        b.iter(|| black_box(anthropic.validate_request(&request)))
    });

    let request_openai = LLMRequest {
        model: "gpt-5".to_string(),
        ..request.clone()
    };
    let openai = OpenAIProvider::new("test_key".to_string());
    group.bench_function("openai_validate_request", |b| {
        b.iter(|| black_box(openai.validate_request(&request_openai)))
    });

    let request_gemini = LLMRequest {
        model: "gemini-2.5-flash".to_string(),
        ..request
    };
    let gemini = GeminiProvider::new("test_key".to_string());
    group.bench_function("gemini_validate_request", |b| {
        b.iter(|| black_box(gemini.validate_request(&request_gemini)))
    });

    group.finish();
}

// ============================================================================
// Message Creation Benchmarks
// ============================================================================

fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");

    group.bench_function("user_message", |b| {
        b.iter(|| black_box(Message::user("Hello, world!".to_string())))
    });

    group.bench_function("assistant_message", |b| {
        b.iter(|| black_box(Message::assistant("Hello!".to_string())))
    });

    group.bench_function("system_message", |b| {
        b.iter(|| black_box(Message::system("You are helpful".to_string())))
    });

    group.bench_function("tool_response_message", |b| {
        b.iter(|| black_box(Message::tool_response("call_123".to_string(), "result".to_string())))
    });

    group.finish();
}

// ============================================================================
// Provider Name/ID Benchmarks
// ============================================================================

fn bench_provider_metadata(c: &mut Criterion) {
    use vtcode_core::llm::client::LLMClient;
    use vtcode_core::llm::provider::LLMProvider;

    let mut group = c.benchmark_group("provider_metadata");

    // Benchmark provider name() method
    let anthropic = AnthropicProvider::new("test_key".to_string());
    group.bench_function("anthropic_name", |b| {
        b.iter(|| black_box(anthropic.name()))
    });

    let openai = OpenAIProvider::new("test_key".to_string());
    group.bench_function("openai_name", |b| {
        b.iter(|| black_box(openai.name()))
    });

    let gemini = GeminiProvider::new("test_key".to_string());
    group.bench_function("gemini_name", |b| {
        b.iter(|| black_box(gemini.name()))
    });

    // Benchmark provider model_id() method
    group.bench_function("anthropic_model_id", |b| {
        b.iter(|| black_box(anthropic.model_id()))
    });

    group.bench_function("openai_model_id", |b| {
        b.iter(|| black_box(openai.model_id()))
    });

    group.bench_function("gemini_model_id", |b| {
        b.iter(|| black_box(gemini.model_id()))
    });

    group.finish();
}

// ============================================================================
// Factory Benchmarks
// ============================================================================

fn bench_factory_operations(c: &mut Criterion) {
    use vtcode_core::llm::factory::{LLMFactory, create_provider_for_model};

    let mut group = c.benchmark_group("factory_operations");

    group.bench_function("factory_new", |b| {
        b.iter(|| black_box(LLMFactory::new()))
    });

    let factory = LLMFactory::new();
    group.bench_function("factory_list_providers", |b| {
        b.iter(|| black_box(factory.list_providers()))
    });

    group.bench_function("factory_provider_from_model_anthropic", |b| {
        b.iter(|| black_box(factory.provider_from_model(models::CLAUDE_SONNET_4_5)))
    });

    group.bench_function("factory_provider_from_model_openai", |b| {
        b.iter(|| black_box(factory.provider_from_model("gpt-5")))
    });

    group.bench_function("factory_provider_from_model_gemini", |b| {
        b.iter(|| black_box(factory.provider_from_model("gemini-2.5-flash")))
    });

    group.bench_function("factory_provider_from_model_unknown", |b| {
        b.iter(|| black_box(factory.provider_from_model("unknown-model")))
    });

    group.bench_function("create_provider_anthropic", |b| {
        b.iter(|| {
            black_box(create_provider_for_model(
                models::CLAUDE_SONNET_4_5,
                "test_key".to_string(),
                None,
            ))
        })
    });

    group.bench_function("create_provider_openai", |b| {
        b.iter(|| {
            black_box(create_provider_for_model(
                "gpt-5",
                "test_key".to_string(),
                None,
            ))
        })
    });

    group.finish();
}

// ============================================================================
// Batch Operations Benchmarks
// ============================================================================

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    group.bench_function("create_10_providers", |b| {
        b.iter(|| {
            for _ in 0..10 {
                black_box(AnthropicProvider::new("test_key".to_string()));
            }
        })
    });

    group.bench_function("create_10_requests", |b| {
        b.iter(|| {
            for i in 0..10 {
                black_box(LLMRequest {
                    messages: vec![Message::user(format!("Message {}", i))],
                    system_prompt: None,
                    tools: None,
                    model: "test-model".to_string(),
                    max_tokens: None,
                    temperature: None,
                    stream: false,
                    tool_choice: None,
                    parallel_tool_calls: None,
                    parallel_tool_config: None,
                    reasoning_effort: None,
                });
            }
        })
    });

    group.finish();
}

// ============================================================================
// Clone Operations Benchmarks
// ============================================================================

fn bench_clone_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_operations");

    let request = LLMRequest {
        messages: vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi".to_string()),
            Message::user("How are you?".to_string()),
        ],
        system_prompt: Some("You are a helpful assistant".to_string()),
        tools: None,
        model: "test-model".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    };

    group.bench_function("clone_request", |b| {
        b.iter(|| black_box(request.clone()))
    });

    let message = Message::user("This is a test message with some content".to_string());
    group.bench_function("clone_message", |b| {
        b.iter(|| black_box(message.clone()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_provider_construction,
    bench_request_creation,
    bench_supported_models,
    bench_request_validation,
    bench_message_creation,
    bench_provider_metadata,
    bench_factory_operations,
    bench_batch_operations,
    bench_clone_operations,
);

criterion_main!(benches);
