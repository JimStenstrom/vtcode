/**
 * MCP Tool Integration for VTCode Chat Extension
 *
 * Provides Model Context Protocol (MCP) tool support for enhanced
 * chat capabilities including external tool invocations.
 */

import { spawn } from "node:child_process";
import * as vscode from "vscode";
import * as TOML from "@iarna/toml";
import { ConfigLimits } from "./configLimits";

export interface McpTool {
    name: string;
    description: string;
    inputSchema: Record<string, unknown>;
    provider: string;
}

export interface McpProvider {
    name: string;
    command: string;
    args: string[];
    enabled: boolean;
    env?: Record<string, string>;
}

export interface McpToolInvocation {
    provider: string;
    tool: string;
    arguments: Record<string, unknown>;
}

export interface McpToolResult {
    success: boolean;
    result?: unknown;
    error?: string;
    executionTimeMs: number;
}

export class McpToolManager {
    private providers: Map<string, McpProvider> = new Map();
    private tools: Map<string, McpTool> = new Map();
    private outputChannel: vscode.OutputChannel;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
    }

    /**
     * Load MCP providers from vtcode.toml configuration
     */
    async loadProviders(workspaceRoot: string): Promise<void> {
        try {
            const configUri = vscode.Uri.file(`${workspaceRoot}/vtcode.toml`);
            const configContent = await vscode.workspace.fs.readFile(configUri);
            const config = this.parseToml(
                Buffer.from(configContent).toString("utf8")
            );

            if (config.mcp?.providers) {
                for (const provider of config.mcp.providers) {
                    if (provider.enabled !== false) {
                        this.providers.set(provider.name, provider);
                        this.outputChannel.appendLine(
                            `[MCP] Loaded provider: ${provider.name}`
                        );
                    }
                }
            }

            // Discover tools from each provider
            await this.discoverTools();
        } catch (error) {
            this.outputChannel.appendLine(
                `[MCP] Failed to load providers: ${
                    error instanceof Error ? error.message : String(error)
                }`
            );
        }
    }

    /**
     * Discover available tools from all enabled providers
     */
    private async discoverTools(): Promise<void> {
        for (const [providerName, provider] of this.providers) {
            try {
                const tools = await this.queryProviderTools(provider);
                for (const tool of tools) {
                    const fullName = `${providerName}/${tool.name}`;
                    this.tools.set(fullName, {
                        ...tool,
                        provider: providerName,
                    });
                    this.outputChannel.appendLine(
                        `[MCP] Discovered tool: ${fullName}`
                    );
                }
            } catch (error) {
                this.outputChannel.appendLine(
                    `[MCP] Failed to discover tools from ${providerName}: ${
                        error instanceof Error ? error.message : String(error)
                    }`
                );
            }
        }
    }

    /**
     * Query a provider for its available tools
     */
    private async queryProviderTools(
        provider: McpProvider
    ): Promise<McpTool[]> {
        return new Promise((resolve, reject) => {
            const proc = spawn(
                provider.command,
                [...provider.args, "--list-tools"],
                {
                    env: { ...process.env, ...provider.env },
                }
            );

            let stdout = "";
            let stderr = "";

            proc.stdout.on("data", (data: Buffer) => {
                stdout += data.toString();
            });

            proc.stderr.on("data", (data: Buffer) => {
                stderr += data.toString();
            });

            proc.on("close", (code: number | null) => {
                if (code !== 0) {
                    reject(
                        new Error(
                            `Provider exited with code ${code}: ${stderr}`
                        )
                    );
                    return;
                }

                try {
                    const tools = JSON.parse(stdout) as McpTool[];
                    resolve(tools);
                } catch (error) {
                    reject(new Error(`Failed to parse tools: ${error}`));
                }
            });

            proc.on("error", (error: Error) => {
                reject(error);
            });

            // Configurable timeout for discovery
            const timeout = ConfigLimits.mcpDiscoveryTimeoutMs;
            setTimeout(() => {
                proc.kill();
                reject(new Error(`Tool discovery timed out after ${timeout}ms`));
            }, timeout);
        });
    }

    /**
     * Invoke an MCP tool
     */
    async invokeTool(invocation: McpToolInvocation): Promise<McpToolResult> {
        const startTime = Date.now();

        const toolName = invocation.tool.includes("/")
            ? invocation.tool
            : `${invocation.provider}/${invocation.tool}`;

        const tool = this.tools.get(toolName);
        if (!tool) {
            return {
                success: false,
                error: `Tool not found: ${toolName}`,
                executionTimeMs: Date.now() - startTime,
            };
        }

        const provider = this.providers.get(tool.provider);
        if (!provider) {
            return {
                success: false,
                error: `Provider not found: ${tool.provider}`,
                executionTimeMs: Date.now() - startTime,
            };
        }

        try {
            const result = await this.executeToolViaProvider(
                provider,
                tool,
                invocation.arguments
            );

            return {
                success: true,
                result,
                executionTimeMs: Date.now() - startTime,
            };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error),
                executionTimeMs: Date.now() - startTime,
            };
        }
    }

    /**
     * Execute a tool via its provider
     */
    private async executeToolViaProvider(
        provider: McpProvider,
        tool: McpTool,
        args: Record<string, unknown>
    ): Promise<unknown> {
        return new Promise((resolve, reject) => {
            const proc = spawn(
                provider.command,
                [
                    ...provider.args,
                    "--tool",
                    tool.name,
                    "--args",
                    JSON.stringify(args),
                ],
                {
                    env: { ...process.env, ...provider.env },
                }
            );

            let stdout = "";
            let stderr = "";

            proc.stdout.on("data", (data: Buffer) => {
                stdout += data.toString();
            });

            proc.stderr.on("data", (data: Buffer) => {
                stderr += data.toString();
            });

            proc.on("close", (code: number | null) => {
                if (code !== 0) {
                    reject(new Error(`Tool execution failed: ${stderr}`));
                    return;
                }

                try {
                    const result = JSON.parse(stdout);
                    resolve(result);
                } catch {
                    // Return raw stdout if not JSON
                    resolve(stdout);
                }
            });

            proc.on("error", (error: Error) => {
                reject(error);
            });

            // Configurable timeout for execution
            const timeout = ConfigLimits.mcpExecutionTimeoutMs;
            setTimeout(() => {
                proc.kill();
                reject(new Error(`Tool execution timed out after ${timeout}ms`));
            }, timeout);
        });
    }

    /**
     * Get all available tools
     */
    getAvailableTools(): McpTool[] {
        return Array.from(this.tools.values());
    }

    /**
     * Get tools by provider
     */
    getToolsByProvider(providerName: string): McpTool[] {
        return Array.from(this.tools.values()).filter(
            (tool) => tool.provider === providerName
        );
    }

    /**
     * Check if a tool exists
     */
    hasTool(toolName: string): boolean {
        return this.tools.has(toolName);
    }

    /**
     * Parse TOML configuration using proper TOML library
     */
    private parseToml(content: string): Record<string, unknown> {
        try {
            return TOML.parse(content) as Record<string, unknown>;
        } catch (error) {
            this.outputChannel.appendLine(
                `[MCP] Failed to parse TOML: ${error instanceof Error ? error.message : String(error)}`
            );
            throw new Error(`Invalid TOML configuration: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
}

/**
 * Create MCP tool manager with workspace context
 */
export async function createMcpToolManager(
    outputChannel: vscode.OutputChannel
): Promise<McpToolManager | null> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
        outputChannel.appendLine("[MCP] No workspace folder found");
        return null;
    }

    const manager = new McpToolManager(outputChannel);
    await manager.loadProviders(workspaceFolders[0].uri.fsPath);

    return manager;
}
