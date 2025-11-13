# Microsoft DirectLine Provider Guide

Microsoft DirectLine v3 enables VT Code to communicate with Azure Bot Service and Bot Framework bots through a REST-based protocol. This provider is ideal for integrating with existing Azure-hosted conversational AI solutions, including bots powered by Azure OpenAI Service.

## Overview

DirectLine is Microsoft's protocol for connecting external applications to Bot Framework bots. It provides:

- **Stateful Conversations**: Manages multi-turn conversations with persistent session state
- **Azure Integration**: Works seamlessly with Azure Bot Service deployments
- **OpenAI Compatibility**: Delegates to OpenAI-compatible backends for many scenarios
- **Enterprise Features**: Supports Azure authentication, logging, and compliance requirements

## Prerequisites

- Azure subscription with Bot Service enabled ([Azure Portal](https://portal.azure.com))
- A deployed bot (Bot Framework, Composer, or Azure OpenAI Service bot)
- DirectLine channel enabled on your bot
- DirectLine secret key from the Azure Portal

## Installation and Setup

### 1. Create or Configure Your Bot

If you don't have a bot yet:

1. **Azure Portal**: Navigate to "Azure Bot Service" > "Create"
2. **Bot Type**: Choose "Bot Channels Registration" or "Web App Bot"
3. **Configure**: Set up messaging endpoint, app ID, and authentication
4. **Deploy**: Publish your bot code to Azure App Service or use Bot Framework Composer

### 2. Enable DirectLine Channel

1. Open your bot resource in the Azure Portal
2. Navigate to "Channels" under "Settings"
3. Click "DirectLine" to enable the channel
4. Configure allowed sites if needed
5. Copy one of the DirectLine secrets (you'll need this for VT Code)

### 3. Obtain DirectLine Secret

In the DirectLine channel configuration:

```bash
# Secret keys are displayed under "Secret keys"
# Example: 1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p
```

⚠️ **Security Note**: Keep your DirectLine secret secure. Don't commit it to version control.

## Configuration

### Environment Variables

- `DIRECTLINE_API_KEY` (required): Your DirectLine secret from the Azure Portal
- `DIRECTLINE_BASE_URL` (optional): Custom DirectLine endpoint (defaults to `https://directline.botframework.com/v3/directline`)

### VT Code Configuration

Configure `vtcode.toml` in your workspace:

```toml
[agent]
provider = "microsoft"                    # Microsoft DirectLine provider
default_model = "directline-gpt-4"       # Model identifier for routing

[tools]
default_policy = "prompt"

[tools.policies]
read_file = "allow"
write_file = "prompt"
run_terminal_cmd = "prompt"
```

You can also override via CLI:

```bash
# Using environment variable for API key
export DIRECTLINE_API_KEY="your-directline-secret"
vtcode --provider microsoft --model directline-gpt-4

# Or specify directly (not recommended for production)
vtcode --provider microsoft --model directline-custom
```

## Supported Models

The DirectLine provider supports multiple model identifiers for routing to different bot deployments:

- **directline-gpt-4**: Default model for GPT-4 powered bots
- **directline-gpt-35-turbo**: For GPT-3.5 Turbo powered bots
- **directline-custom**: For custom bot implementations

These model identifiers are used internally by VT Code and don't directly correspond to Azure OpenAI deployment names. Your bot's backend handles the actual model routing.

## Using with Azure OpenAI Service

DirectLine works seamlessly with Azure OpenAI Service bots:

1. **Create Azure OpenAI Resource**: Deploy GPT-4, GPT-3.5, or other models
2. **Configure Bot**: Set your bot's backend to call Azure OpenAI APIs
3. **Connect via DirectLine**: Use VT Code with the DirectLine provider
4. **Conversation Flow**:
   - VT Code → DirectLine → Bot Framework → Your Bot → Azure OpenAI → Response

## Advanced Features

### Tool Calling and Function Execution

DirectLine bots can implement tool calling through Bot Framework's Activity schema:

```json
{
  "type": "message",
  "from": { "id": "bot" },
  "text": "Result from tool execution",
  "channelData": {
    "functionCall": {
      "name": "read_file",
      "arguments": "{\"path\": \"/home/user/file.txt\"}"
    }
  }
}
```

VT Code's DirectLine provider handles tool definitions and responses when your bot backend supports them.

### Streaming Support

DirectLine v3 supports real-time streaming via WebSocket connections. The current implementation uses polling, but WebSocket streaming can be enabled for lower latency conversations.

### Conversation State Management

DirectLine automatically manages conversation sessions:

- **Conversation ID**: Generated when starting a new conversation
- **Activity Watermark**: Tracks message position in the conversation
- **Session Expiration**: Conversations expire after inactivity (configurable in Azure)

## Local Development with Bot Framework Emulator

For local testing without Azure:

1. **Download Bot Framework Emulator**: [https://github.com/microsoft/BotFramework-Emulator](https://github.com/microsoft/BotFramework-Emulator)
2. **Run Your Bot Locally**: Start on `http://localhost:3978/api/messages`
3. **Configure Emulator**:
   - Bot URL: `http://localhost:3978/api/messages`
   - App ID and Password: Leave blank for local testing
4. **VT Code Configuration**:
   ```bash
   export DIRECTLINE_BASE_URL="http://localhost:3978/v3/directline"
   export DIRECTLINE_API_KEY="local-testing-key"
   ```

## Troubleshooting

### Common Issues

1. **401 Unauthorized**
   - Verify `DIRECTLINE_API_KEY` is set correctly
   - Check that DirectLine channel is enabled in Azure Portal
   - Ensure the secret hasn't expired (regenerate if needed)

2. **Connection Timeout**
   - Confirm bot messaging endpoint is accessible
   - Check Azure App Service is running
   - Verify `DIRECTLINE_BASE_URL` if using custom endpoint

3. **No Response from Bot**
   - Check bot logs in Azure Portal (Application Insights)
   - Verify bot is handling messages correctly
   - Test bot directly in Azure Portal's "Test in Web Chat"

4. **Tool Calling Not Working**
   - Ensure your bot backend implements tool calling
   - Check bot's adapter configuration for activity handling
   - Verify tool definitions are passed correctly

### Debugging Tips

Enable verbose logging in your bot to see incoming DirectLine activities:

```csharp
// Bot Framework SDK (C#)
public override async Task OnMessageActivityAsync(ITurnContext<IMessageActivity> turnContext, CancellationToken cancellationToken)
{
    var text = turnContext.Activity.Text;
    _logger.LogInformation($"Received: {text}");
    // Handle message...
}
```

Check DirectLine conversation status:

```bash
# Get conversation info (requires conversation ID)
curl -H "Authorization: Bearer YOUR_SECRET" \
  https://directline.botframework.com/v3/directline/conversations/CONVERSATION_ID
```

## Security Best Practices

1. **Use Enhanced Authentication**: Enable enhanced authentication token generation in Azure
2. **Rotate Secrets Regularly**: Regenerate DirectLine secrets periodically
3. **Limit Allowed Sites**: Configure allowed origins in DirectLine channel settings
4. **Use Managed Identity**: For Azure-hosted applications, use managed identities instead of secrets
5. **Monitor Usage**: Enable Azure Monitor and Application Insights for audit logs

## Resources

- [DirectLine API v3 Documentation](https://docs.microsoft.com/azure/bot-service/rest-api/bot-framework-rest-direct-line-3-0-concepts)
- [Azure Bot Service](https://azure.microsoft.com/services/bot-services/)
- [Bot Framework SDK](https://github.com/microsoft/botframework-sdk)
- [Azure OpenAI Service](https://azure.microsoft.com/products/ai-services/openai-service/)
- [Bot Framework Emulator](https://github.com/microsoft/BotFramework-Emulator)

## Example: Full Configuration

```toml
# vtcode.toml
[agent]
provider = "microsoft"
default_model = "directline-gpt-4"
temperature = 0.7
max_tokens = 4096

[agent.environment]
DIRECTLINE_API_KEY = "${DIRECTLINE_SECRET}"  # Use environment variable
DIRECTLINE_BASE_URL = "https://directline.botframework.com/v3/directline"

[tools]
default_policy = "prompt"
allowed_tools = ["read_file", "write_file", "run_terminal_cmd", "grep_file"]

[tools.policies]
read_file = "allow"
write_file = "prompt"
run_terminal_cmd = "prompt"
grep_file = "allow"
```

```bash
# .env (don't commit!)
DIRECTLINE_SECRET=your-actual-secret-key-here
```

For production deployments, use Azure Key Vault or similar secret management services instead of environment variables.
