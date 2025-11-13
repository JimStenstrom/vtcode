import { describe, it, expect, beforeEach, vi } from "vitest";
import {
    getConfiguredCommandPath,
    getWorkspaceRoot,
    getConfigArguments,
    createSpawnOptions,
    formatArgsForLogging,
    RunVtcodeCommandOptions,
    VtcodeTaskDefinition,
} from "../vtcodeRunner";
import * as vscode from "vscode";

describe("vtcodeRunner utilities", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe("getConfiguredCommandPath", () => {
        it("should return default 'vtcode' when not configured", () => {
            vi.spyOn(vscode.workspace, "getConfiguration").mockReturnValue({
                get: vi.fn().mockReturnValue("vtcode"),
            } as any);

            const path = getConfiguredCommandPath();
            expect(path).toBe("vtcode");
        });

        it("should return configured path when set", () => {
            vi.spyOn(vscode.workspace, "getConfiguration").mockReturnValue({
                get: vi.fn().mockReturnValue("/usr/local/bin/vtcode"),
            } as any);

            const path = getConfiguredCommandPath();
            expect(path).toBe("/usr/local/bin/vtcode");
        });

        it("should trim whitespace from path", () => {
            vi.spyOn(vscode.workspace, "getConfiguration").mockReturnValue({
                get: vi.fn().mockReturnValue("  vtcode  "),
            } as any);

            const path = getConfiguredCommandPath();
            expect(path).toBe("vtcode");
        });

        it("should return 'vtcode' for empty string", () => {
            vi.spyOn(vscode.workspace, "getConfiguration").mockReturnValue({
                get: vi.fn().mockReturnValue(""),
            } as any);

            const path = getConfiguredCommandPath();
            expect(path).toBe("vtcode");
        });

        it("should handle paths with spaces", () => {
            vi.spyOn(vscode.workspace, "getConfiguration").mockReturnValue({
                get: vi.fn().mockReturnValue("/path with spaces/vtcode"),
            } as any);

            const path = getConfiguredCommandPath();
            expect(path).toBe("/path with spaces/vtcode");
        });
    });

    describe("getWorkspaceRoot", () => {
        it("should return undefined when no workspace", () => {
            vi.spyOn(vscode.window, "activeTextEditor", "get").mockReturnValue(
                undefined
            );
            vi.spyOn(
                vscode.workspace,
                "workspaceFolders",
                "get"
            ).mockReturnValue(undefined);

            const root = getWorkspaceRoot();
            expect(root).toBeUndefined();
        });

        it("should return active editor workspace", () => {
            const mockUri = vscode.Uri.file("/workspace/file.ts");
            const mockWorkspace = {
                uri: { fsPath: "/workspace" },
            } as vscode.WorkspaceFolder;

            vi.spyOn(vscode.window, "activeTextEditor", "get").mockReturnValue({
                document: { uri: mockUri },
            } as any);
            vi.spyOn(vscode.workspace, "getWorkspaceFolder").mockReturnValue(
                mockWorkspace
            );

            const root = getWorkspaceRoot();
            expect(root).toBe("/workspace");
        });

        it("should return first workspace when no active editor", () => {
            const mockWorkspace = {
                uri: { fsPath: "/first-workspace" },
            } as vscode.WorkspaceFolder;

            vi.spyOn(vscode.window, "activeTextEditor", "get").mockReturnValue(
                undefined
            );
            vi.spyOn(
                vscode.workspace,
                "workspaceFolders",
                "get"
            ).mockReturnValue([mockWorkspace]);

            const root = getWorkspaceRoot();
            expect(root).toBe("/first-workspace");
        });

        it("should prefer active editor over first workspace", () => {
            const mockEditorWorkspace = {
                uri: { fsPath: "/editor-workspace" },
            } as vscode.WorkspaceFolder;
            const mockFirstWorkspace = {
                uri: { fsPath: "/first-workspace" },
            } as vscode.WorkspaceFolder;

            vi.spyOn(vscode.window, "activeTextEditor", "get").mockReturnValue({
                document: { uri: vscode.Uri.file("/editor-workspace/file.ts") },
            } as any);
            vi.spyOn(vscode.workspace, "getWorkspaceFolder").mockReturnValue(
                mockEditorWorkspace
            );
            vi.spyOn(
                vscode.workspace,
                "workspaceFolders",
                "get"
            ).mockReturnValue([mockFirstWorkspace]);

            const root = getWorkspaceRoot();
            expect(root).toBe("/editor-workspace");
        });
    });

    describe("getConfigArguments", () => {
        it("should return empty array when no config uri", () => {
            const args = getConfigArguments();
            expect(args).toEqual([]);
        });

        it("should return empty array for undefined", () => {
            const args = getConfigArguments(undefined);
            expect(args).toEqual([]);
        });

        it("should return config arguments when uri provided", () => {
            const uri = vscode.Uri.file("/workspace/vtcode.toml");
            const args = getConfigArguments(uri);

            expect(args).toEqual(["--config", "/workspace/vtcode.toml"]);
        });

        it("should handle paths with spaces", () => {
            const uri = vscode.Uri.file("/path with spaces/vtcode.toml");
            const args = getConfigArguments(uri);

            expect(args).toEqual([
                "--config",
                "/path with spaces/vtcode.toml",
            ]);
        });
    });

    describe("createSpawnOptions", () => {
        it("should create basic spawn options", () => {
            const options = createSpawnOptions();

            expect(options).toBeDefined();
            expect(options.env).toBeDefined();
        });

        it("should merge overrides", () => {
            const options = createSpawnOptions({
                cwd: "/workspace",
                shell: true,
            });

            expect(options.cwd).toBe("/workspace");
            expect(options.shell).toBe(true);
        });

        it("should merge environment variables", () => {
            const options = createSpawnOptions({
                env: {
                    CUSTOM_VAR: "value",
                },
            });

            expect(options.env?.CUSTOM_VAR).toBe("value");
        });

        it("should apply environment provider", () => {
            const envProvider = () => ({
                API_KEY: "secret",
                DEBUG: "true",
            });

            const options = createSpawnOptions({}, envProvider);

            expect(options.env?.API_KEY).toBe("secret");
            expect(options.env?.DEBUG).toBe("true");
        });

        it("should override provider env with explicit env", () => {
            const envProvider = () => ({
                VAR: "from-provider",
            });

            const options = createSpawnOptions(
                {
                    env: {
                        VAR: "from-override",
                    },
                },
                envProvider
            );

            expect(options.env?.VAR).toBe("from-override");
        });

        it("should handle provider errors gracefully", () => {
            const envProvider = () => {
                throw new Error("Provider error");
            };

            expect(() => {
                createSpawnOptions({}, envProvider);
            }).not.toThrow();
        });

        it("should preserve process.env", () => {
            const options = createSpawnOptions();

            expect(options.env).toBeDefined();
            // Should have at least some environment variables
            expect(Object.keys(options.env || {}).length).toBeGreaterThan(0);
        });
    });

    describe("formatArgsForLogging", () => {
        it("should format simple arguments", () => {
            const args = ["ask", "hello"];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toBe("ask hello");
        });

        it("should quote arguments with spaces", () => {
            const args = ["ask", "hello world"];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toContain('"hello world"');
        });

        it("should quote arguments with quotes", () => {
            const args = ["ask", 'say "hello"'];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toContain(JSON.stringify('say "hello"'));
        });

        it("should handle empty array", () => {
            const formatted = formatArgsForLogging([]);
            expect(formatted).toBe("");
        });

        it("should handle single argument", () => {
            const formatted = formatArgsForLogging(["analyze"]);
            expect(formatted).toBe("analyze");
        });

        it("should handle arguments with single quotes", () => {
            const args = ["ask", "what's up"];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toContain('"what\'s up"');
        });

        it("should handle mixed arguments", () => {
            const args = ["ask", "simple", "with spaces", "another"];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toContain("simple");
            expect(formatted).toContain('"with spaces"');
            expect(formatted).toContain("another");
        });

        it("should handle special characters", () => {
            const args = ["--config", "/path/to/file"];
            const formatted = formatArgsForLogging(args);

            expect(formatted).toBe("--config /path/to/file");
        });
    });
});

describe("RunVtcodeCommandOptions interface", () => {
    it("should define options structure", () => {
        const options: RunVtcodeCommandOptions = {
            title: "Running command",
            revealOutput: true,
            showProgress: true,
        };

        expect(options.title).toBe("Running command");
        expect(options.revealOutput).toBe(true);
        expect(options.showProgress).toBe(true);
    });

    it("should support callbacks", () => {
        const onStdout = vi.fn();
        const onStderr = vi.fn();

        const options: RunVtcodeCommandOptions = {
            onStdout,
            onStderr,
        };

        options.onStdout?.("test output");
        options.onStderr?.("test error");

        expect(onStdout).toHaveBeenCalledWith("test output");
        expect(onStderr).toHaveBeenCalledWith("test error");
    });

    it("should support cancellation token", () => {
        const mockToken = {
            isCancellationRequested: false,
            onCancellationRequested: vi.fn(),
        } as any;

        const options: RunVtcodeCommandOptions = {
            cancellationToken: mockToken,
        };

        expect(options.cancellationToken).toBeDefined();
    });

    it("should support output channel", () => {
        const mockOutput = {
            appendLine: vi.fn(),
            append: vi.fn(),
        } as any;

        const options: RunVtcodeCommandOptions = {
            output: mockOutput,
        };

        expect(options.output).toBeDefined();
    });

    it("should support all optional fields", () => {
        const options: RunVtcodeCommandOptions = {
            title: "Test",
            revealOutput: false,
            showProgress: false,
            onStdout: vi.fn(),
            onStderr: vi.fn(),
            cancellationToken: {} as any,
            output: {} as any,
        };

        expect(Object.keys(options)).toHaveLength(7);
    });

    it("should allow empty options", () => {
        const options: RunVtcodeCommandOptions = {};
        expect(options).toEqual({});
    });
});

describe("VtcodeTaskDefinition interface", () => {
    it("should define task structure", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
        };

        expect(task.type).toBe("vtcode");
        expect(task.command).toBe("update-plan");
    });

    it("should support summary field", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
            summary: "Update project plan",
        };

        expect(task.summary).toBe("Update project plan");
    });

    it("should support steps array", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
            steps: ["Step 1", "Step 2", "Step 3"],
        };

        expect(task.steps).toHaveLength(3);
        expect(task.steps).toContain("Step 1");
    });

    it("should support label field", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
            label: "My Custom Task",
        };

        expect(task.label).toBe("My Custom Task");
    });

    it("should support all optional fields", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
            summary: "Summary",
            steps: ["Step 1"],
            label: "Label",
        };

        expect(task.summary).toBeDefined();
        expect(task.steps).toBeDefined();
        expect(task.label).toBeDefined();
    });

    it("should only support update-plan command", () => {
        const task: VtcodeTaskDefinition = {
            type: "vtcode",
            command: "update-plan",
        };

        // Type should enforce command: "update-plan"
        expect(task.command).toBe("update-plan");
    });
});

describe("vtcodeRunner integration", () => {
    it("should support complete command execution flow", () => {
        const commandPath = getConfiguredCommandPath();
        const workspaceRoot = getWorkspaceRoot();
        const configArgs = getConfigArguments();

        const args = ["analyze"];
        const finalArgs = [...configArgs, ...args];
        const displayArgs = formatArgsForLogging(finalArgs);

        expect(commandPath).toBeDefined();
        expect(displayArgs).toContain("analyze");
    });

    it("should build spawn options correctly", () => {
        const options = createSpawnOptions({
            cwd: getWorkspaceRoot(),
        });

        expect(options).toBeDefined();
        expect(options.env).toBeDefined();
    });

    it("should format command for logging", () => {
        const commandPath = "vtcode";
        const args = ["ask", "hello world"];
        const formatted = formatArgsForLogging(args);

        const fullCommand = `${commandPath} ${formatted}`;
        expect(fullCommand).toContain("vtcode");
        expect(fullCommand).toContain('"hello world"');
    });
});
