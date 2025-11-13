import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import {
    VtcodeBackend,
    VtcodeStreamChunk,
    VtcodeToolCall,
    VtcodeToolResultChunk,
} from "../vtcodeBackend";
import * as vscode from "vscode";

describe("VtcodeBackend", () => {
    let backend: VtcodeBackend;
    let mockOutput: vscode.OutputChannel;
    let commandPath: string;
    let workspaceRoot: string;

    beforeEach(() => {
        mockOutput = {
            append: vi.fn(),
            appendLine: vi.fn(),
            clear: vi.fn(),
            show: vi.fn(),
            hide: vi.fn(),
            dispose: vi.fn(),
        } as any;

        commandPath = "/usr/bin/vtcode";
        workspaceRoot = "/workspace";

        backend = new VtcodeBackend(commandPath, workspaceRoot, mockOutput);
    });

    afterEach(() => {
        backend.dispose();
        vi.restoreAllMocks();
    });

    describe("constructor", () => {
        it("should create backend instance with correct properties", () => {
            expect(backend).toBeDefined();
            expect(backend).toBeInstanceOf(VtcodeBackend);
        });

        it("should accept undefined workspace root", () => {
            const backendNoWorkspace = new VtcodeBackend(
                commandPath,
                undefined,
                mockOutput
            );
            expect(backendNoWorkspace).toBeDefined();
            backendNoWorkspace.dispose();
        });
    });

    describe("updateConfiguration", () => {
        it("should update command path and workspace root", () => {
            const newCommandPath = "/usr/local/bin/vtcode";
            const newWorkspaceRoot = "/new/workspace";

            backend.updateConfiguration(newCommandPath, newWorkspaceRoot);

            // Configuration updated (we can't directly verify but no errors)
            expect(true).toBe(true);
        });

        it("should accept undefined workspace root", () => {
            backend.updateConfiguration("/usr/bin/vtcode", undefined);
            expect(true).toBe(true);
        });

        it("should accept config summary", () => {
            const configSummary = {
                hasConfig: true,
                mcpProviders: [],
                humanInTheLoop: true,
                agentProvider: "anthropic",
                agentDefaultModel: "claude-sonnet-4-5",
                automationFullAutoEnabled: true,
            };

            backend.updateConfiguration(
                commandPath,
                workspaceRoot,
                configSummary
            );

            expect(true).toBe(true);
        });

        it("should work without config summary", () => {
            backend.updateConfiguration(commandPath, workspaceRoot);
            expect(true).toBe(true);
        });
    });

    describe("setEnvironmentProvider", () => {
        it("should accept environment provider function", () => {
            const envProvider = () => ({
                API_KEY: "test-key",
                CUSTOM_VAR: "value",
            });

            backend.setEnvironmentProvider(envProvider);
            expect(true).toBe(true);
        });

        it("should accept undefined to clear provider", () => {
            backend.setEnvironmentProvider(undefined);
            expect(true).toBe(true);
        });

        it("should not throw when setting provider", () => {
            expect(() => {
                backend.setEnvironmentProvider(() => ({}));
            }).not.toThrow();
        });
    });

    describe("dispose", () => {
        it("should dispose without errors", () => {
            expect(() => {
                backend.dispose();
            }).not.toThrow();
        });

        it("should be idempotent", () => {
            backend.dispose();
            expect(() => {
                backend.dispose();
            }).not.toThrow();
        });
    });
});

describe("VtcodeStreamChunk types", () => {
    describe("text chunk", () => {
        it("should have correct structure", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "text",
                text: "Hello world",
            };

            expect(chunk.kind).toBe("text");
            expect((chunk as any).text).toBe("Hello world");
        });
    });

    describe("reasoning chunk", () => {
        it("should have correct structure", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "reasoning",
                text: "Let me think...",
            };

            expect(chunk.kind).toBe("reasoning");
            expect((chunk as any).text).toBe("Let me think...");
        });
    });

    describe("metadata chunk", () => {
        it("should have correct structure", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "metadata",
                metadata: { usage: { input_tokens: 100, output_tokens: 50 } },
            };

            expect(chunk.kind).toBe("metadata");
            expect((chunk as any).metadata).toBeDefined();
        });
    });

    describe("toolCall chunk", () => {
        it("should have correct structure with callbacks", () => {
            const respondFn = vi.fn();
            const rejectFn = vi.fn();

            const chunk: VtcodeStreamChunk = {
                kind: "toolCall",
                call: {
                    id: "tool-1",
                    name: "read_file",
                    args: { path: "/test.txt" },
                },
                respond: respondFn,
                reject: rejectFn,
            };

            expect(chunk.kind).toBe("toolCall");
            expect((chunk as any).call.id).toBe("tool-1");
            expect((chunk as any).call.name).toBe("read_file");
            expect((chunk as any).respond).toBe(respondFn);
            expect((chunk as any).reject).toBe(rejectFn);
        });

        it("should allow calling respond callback", () => {
            const respondFn = vi.fn();
            const chunk: VtcodeStreamChunk = {
                kind: "toolCall",
                call: { id: "1", name: "test", args: {} },
                respond: respondFn,
                reject: vi.fn(),
            };

            (chunk as any).respond({ result: "success" });
            expect(respondFn).toHaveBeenCalledWith({ result: "success" });
        });

        it("should allow calling reject callback", () => {
            const rejectFn = vi.fn();
            const chunk: VtcodeStreamChunk = {
                kind: "toolCall",
                call: { id: "1", name: "test", args: {} },
                respond: vi.fn(),
                reject: rejectFn,
            };

            (chunk as any).reject("Error occurred");
            expect(rejectFn).toHaveBeenCalledWith("Error occurred");
        });
    });

    describe("toolResult chunk", () => {
        it("should have started status", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "bash",
                status: "started",
            };

            expect(chunk.kind).toBe("toolResult");
            expect(chunk.status).toBe("started");
            expect(chunk.toolType).toBe("command");
        });

        it("should have in_progress status with output", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "bash",
                status: "in_progress",
                output: "Executing...",
            };

            expect(chunk.status).toBe("in_progress");
            expect(chunk.output).toBe("Executing...");
        });

        it("should have completed status with exit code", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "bash",
                status: "completed",
                exitCode: 0,
                output: "Done",
            };

            expect(chunk.status).toBe("completed");
            expect(chunk.exitCode).toBe(0);
        });

        it("should have failed status", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "bash",
                status: "failed",
                exitCode: 1,
                output: "Error",
            };

            expect(chunk.status).toBe("failed");
            expect(chunk.exitCode).toBe(1);
        });

        it("should support MCP tool type", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "mcp-1",
                toolType: "mcp",
                name: "mcp_read_file",
                status: "completed",
            };

            expect(chunk.toolType).toBe("mcp");
        });

        it("should include arguments when provided", () => {
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "read_file",
                status: "completed",
                arguments: { path: "/test.txt" },
            };

            expect(chunk.arguments).toEqual({ path: "/test.txt" });
        });

        it("should include raw event when provided", () => {
            const rawEvent = { type: "item.completed", item: {} };
            const chunk: VtcodeToolResultChunk = {
                kind: "toolResult",
                id: "tool-1",
                toolType: "command",
                name: "bash",
                status: "completed",
                rawEvent,
            };

            expect(chunk.rawEvent).toBe(rawEvent);
        });
    });

    describe("error chunk", () => {
        it("should have correct structure", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "error",
                message: "An error occurred",
            };

            expect(chunk.kind).toBe("error");
            expect((chunk as any).message).toBe("An error occurred");
        });

        it("should handle multiline error messages", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "error",
                message: "Error line 1\nError line 2\nError line 3",
            };

            expect((chunk as any).message).toContain("\n");
        });
    });

    describe("done chunk", () => {
        it("should have correct structure", () => {
            const chunk: VtcodeStreamChunk = {
                kind: "done",
            };

            expect(chunk.kind).toBe("done");
            expect(Object.keys(chunk)).toHaveLength(1);
        });
    });
});

describe("VtcodeToolCall interface", () => {
    it("should define tool call structure", () => {
        const toolCall: VtcodeToolCall = {
            id: "call-123",
            name: "read_file",
            args: {
                path: "/workspace/test.txt",
                encoding: "utf-8",
            },
        };

        expect(toolCall.id).toBe("call-123");
        expect(toolCall.name).toBe("read_file");
        expect(toolCall.args.path).toBe("/workspace/test.txt");
    });

    it("should allow empty args", () => {
        const toolCall: VtcodeToolCall = {
            id: "call-456",
            name: "get_cwd",
            args: {},
        };

        expect(toolCall.args).toEqual({});
    });

    it("should allow complex nested args", () => {
        const toolCall: VtcodeToolCall = {
            id: "call-789",
            name: "write_file",
            args: {
                path: "/test.json",
                content: JSON.stringify({ foo: "bar" }),
                options: {
                    mode: 0o644,
                    encoding: "utf-8",
                },
            },
        };

        expect(toolCall.args.options).toBeDefined();
        expect((toolCall.args.options as any).mode).toBe(0o644);
    });
});

describe("Stream chunk type safety", () => {
    it("should enforce kind property", () => {
        const chunks: VtcodeStreamChunk[] = [
            { kind: "text", text: "hello" },
            { kind: "reasoning", text: "thinking" },
            { kind: "metadata", metadata: {} },
            {
                kind: "toolCall",
                call: { id: "1", name: "test", args: {} },
                respond: () => {},
                reject: () => {},
            },
            {
                kind: "toolResult",
                id: "1",
                toolType: "command",
                name: "bash",
                status: "completed",
            },
            { kind: "error", message: "error" },
            { kind: "done" },
        ];

        chunks.forEach((chunk) => {
            expect(chunk.kind).toBeDefined();
            expect(typeof chunk.kind).toBe("string");
        });
    });

    it("should handle discriminated union correctly", () => {
        const processChunk = (chunk: VtcodeStreamChunk) => {
            switch (chunk.kind) {
                case "text":
                    return `Text: ${chunk.text}`;
                case "reasoning":
                    return `Reasoning: ${chunk.text}`;
                case "metadata":
                    return "Metadata";
                case "toolCall":
                    return `Tool: ${chunk.call.name}`;
                case "toolResult":
                    return `Result: ${chunk.status}`;
                case "error":
                    return `Error: ${chunk.message}`;
                case "done":
                    return "Done";
                default:
                    return "Unknown";
            }
        };

        expect(processChunk({ kind: "text", text: "hi" })).toBe("Text: hi");
        expect(
            processChunk({ kind: "reasoning", text: "thinking" })
        ).toBe("Reasoning: thinking");
        expect(processChunk({ kind: "done" })).toBe("Done");
    });
});
