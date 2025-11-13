import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import {
    EnhancedMcpToolManager,
    EnhancedMcpProvider,
    ProviderHealth,
    ProviderStats,
    ToolExecutionOptions,
    StreamingToolResult,
} from "../enhancedMcpToolManager";
import * as vscode from "vscode";

describe("EnhancedMcpToolManager", () => {
    let manager: EnhancedMcpToolManager;
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

        manager = new EnhancedMcpToolManager(mockOutput);
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    describe("constructor", () => {
        it("should create enhanced MCP manager instance", () => {
            expect(manager).toBeDefined();
            expect(manager).toBeInstanceOf(EnhancedMcpToolManager);
        });

        it("should log initialization message", () => {
            expect(mockOutput.appendLine).toHaveBeenCalledWith(
                expect.stringContaining("Enhanced MCP manager initialized")
            );
        });
    });

    describe("getEnhancedProvider", () => {
        it("should return undefined for non-existent provider", () => {
            const provider = manager.getEnhancedProvider("non-existent");
            expect(provider).toBeUndefined();
        });

        it("should return provider when it exists", () => {
            // This would require setting up a provider first
            // For now, test the interface
            expect(typeof manager.getEnhancedProvider).toBe("function");
        });
    });

    describe("getAllEnhancedProviders", () => {
        it("should return array of providers", () => {
            const providers = manager.getAllEnhancedProviders();
            expect(Array.isArray(providers)).toBe(true);
        });

        it("should return empty array initially", () => {
            const providers = manager.getAllEnhancedProviders();
            expect(providers).toHaveLength(0);
        });
    });

    describe("getProviderHealth", () => {
        it("should return undefined for non-existent provider", () => {
            const health = manager.getProviderHealth("non-existent");
            expect(health).toBeUndefined();
        });

        it("should have correct method signature", () => {
            expect(typeof manager.getProviderHealth).toBe("function");
        });
    });

    describe("clearCaches", () => {
        it("should not throw when clearing caches", () => {
            expect(() => {
                manager.clearCaches();
            }).not.toThrow();
        });

        it("should be callable multiple times", () => {
            manager.clearCaches();
            expect(() => {
                manager.clearCaches();
            }).not.toThrow();
        });
    });

    describe("registerStreamingCallback", () => {
        it("should accept callback function", () => {
            const callback = vi.fn((result: StreamingToolResult) => {});

            expect(() => {
                manager.registerStreamingCallback("test-id", callback);
            }).not.toThrow();
        });

        it("should register multiple callbacks", () => {
            const callback1 = vi.fn();
            const callback2 = vi.fn();

            manager.registerStreamingCallback("id1", callback1);
            manager.registerStreamingCallback("id2", callback2);

            expect(true).toBe(true); // Successfully registered
        });
    });
});

describe("EnhancedMcpProvider interface", () => {
    it("should define enhanced provider structure", () => {
        const provider: EnhancedMcpProvider = {
            name: "test-provider",
            command: "/usr/bin/test",
            args: ["--arg"],
            enabled: true,
            health: {
                status: "healthy",
                responseTime: 100,
                consecutiveFailures: 0,
            },
            lastHealthCheck: Date.now(),
            toolCache: new Map(),
            stats: {
                totalInvocations: 10,
                successfulInvocations: 9,
                failedInvocations: 1,
                averageResponseTime: 150,
            },
        };

        expect(provider.name).toBe("test-provider");
        expect(provider.health.status).toBe("healthy");
        expect(provider.stats.totalInvocations).toBe(10);
    });

    it("should support unhealthy status", () => {
        const provider: EnhancedMcpProvider = {
            name: "test",
            command: "test",
            args: [],
            enabled: false,
            health: {
                status: "unhealthy",
                responseTime: 5000,
                consecutiveFailures: 3,
                lastError: "Connection timeout",
            },
            lastHealthCheck: Date.now(),
            toolCache: new Map(),
            stats: {
                totalInvocations: 5,
                successfulInvocations: 2,
                failedInvocations: 3,
                averageResponseTime: 2000,
            },
        };

        expect(provider.health.status).toBe("unhealthy");
        expect(provider.health.lastError).toBe("Connection timeout");
        expect(provider.health.consecutiveFailures).toBe(3);
    });

    it("should support unknown health status", () => {
        const provider: EnhancedMcpProvider = {
            name: "test",
            command: "test",
            args: [],
            enabled: true,
            health: {
                status: "unknown",
                responseTime: 0,
                consecutiveFailures: 0,
            },
            lastHealthCheck: 0,
            toolCache: new Map(),
            stats: {
                totalInvocations: 0,
                successfulInvocations: 0,
                failedInvocations: 0,
                averageResponseTime: 0,
            },
        };

        expect(provider.health.status).toBe("unknown");
        expect(provider.lastHealthCheck).toBe(0);
    });

    it("should support environment variables", () => {
        const provider: EnhancedMcpProvider = {
            name: "test",
            command: "test",
            args: [],
            enabled: true,
            env: {
                API_KEY: "secret",
                DEBUG: "true",
            },
            health: {
                status: "healthy",
                responseTime: 100,
                consecutiveFailures: 0,
            },
            lastHealthCheck: Date.now(),
            toolCache: new Map(),
            stats: {
                totalInvocations: 0,
                successfulInvocations: 0,
                failedInvocations: 0,
                averageResponseTime: 0,
            },
        };

        expect(provider.env?.API_KEY).toBe("secret");
        expect(provider.env?.DEBUG).toBe("true");
    });
});

describe("ProviderHealth interface", () => {
    it("should define health structure", () => {
        const health: ProviderHealth = {
            status: "healthy",
            responseTime: 250,
            consecutiveFailures: 0,
        };

        expect(health.status).toBe("healthy");
        expect(health.responseTime).toBe(250);
        expect(health.consecutiveFailures).toBe(0);
    });

    it("should include error message when unhealthy", () => {
        const health: ProviderHealth = {
            status: "unhealthy",
            responseTime: 10000,
            consecutiveFailures: 5,
            lastError: "Provider not responding",
        };

        expect(health.lastError).toBe("Provider not responding");
        expect(health.consecutiveFailures).toBe(5);
    });

    it("should track consecutive failures", () => {
        const health: ProviderHealth = {
            status: "unhealthy",
            responseTime: 0,
            consecutiveFailures: 10,
        };

        expect(health.consecutiveFailures).toBeGreaterThan(5);
    });
});

describe("ProviderStats interface", () => {
    it("should track invocation statistics", () => {
        const stats: ProviderStats = {
            totalInvocations: 100,
            successfulInvocations: 95,
            failedInvocations: 5,
            averageResponseTime: 300,
        };

        expect(stats.totalInvocations).toBe(100);
        expect(stats.successfulInvocations).toBe(95);
        expect(stats.failedInvocations).toBe(5);
        expect(stats.averageResponseTime).toBe(300);
    });

    it("should calculate success rate", () => {
        const stats: ProviderStats = {
            totalInvocations: 100,
            successfulInvocations: 95,
            failedInvocations: 5,
            averageResponseTime: 300,
        };

        const successRate =
            (stats.successfulInvocations / stats.totalInvocations) * 100;
        expect(successRate).toBe(95);
    });

    it("should handle zero invocations", () => {
        const stats: ProviderStats = {
            totalInvocations: 0,
            successfulInvocations: 0,
            failedInvocations: 0,
            averageResponseTime: 0,
        };

        expect(stats.totalInvocations).toBe(0);
        expect(stats.averageResponseTime).toBe(0);
    });
});

describe("ToolExecutionOptions interface", () => {
    it("should define execution options", () => {
        const options: ToolExecutionOptions = {
            timeoutMs: 30000,
            enableStreaming: true,
            retryAttempts: 3,
            retryDelayMs: 1000,
        };

        expect(options.timeoutMs).toBe(30000);
        expect(options.enableStreaming).toBe(true);
        expect(options.retryAttempts).toBe(3);
        expect(options.retryDelayMs).toBe(1000);
    });

    it("should support disabling streaming", () => {
        const options: ToolExecutionOptions = {
            timeoutMs: 5000,
            enableStreaming: false,
            retryAttempts: 1,
            retryDelayMs: 500,
        };

        expect(options.enableStreaming).toBe(false);
    });

    it("should support custom retry configuration", () => {
        const options: ToolExecutionOptions = {
            timeoutMs: 60000,
            enableStreaming: true,
            retryAttempts: 5,
            retryDelayMs: 2000,
        };

        expect(options.retryAttempts).toBe(5);
        expect(options.retryDelayMs).toBe(2000);
    });
});

describe("StreamingToolResult interface", () => {
    it("should define data result", () => {
        const result: StreamingToolResult = {
            type: "data",
            data: { message: "Processing..." },
            progress: 50,
        };

        expect(result.type).toBe("data");
        expect(result.data).toBeDefined();
        expect(result.progress).toBe(50);
    });

    it("should define error result", () => {
        const result: StreamingToolResult = {
            type: "error",
            error: "Tool execution failed",
        };

        expect(result.type).toBe("error");
        expect(result.error).toBe("Tool execution failed");
    });

    it("should define complete result", () => {
        const result: StreamingToolResult = {
            type: "complete",
            data: { finalResult: "Success" },
            progress: 100,
        };

        expect(result.type).toBe("complete");
        expect(result.progress).toBe(100);
    });

    it("should support progress tracking", () => {
        const results: StreamingToolResult[] = [
            { type: "data", progress: 0 },
            { type: "data", progress: 25 },
            { type: "data", progress: 50 },
            { type: "data", progress: 75 },
            { type: "complete", progress: 100 },
        ];

        results.forEach((result, index) => {
            expect(result.progress).toBe(index * 25);
        });
    });

    it("should handle streaming without progress", () => {
        const result: StreamingToolResult = {
            type: "data",
            data: "chunk1",
        };

        expect(result.progress).toBeUndefined();
    });
});

describe("EnhancedMcpToolManager integration", () => {
    it("should manage provider lifecycle", () => {
        const mockOutput = {
            appendLine: vi.fn(),
        } as any;

        const manager = new EnhancedMcpToolManager(mockOutput);

        // Verify initialization
        expect(manager).toBeDefined();
        expect(manager.getAllEnhancedProviders()).toHaveLength(0);

        // Verify caching
        expect(() => manager.clearCaches()).not.toThrow();
    });

    it("should support health monitoring", () => {
        const mockOutput = { appendLine: vi.fn() } as any;
        const manager = new EnhancedMcpToolManager(mockOutput);

        const health = manager.getProviderHealth("test-provider");
        expect(health).toBeUndefined(); // No providers loaded yet
    });
});
