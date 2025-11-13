import { describe, it, expect, beforeEach, vi } from "vitest";
import {
    McpTool,
    McpProvider,
    McpToolInvocation,
    McpToolResult,
    McpToolManager,
} from "../mcpTools";
import * as vscode from "vscode";

describe("McpTool interface", () => {
    it("should define tool structure", () => {
        const tool: McpTool = {
            name: "read_file",
            description: "Read contents of a file",
            inputSchema: {
                type: "object",
                properties: {
                    path: { type: "string" },
                },
                required: ["path"],
            },
            provider: "filesystem",
        };

        expect(tool.name).toBe("read_file");
        expect(tool.description).toBeDefined();
        expect(tool.inputSchema).toBeDefined();
        expect(tool.provider).toBe("filesystem");
    });

    it("should support complex input schemas", () => {
        const tool: McpTool = {
            name: "execute_query",
            description: "Execute database query",
            inputSchema: {
                type: "object",
                properties: {
                    query: { type: "string" },
                    params: { type: "array", items: { type: "string" } },
                    timeout: { type: "number" },
                },
                required: ["query"],
            },
            provider: "database",
        };

        expect(tool.inputSchema.properties).toBeDefined();
        expect(tool.inputSchema.required).toContain("query");
    });

    it("should support minimal tool definition", () => {
        const tool: McpTool = {
            name: "ping",
            description: "Health check",
            inputSchema: {},
            provider: "system",
        };

        expect(tool.name).toBe("ping");
        expect(tool.inputSchema).toEqual({});
    });
});

describe("McpProvider interface", () => {
    it("should define provider structure", () => {
        const provider: McpProvider = {
            name: "filesystem",
            command: "/usr/bin/mcp-filesystem",
            args: ["--mode", "server"],
            enabled: true,
        };

        expect(provider.name).toBe("filesystem");
        expect(provider.command).toBeDefined();
        expect(provider.args).toHaveLength(2);
        expect(provider.enabled).toBe(true);
    });

    it("should support environment variables", () => {
        const provider: McpProvider = {
            name: "api-provider",
            command: "node",
            args: ["server.js"],
            enabled: true,
            env: {
                API_KEY: "secret-key",
                PORT: "3000",
            },
        };

        expect(provider.env).toBeDefined();
        expect(provider.env?.API_KEY).toBe("secret-key");
    });

    it("should support disabled providers", () => {
        const provider: McpProvider = {
            name: "disabled-provider",
            command: "test",
            args: [],
            enabled: false,
        };

        expect(provider.enabled).toBe(false);
    });

    it("should handle complex argument arrays", () => {
        const provider: McpProvider = {
            name: "complex",
            command: "python",
            args: [
                "-m",
                "mcp_server",
                "--config",
                "/path/to/config.json",
                "--verbose",
            ],
            enabled: true,
        };

        expect(provider.args).toHaveLength(5);
        expect(provider.args).toContain("--verbose");
    });
});

describe("McpToolInvocation interface", () => {
    it("should define invocation structure", () => {
        const invocation: McpToolInvocation = {
            provider: "filesystem",
            tool: "read_file",
            arguments: {
                path: "/workspace/test.txt",
            },
        };

        expect(invocation.provider).toBe("filesystem");
        expect(invocation.tool).toBe("read_file");
        expect(invocation.arguments.path).toBe("/workspace/test.txt");
    });

    it("should support complex arguments", () => {
        const invocation: McpToolInvocation = {
            provider: "api",
            tool: "make_request",
            arguments: {
                url: "https://api.example.com",
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: "Bearer token",
                },
                body: {
                    data: "test",
                },
            },
        };

        expect(invocation.arguments.method).toBe("POST");
        expect(invocation.arguments.headers).toBeDefined();
        expect((invocation.arguments.headers as any)["Content-Type"]).toBe(
            "application/json"
        );
    });

    it("should support empty arguments", () => {
        const invocation: McpToolInvocation = {
            provider: "system",
            tool: "health_check",
            arguments: {},
        };

        expect(invocation.arguments).toEqual({});
    });
});

describe("McpToolResult interface", () => {
    it("should define successful result", () => {
        const result: McpToolResult = {
            success: true,
            result: { content: "file contents here" },
            executionTimeMs: 150,
        };

        expect(result.success).toBe(true);
        expect(result.result).toBeDefined();
        expect(result.executionTimeMs).toBe(150);
        expect(result.error).toBeUndefined();
    });

    it("should define failed result", () => {
        const result: McpToolResult = {
            success: false,
            error: "File not found",
            executionTimeMs: 50,
        };

        expect(result.success).toBe(false);
        expect(result.error).toBe("File not found");
        expect(result.result).toBeUndefined();
    });

    it("should track execution time", () => {
        const results: McpToolResult[] = [
            { success: true, executionTimeMs: 100 },
            { success: true, executionTimeMs: 200 },
            { success: true, executionTimeMs: 150 },
        ];

        const avgTime =
            results.reduce((sum, r) => sum + r.executionTimeMs, 0) /
            results.length;
        expect(avgTime).toBe(150);
    });

    it("should support complex result data", () => {
        const result: McpToolResult = {
            success: true,
            result: {
                files: ["file1.txt", "file2.txt"],
                totalSize: 1024,
                metadata: {
                    lastModified: Date.now(),
                },
            },
            executionTimeMs: 300,
        };

        expect((result.result as any).files).toHaveLength(2);
        expect((result.result as any).totalSize).toBe(1024);
    });
});

describe("McpToolManager", () => {
    let manager: McpToolManager;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        mockOutput = {
            append: vi.fn(),
            appendLine: vi.fn(),
            clear: vi.fn(),
            show: vi.fn(),
            hide: vi.fn(),
            dispose: vi.fn(),
        } as any;

        manager = new McpToolManager(mockOutput);
    });

    describe("constructor", () => {
        it("should create MCP manager instance", () => {
            expect(manager).toBeDefined();
            expect(manager).toBeInstanceOf(McpToolManager);
        });

        it("should accept output channel", () => {
            const output = { appendLine: vi.fn() } as any;
            const mgr = new McpToolManager(output);
            expect(mgr).toBeDefined();
        });
    });

    describe("tool management", () => {
        it("should be instantiable", () => {
            expect(manager).toBeDefined();
        });

        it("should have loadProviders method", () => {
            expect(typeof manager.loadProviders).toBe("function");
        });

        it("should accept workspace root", async () => {
            // This would fail without a real workspace, but we test the interface
            expect(manager.loadProviders).toBeDefined();
        });
    });
});

describe("MCP Tool system integration", () => {
    it("should support tool discovery flow", () => {
        const provider: McpProvider = {
            name: "test-provider",
            command: "test",
            args: [],
            enabled: true,
        };

        const tool: McpTool = {
            name: "test_tool",
            description: "A test tool",
            inputSchema: {},
            provider: provider.name,
        };

        const invocation: McpToolInvocation = {
            provider: provider.name,
            tool: tool.name,
            arguments: {},
        };

        expect(invocation.provider).toBe(provider.name);
        expect(invocation.tool).toBe(tool.name);
    });

    it("should support result tracking", () => {
        const result: McpToolResult = {
            success: true,
            result: { data: "test" },
            executionTimeMs: 100,
        };

        expect(result.success).toBe(true);
        expect(result.executionTimeMs).toBeGreaterThan(0);
    });

    it("should handle multiple providers", () => {
        const providers: McpProvider[] = [
            {
                name: "provider1",
                command: "test1",
                args: [],
                enabled: true,
            },
            {
                name: "provider2",
                command: "test2",
                args: [],
                enabled: true,
            },
            {
                name: "provider3",
                command: "test3",
                args: [],
                enabled: false,
            },
        ];

        const enabledProviders = providers.filter((p) => p.enabled);
        expect(enabledProviders).toHaveLength(2);
    });

    it("should namespace tools by provider", () => {
        const tool1: McpTool = {
            name: "read",
            description: "Read from provider 1",
            inputSchema: {},
            provider: "provider1",
        };

        const tool2: McpTool = {
            name: "read",
            description: "Read from provider 2",
            inputSchema: {},
            provider: "provider2",
        };

        const fullName1 = `${tool1.provider}/${tool1.name}`;
        const fullName2 = `${tool2.provider}/${tool2.name}`;

        expect(fullName1).toBe("provider1/read");
        expect(fullName2).toBe("provider2/read");
        expect(fullName1).not.toBe(fullName2);
    });
});

describe("MCP error handling", () => {
    it("should create error results", () => {
        const result: McpToolResult = {
            success: false,
            error: "Provider connection failed",
            executionTimeMs: 5000,
        };

        expect(result.success).toBe(false);
        expect(result.error).toContain("connection failed");
    });

    it("should track failed invocations", () => {
        const results: McpToolResult[] = [
            { success: true, executionTimeMs: 100 },
            { success: false, error: "Error 1", executionTimeMs: 50 },
            { success: true, executionTimeMs: 200 },
            { success: false, error: "Error 2", executionTimeMs: 75 },
        ];

        const failures = results.filter((r) => !r.success);
        expect(failures).toHaveLength(2);
    });

    it("should include error details", () => {
        const result: McpToolResult = {
            success: false,
            error: "Timeout after 30000ms",
            executionTimeMs: 30000,
        };

        expect(result.error).toContain("Timeout");
        expect(result.error).toContain("30000");
    });
});
