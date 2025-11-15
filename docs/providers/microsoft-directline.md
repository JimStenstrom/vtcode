# Microsoft Direct Line v3 Provider

**Provider Name**: `microsoft`
**API**: [Microsoft Bot Framework Direct Line v3](https://docs.microsoft.com/en-us/azure/bot-service/rest-api/bot-framework-rest-direct-line-3-0)
**Purpose**: Connect to Microsoft Bot Framework bots, including M365 Copilot

---

## Overview

The Microsoft Direct Line v3 provider enables VTCode to communicate with bots built on the Microsoft Bot Framework, including Microsoft 365 Copilot. Unlike traditional LLM providers that use a simple request/response model, Direct Line uses a **conversation-based architecture** with **activity polling**.

### Key Characteristics

| Feature | Support | Notes |
|---------|---------|-------|
| **Streaming** | ❌ No | Uses polling instead |
| **Tools** | ❌ No | Uses Adaptive Cards instead |
| **Reasoning** | ❌ No | Depends on bot capabilities |
| **Adaptive Cards** | ✅ Yes | Full support for card extraction |
| **Authentication** | Bearer Token | Direct Line secret |

---

## Architecture

### Conversation-Based Model

Direct Line v3 uses a fundamentally different architecture from traditional LLM APIs:

```
Traditional LLM:          Direct Line v3:
┌─────────────┐          ┌─────────────────┐
│   Request   │          │ 1. Start Conv.  │
│      ↓      │          │ 2. Send Activity│
│   Response  │          │ 3. Poll Response│
└─────────────┘          └─────────────────┘
  (Synchronous)            (Asynchronous)
```

### Activity Lifecycle

1. **Start Conversation**: Create a conversation and get a conversation ID
2. **Send Activity**: Post a message activity to the conversation
3. **Poll Activities**: Repeatedly query for new activities using watermarks
4. **Extract Content**: Parse bot responses (text, Adaptive Cards, attachments)

### Watermark System

Direct Line uses watermarks to track which activities have been received:

```rust
// Initial poll (no watermark)
GET /conversations/{id}/activities
→ Returns activities 0-5, watermark="5"

// Subsequent poll (with watermark)
GET /conversations/{id}/activities?watermark=5
→ Returns activities 6-10, watermark="10"
```

This ensures clients only receive new activities and don't get duplicates.

---

## Configuration

### Environment Variables

```bash
# Required: Direct Line secret (get from Azure Portal)
export MICROSOFT_DIRECTLINE_SECRET="your-secret-here"

# Optional: Custom base URL (defaults to official endpoint)
export MICROSOFT_DIRECTLINE_BASE_URL="https://directline.botframework.com/v3"
```

### Code Configuration

```rust
use vtcode_core::llm::providers::MicrosoftProvider;

// Using environment variable
let provider = MicrosoftProvider::new(
    std::env::var("MICROSOFT_DIRECTLINE_SECRET").unwrap()
);

// With custom model
let provider = MicrosoftProvider::with_model(
    secret,
    "copilot-m365".to_string()
);

// Full configuration
let provider = MicrosoftProvider::from_config(
    Some(secret),
    Some("copilot-m365".to_string()),
    Some("https://directline.botframework.com/v3".to_string()),
    None
);
```

### VTCode Configuration File

```toml
# ~/.config/vtcode/config.toml
[llm]
provider = "microsoft"
model = "copilot-m365"

# Secret can be in env var or here
# api_key = "your-direct-line-secret"
```

---

## Adaptive Cards

### What are Adaptive Cards?

[Adaptive Cards](https://adaptivecards.io/) are platform-agnostic UI cards that can contain:
- Rich text and markdown
- Images and media
- Buttons and actions
- Input fields
- Container layouts

Example Adaptive Card structure:
```json
{
  "type": "AdaptiveCard",
  "version": "1.5",
  "body": [
    {
      "type": "TextBlock",
      "text": "Hello from M365 Copilot!",
      "size": "Large"
    },
    {
      "type": "TextBlock",
      "text": "This is additional information.",
      "wrap": true
    }
  ]
}
```

### Adaptive Card Extraction

The Microsoft provider automatically extracts content from Adaptive Cards:

**Priority Order**:
1. **Simple Text**: If `activity.text` is present, use it
2. **Adaptive Card**: If attachments contain `application/vnd.microsoft.card.adaptive`, extract text from card body
3. **Value Field**: If `activity.value` contains structured data, extract text

**Extraction Logic**:
```rust
// From an Adaptive Card with multiple TextBlocks:
{
  "body": [
    {"type": "TextBlock", "text": "Line 1"},
    {"type": "TextBlock", "text": "Line 2"}
  ]
}

// Extracted as:
"Line 1\nLine 2"
```

### Supported Content Types

The provider currently extracts text from these content types:

| Content Type | Description | Extraction |
|--------------|-------------|------------|
| `text/plain` | Simple text | Direct text field |
| `application/vnd.microsoft.card.adaptive` | Adaptive Card | TextBlock elements |
| Custom `value` | Structured data | `value.text` field |

### Limitations

**Not Currently Supported**:
- Interactive card actions (buttons, inputs)
- Image extraction from cards
- FactSet, ColumnSet rendering
- Media playback elements

**Reason**: VTCode is a CLI tool focused on text-based interactions. Visual elements would need terminal rendering support.

**Future**: Could be extended to:
- Render cards as formatted text in terminal
- Support button actions via menu selection
- Extract images and save to files

---

## Usage Examples

### Basic Usage

```rust
use vtcode_core::llm::provider::{LLMProvider, LLMRequest, Message, MessageContent, MessageRole};
use vtcode_core::llm::providers::MicrosoftProvider;

#[tokio::main]
async fn main() {
    let provider = MicrosoftProvider::new(
        std::env::var("MICROSOFT_DIRECTLINE_SECRET").unwrap()
    );

    let request = LLMRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::Text("Hello, Copilot!".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        system_prompt: Some("You are a helpful assistant.".to_string()),
        tools: None,
        model: "copilot-m365".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        reasoning_effort: None,
    };

    match provider.generate(request).await {
        Ok(response) => println!("Bot: {}", response.content),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

### Multi-Turn Conversation

```rust
// Note: Each generate() call creates a NEW conversation
// For multi-turn, you need to include conversation history

let messages = vec![
    Message {
        role: MessageRole::User,
        content: MessageContent::Text("What is 2+2?".to_string()),
        ..Default::default()
    },
    Message {
        role: MessageRole::Assistant,
        content: MessageContent::Text("4".to_string()),
        ..Default::default()
    },
    Message {
        role: MessageRole::User,
        content: MessageContent::Text("What is that times 3?".to_string()),
        ..Default::default()
    },
];

let request = LLMRequest {
    messages,
    model: "copilot-m365".to_string(),
    ..Default::default()
};

let response = provider.generate(request).await?;
```

### Error Handling

```rust
use vtcode_core::llm::provider::LLMError;

match provider.generate(request).await {
    Ok(response) => {
        println!("Success: {}", response.content);
    }
    Err(LLMError::Authentication(msg)) => {
        eprintln!("Auth error: {}", msg);
        eprintln!("Check MICROSOFT_DIRECTLINE_SECRET");
    }
    Err(LLMError::Network(msg)) => {
        eprintln!("Network error: {}", msg);
        eprintln!("Check internet connection");
    }
    Err(LLMError::Provider(msg)) => {
        eprintln!("Provider error: {}", msg);
        eprintln!("Check bot is running and Direct Line is configured");
    }
    Err(e) => {
        eprintln!("Other error: {:?}", e);
    }
}
```

---

## Performance Characteristics

### Latency

**Typical Response Times**:
- Conversation start: 200-500ms
- Send activity: 100-300ms
- Poll response: 1-30 seconds (depends on bot)
- **Total**: 1.3-31 seconds

**Factors Affecting Latency**:
1. **Bot Processing Time**: The bot must generate a response
2. **Polling Frequency**: Currently polls every 1 second
3. **Network Latency**: Round-trip time to Azure
4. **Bot Framework Overhead**: Activity routing and processing

### Timeout Configuration

```rust
// Default timeout: 30 polling attempts × 1 second = 30 seconds
const MAX_POLL_ATTEMPTS: usize = 30;
const POLL_INTERVAL: Duration = Duration::from_secs(1);

// HTTP client timeout: 120 seconds
let http_client = HttpClient::builder()
    .timeout(Duration::from_secs(120))
    .build()
    .unwrap();
```

**Customization**: Currently hardcoded. Future enhancement could allow configuration:
```rust
// Potential future API
let provider = MicrosoftProvider::from_config(...)
    .with_max_poll_attempts(60)
    .with_poll_interval(Duration::from_millis(500));
```

### Throughput

**Concurrent Conversations**:
- ✅ **Supported**: Each `generate()` call uses its own conversation
- ✅ **Thread-safe**: Can be used with `Arc<MicrosoftProvider>`
- ⚠️ **Rate Limits**: Subject to Direct Line API limits (see below)

**Rate Limits** (Direct Line API):
- Standard tier: 1,000 conversations/minute
- Premium tier: Custom limits

---

## Comparison with Other Providers

| Feature | OpenAI | Anthropic | Microsoft Direct Line |
|---------|--------|-----------|----------------------|
| **Architecture** | Request/Response | Request/Response | Conversation/Polling |
| **Latency** | 1-5s | 1-5s | 1-30s |
| **Streaming** | ✅ SSE | ✅ SSE | ❌ Polling only |
| **Tools** | ✅ Function calling | ✅ Tool use | ❌ Adaptive Cards |
| **Auth** | API Key | API Key | Direct Line Secret |
| **Session** | Stateless | Stateless | Conversation-based |
| **Rich Content** | Text only | Text only | ✅ Adaptive Cards |

---

## Troubleshooting

### Common Issues

#### 1. "Direct Line secret is required"

**Problem**: Missing or empty secret

**Solution**:
```bash
# Set environment variable
export MICROSOFT_DIRECTLINE_SECRET="your-secret-here"

# Or provide in code
let provider = MicrosoftProvider::new("your-secret-here".to_string());
```

#### 2. "Timeout waiting for bot response"

**Problem**: Bot didn't respond within 30 seconds

**Possible Causes**:
- Bot is not running
- Bot is processing a complex request
- Network connectivity issues
- Bot encountered an error

**Solution**:
1. Check bot status in Azure Portal
2. Test bot with Bot Framework Emulator
3. Check bot logs for errors
4. Verify Direct Line channel is enabled

#### 3. "Failed to start conversation"

**Problem**: Can't create conversation with Direct Line

**Possible Causes**:
- Invalid Direct Line secret
- Direct Line channel not enabled
- Network/firewall blocking Azure
- Incorrect base URL

**Solution**:
1. Verify secret in Azure Portal → Bot → Channels → Direct Line
2. Ensure Direct Line channel is enabled
3. Check firewall allows `directline.botframework.com`
4. Try default base URL (don't set `MICROSOFT_DIRECTLINE_BASE_URL`)

#### 4. Empty or No Response

**Problem**: Bot responds but no content extracted

**Possible Causes**:
- Bot sends activities with no text or attachments
- Adaptive Card format not recognized
- Bot sends non-message activities (typing, etc.)

**Debug Steps**:
1. Enable debug logging: `RUST_LOG=vtcode::llm::microsoft=debug`
2. Check for `has_attachments` and `has_adaptive_card` in logs
3. Inspect raw activity JSON from bot
4. Verify Adaptive Card schema version (1.0-1.5 supported)

---

## Testing

### Unit Tests

Run provider unit tests:
```bash
cargo test --package vtcode-core --lib providers::microsoft
```

### Integration Tests

**Requirements**:
- Valid Direct Line secret
- Running bot (or use Bot Framework Emulator)

Run integration tests:
```bash
export MICROSOFT_DIRECTLINE_SECRET="your-secret"
cargo test --package vtcode-core --test integration -- microsoft --ignored
```

### Manual Testing

Test with Bot Framework Emulator:
1. Download [Bot Framework Emulator](https://github.com/Microsoft/BotFramework-Emulator)
2. Configure emulator with Direct Line secret
3. Run VTCode with same secret
4. Verify responses match

---

## Implementation Details

### Code Structure

```
vtcode-core/src/llm/providers/microsoft.rs
├── MicrosoftProvider struct
│   ├── secret: String
│   ├── base_url: String
│   ├── model: String
│   └── http_client: HttpClient
├── Activity structs (serialization)
│   ├── Activity (outgoing)
│   ├── BotActivity (incoming)
│   ├── Attachment (Adaptive Cards)
│   └── ActivityParticipant
└── Methods
    ├── start_conversation() → ConversationResponse
    ├── send_activity() → Result<(), LLMError>
    ├── get_activities() → ActivitiesResponse
    ├── extract_activity_content() → Option<String>
    └── convert_messages_to_text() → String
```

### Duplication Patterns

The Microsoft provider follows the same patterns as other providers:

1. **Constructors**: ✅ Uses `impl_provider_constructors!` macro
2. **HTTP Client**: Standard `HttpClient::builder()` pattern
3. **Error Handling**: Uses `error_display::format_llm_error()`
4. **Message Conversion**: Custom for Direct Line API format
5. **LLMProvider Trait**: Standard implementation

**Phase 3 Impact**: Will benefit from shared abstractions:
- `ErrorMapper` trait (error handling)
- `MessageConverter` trait (message format conversion)
- Shared HTTP client builder

---

## Future Enhancements

### Potential Improvements

1. **Configurable Polling**
   - Adjust poll interval and max attempts
   - Exponential backoff for long-running requests

2. **Rich Card Rendering**
   - Format Adaptive Cards as terminal-friendly text
   - Support button selection via menus
   - Render FactSets, ColumnSets as tables

3. **Conversation Persistence**
   - Reuse conversations across multiple messages
   - Support conversation tokens for extended sessions

4. **Activity Types**
   - Handle typing indicators
   - Process event activities
   - Support invoke activities

5. **Streaming via WebSocket**
   - Direct Line supports WebSocket for streaming
   - Would eliminate polling overhead
   - Real-time updates

6. **Enhanced Telemetry**
   - Track conversation lifetimes
   - Monitor polling efficiency
   - Measure bot response times

---

## References

- [Direct Line API v3.0 Documentation](https://docs.microsoft.com/en-us/azure/bot-service/rest-api/bot-framework-rest-direct-line-3-0)
- [Adaptive Cards Documentation](https://adaptivecards.io/)
- [Bot Framework Documentation](https://docs.microsoft.com/en-us/azure/bot-service/)
- [M365 Copilot Integration](https://learn.microsoft.com/en-us/microsoft-365-copilot/extensibility/)

---

## Support

For issues specific to the Microsoft Direct Line provider:
1. Check Azure Bot Service status
2. Verify Direct Line channel configuration
3. Review bot logs in Azure Portal
4. Test bot with Bot Framework Emulator
5. Check VTCode debug logs: `RUST_LOG=vtcode::llm::microsoft=debug`

For VTCode issues, see main documentation or file an issue on GitHub.
