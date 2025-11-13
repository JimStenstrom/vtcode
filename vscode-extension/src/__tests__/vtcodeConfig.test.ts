import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import {
    pickVtcodeConfigUri,
    VtcodeConfigSummary,
    VtcodeMcpProviderSummary,
} from "../vtcodeConfig";
import * as vscode from "vscode";

describe("vtcodeConfig", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    describe("pickVtcodeConfigUri", () => {
        it("should return undefined when no vtcode.toml files found", async () => {
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([]);

            const result = await pickVtcodeConfigUri();

            expect(result).toBeUndefined();
            expect(vscode.workspace.findFiles).toHaveBeenCalledWith(
                "**/vtcode.toml",
                "**/{node_modules,dist,out,.git,target}/**",
                10
            );
        });

        it("should return the only file when one vtcode.toml found", async () => {
            const mockUri = vscode.Uri.file("/workspace/vtcode.toml");
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([
                mockUri,
            ]);

            const result = await pickVtcodeConfigUri();

            expect(result).toBe(mockUri);
        });

        it("should return preferred uri when it exists in matches", async () => {
            const preferredUri = vscode.Uri.file("/workspace/vtcode.toml");
            const otherUri = vscode.Uri.file("/workspace/sub/vtcode.toml");
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([
                preferredUri,
                otherUri,
            ]);

            const result = await pickVtcodeConfigUri(preferredUri);

            expect(result).toBe(preferredUri);
        });

        it("should prompt user when multiple files found", async () => {
            const uri1 = vscode.Uri.file("/workspace/vtcode.toml");
            const uri2 = vscode.Uri.file("/workspace/sub/vtcode.toml");
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([
                uri1,
                uri2,
            ]);
            vi.spyOn(vscode.workspace, "asRelativePath").mockImplementation(
                (uri: any) => uri.fsPath.replace("/workspace/", "")
            );

            const showQuickPickSpy = vi
                .spyOn(vscode.window, "showQuickPick")
                .mockResolvedValue({ label: "vtcode.toml", uri: uri1 });

            const result = await pickVtcodeConfigUri();

            expect(showQuickPickSpy).toHaveBeenCalledWith(
                expect.arrayContaining([
                    expect.objectContaining({ uri: uri1 }),
                    expect.objectContaining({ uri: uri2 }),
                ]),
                expect.objectContaining({
                    placeHolder: expect.stringContaining("Select"),
                })
            );
            expect(result).toBe(uri1);
        });

        it("should return undefined when user cancels selection", async () => {
            const uri1 = vscode.Uri.file("/workspace/vtcode.toml");
            const uri2 = vscode.Uri.file("/workspace/sub/vtcode.toml");
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([
                uri1,
                uri2,
            ]);
            vi.spyOn(vscode.workspace, "asRelativePath").mockReturnValue(
                "vtcode.toml"
            );
            vi.spyOn(vscode.window, "showQuickPick").mockResolvedValue(
                undefined
            );

            const result = await pickVtcodeConfigUri();

            expect(result).toBeUndefined();
        });

        it("should exclude common build directories", async () => {
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([]);

            await pickVtcodeConfigUri();

            expect(vscode.workspace.findFiles).toHaveBeenCalledWith(
                "**/vtcode.toml",
                expect.stringContaining("node_modules"),
                10
            );
            const call = (vscode.workspace.findFiles as any).mock.calls[0];
            expect(call[1]).toContain("node_modules");
            expect(call[1]).toContain("dist");
            expect(call[1]).toContain(".git");
            expect(call[1]).toContain("target");
        });

        it("should limit search to 10 files", async () => {
            vi.spyOn(vscode.workspace, "findFiles").mockResolvedValue([]);

            await pickVtcodeConfigUri();

            const call = (vscode.workspace.findFiles as any).mock.calls[0];
            expect(call[2]).toBe(10);
        });
    });

    describe("VtcodeConfigSummary interface", () => {
        it("should define config summary structure", () => {
            const summary: VtcodeConfigSummary = {
                hasConfig: true,
                uri: vscode.Uri.file("/workspace/vtcode.toml"),
                humanInTheLoop: true,
                toolDefaultPolicy: "require_approval",
                toolPoliciesCount: 5,
                mcpEnabled: true,
                mcpProviders: [
                    {
                        name: "test-provider",
                        enabled: true,
                        command: "test",
                        args: ["arg1"],
                    },
                ],
                agentProvider: "anthropic",
                agentDefaultModel: "claude-sonnet-4-5",
                automationFullAutoEnabled: false,
                automationFullAutoAllowedTools: ["read", "write"],
            };

            expect(summary.hasConfig).toBe(true);
            expect(summary.uri?.fsPath).toContain("vtcode.toml");
            expect(summary.humanInTheLoop).toBe(true);
            expect(summary.mcpProviders).toHaveLength(1);
        });

        it("should allow minimal config summary", () => {
            const summary: VtcodeConfigSummary = {
                hasConfig: false,
                mcpProviders: [],
            };

            expect(summary.hasConfig).toBe(false);
            expect(summary.mcpProviders).toEqual([]);
        });

        it("should allow config summary with parse error", () => {
            const summary: VtcodeConfigSummary = {
                hasConfig: false,
                mcpProviders: [],
                parseError: "Invalid TOML syntax",
            };

            expect(summary.parseError).toBeDefined();
            expect(summary.parseError).toContain("Invalid TOML");
        });
    });

    describe("VtcodeMcpProviderSummary interface", () => {
        it("should define MCP provider structure", () => {
            const provider: VtcodeMcpProviderSummary = {
                name: "test-provider",
                enabled: true,
                command: "/usr/bin/test",
                args: ["--flag", "value"],
            };

            expect(provider.name).toBe("test-provider");
            expect(provider.enabled).toBe(true);
            expect(provider.command).toBe("/usr/bin/test");
            expect(provider.args).toEqual(["--flag", "value"]);
        });

        it("should allow minimal MCP provider", () => {
            const provider: VtcodeMcpProviderSummary = {
                name: "minimal-provider",
            };

            expect(provider.name).toBe("minimal-provider");
            expect(provider.enabled).toBeUndefined();
            expect(provider.command).toBeUndefined();
            expect(provider.args).toBeUndefined();
        });
    });
});
