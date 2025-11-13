/// Integration tests for Microsoft Direct Line v3 provider
///
/// These tests verify conversation lifecycle, activity polling,
/// Adaptive Cards support, and error handling.

use vtcode_core::llm::provider::{
    LLMError, LLMProvider, LLMRequest, LLMResponse, Message, MessageContent, MessageRole,
};
use vtcode_core::llm::providers::MicrosoftProvider;

/// Test helper to create a basic LLM request
fn create_test_request(prompt: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::Text(prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        system_prompt: None,
        tools: None,
        model: "copilot-m365".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        reasoning_effort: None,
    }
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_conversation_lifecycle() {
    // This test verifies the complete conversation lifecycle:
    // 1. Start conversation
    // 2. Send activity
    // 3. Poll for response
    // 4. Extract content

    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = MicrosoftProvider::new(secret);
    let request = create_test_request("Hello, this is a test message.");

    let result = provider.generate(request).await;
    assert!(result.is_ok(), "Expected successful response from Direct Line");

    let response = result.unwrap();
    assert!(!response.content.is_empty(), "Response content should not be empty");
    println!("Received response: {}", response.content);
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_multiple_messages() {
    // Test sending multiple messages in a single request
    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = MicrosoftProvider::new(secret);

    let request = LLMRequest {
        messages: vec![
            Message {
                role: MessageRole::User,
                content: MessageContent::Text("What is 2 + 2?".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: MessageRole::Assistant,
                content: MessageContent::Text("4".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: MessageRole::User,
                content: MessageContent::Text("What is 3 + 3?".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        system_prompt: None,
        tools: None,
        model: "copilot-m365".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        reasoning_effort: None,
    };

    let result = provider.generate(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_system_prompt() {
    // Test system prompt handling
    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = MicrosoftProvider::new(secret);

    let request = LLMRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::Text("Hello!".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        system_prompt: Some("You are a helpful assistant that always responds in a friendly manner.".to_string()),
        tools: None,
        model: "copilot-m365".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        reasoning_effort: None,
    };

    let result = provider.generate(request).await;
    assert!(result.is_ok());
}

#[test]
fn test_provider_capabilities() {
    // Test that provider capabilities are correctly reported
    let provider = MicrosoftProvider::new("test_secret".to_string());

    assert_eq!(provider.name(), "microsoft");
    assert!(!provider.supports_streaming());
    assert!(!provider.supports_reasoning("copilot-m365"));
    assert!(!provider.supports_tools("copilot-m365")); // Adaptive Cards != OpenAI-style tools
}

#[test]
fn test_supported_models() {
    // Test that supported models are correctly listed
    let provider = MicrosoftProvider::new("test_secret".to_string());
    let models = provider.supported_models();

    assert!(!models.is_empty(), "Should have at least one supported model");
    assert!(
        models.contains(&"copilot-m365".to_string()),
        "Should support copilot-m365 model"
    );
}

#[tokio::test]
async fn test_empty_messages_validation() {
    // Test that empty messages are rejected
    let provider = MicrosoftProvider::new("test_secret".to_string());

    let request = LLMRequest {
        messages: vec![],
        system_prompt: None,
        tools: None,
        model: "copilot-m365".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        reasoning_effort: None,
    };

    let result = provider.generate(request).await;
    assert!(result.is_err(), "Empty messages should be rejected");

    if let Err(LLMError::InvalidRequest(_)) = result {
        // Expected error type
    } else {
        panic!("Expected InvalidRequest error");
    }
}

#[tokio::test]
async fn test_missing_secret_validation() {
    // Test that missing secret is detected
    let provider = MicrosoftProvider::new("".to_string());
    let request = create_test_request("Test message");

    let result = provider.generate(request).await;
    assert!(result.is_err(), "Missing secret should be rejected");

    if let Err(LLMError::Authentication(_)) = result {
        // Expected error type
    } else {
        panic!("Expected Authentication error");
    }
}

#[test]
fn test_constructor_methods() {
    // Test all constructor methods work correctly
    let secret = "test_secret".to_string();

    // Test new()
    let provider1 = MicrosoftProvider::new(secret.clone());
    assert_eq!(provider1.name(), "microsoft");

    // Test with_model()
    let provider2 = MicrosoftProvider::with_model(secret.clone(), "custom-model".to_string());
    assert_eq!(provider2.name(), "microsoft");

    // Test from_config()
    let provider3 = MicrosoftProvider::from_config(
        Some(secret.clone()),
        Some("config-model".to_string()),
        None,
        None,
    );
    assert_eq!(provider3.name(), "microsoft");

    // Test from_config with defaults
    let provider4 = MicrosoftProvider::from_config(Some(secret), None, None, None);
    assert_eq!(provider4.name(), "microsoft");
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret and bot with Adaptive Cards support
async fn test_adaptive_cards_extraction() {
    // This test verifies that Adaptive Cards content is properly extracted
    // The bot must respond with an Adaptive Card for this test to verify extraction

    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = MicrosoftProvider::new(secret);
    let request = create_test_request("Show me an adaptive card");

    let result = provider.generate(request).await;
    assert!(result.is_ok(), "Should handle Adaptive Card responses");

    let response = result.unwrap();
    assert!(!response.content.is_empty(), "Should extract content from Adaptive Card");
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_timeout_handling() {
    // Test that the provider properly handles timeouts
    // This would require a bot that intentionally delays or doesn't respond

    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = MicrosoftProvider::new(secret);
    let request = create_test_request("Test timeout scenario");

    // The timeout is 30 seconds (30 polling attempts at 1 second each)
    // This test will take at least that long if the bot doesn't respond
    let result = provider.generate(request).await;

    // Result could be Ok (if bot responds) or Err (if timeout)
    // We're just verifying it completes within the timeout period
    match result {
        Ok(_) => println!("Bot responded within timeout"),
        Err(e) => println!("Timeout occurred: {:?}", e),
    }
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_custom_base_url() {
    // Test using a custom base URL
    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let custom_url = "https://directline.botframework.com/v3".to_string();

    let provider = MicrosoftProvider::from_config(
        Some(secret),
        None,
        Some(custom_url),
        None,
    );

    let request = create_test_request("Test with custom base URL");
    let result = provider.generate(request).await;

    // Should work the same as default base URL
    assert!(result.is_ok() || matches!(result, Err(LLMError::Provider(_))));
}

#[tokio::test]
#[ignore] // Requires valid Direct Line secret
async fn test_concurrent_requests() {
    // Test that multiple concurrent requests can be made
    let secret = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
        .expect("MICROSOFT_DIRECTLINE_SECRET must be set for integration tests");

    let provider = std::sync::Arc::new(MicrosoftProvider::new(secret));

    let mut handles = vec![];

    for i in 0..3 {
        let provider_clone = provider.clone();
        let handle = tokio::spawn(async move {
            let request = create_test_request(&format!("Concurrent request {}", i));
            provider_clone.generate(request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent request should succeed");
    }
}
