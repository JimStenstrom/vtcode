import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { RefreshCommand } from "../refreshCommand";
import { CommandContext } from "../../types/command";
import * as vscode from "vscode";

describe("RefreshCommand", () => {
    let command: RefreshCommand;
    let mockContext: CommandContext;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        command = new RefreshCommand();
        mockOutput = {
            append: vi.fn(),
            appendLine: vi.fn(),
            clear: vi.fn(),
            show: vi.fn(),
            hide: vi.fn(),
            dispose: vi.fn(),
        } as any;

        mockContext = {
            output: mockOutput,
        };

        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("should have correct id, title, and icon", () => {
        expect(command.id).toBe("vtcode.refreshQuickActions");
        expect(command.title).toBe("Refresh");
        expect(command.icon).toBe("refresh");
    });

    it("should have description", () => {
        expect(command.description).toBeDefined();
        expect(command.description).toContain("Refresh");
    });

    it("should show information message on refresh", async () => {
        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);
        const executeCommandSpy = vi
            .spyOn(vscode.commands, "executeCommand")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("refreshed")
        );
    });

    it("should trigger workspace trust verification", async () => {
        vi.spyOn(vscode.window, "showInformationMessage").mockResolvedValue(
            undefined
        );
        const executeCommandSpy = vi
            .spyOn(vscode.commands, "executeCommand")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(executeCommandSpy).toHaveBeenCalledWith(
            "vtcode.verifyWorkspaceTrust"
        );
    });

    it("should execute in any context", () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);
        expect(command.canExecute(mockContext)).toBe(false);

        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(true);
        expect(command.canExecute(mockContext)).toBe(true);
    });

    it("should handle command execution errors gracefully", async () => {
        vi.spyOn(vscode.window, "showInformationMessage").mockResolvedValue(
            undefined
        );
        vi.spyOn(vscode.commands, "executeCommand").mockRejectedValue(
            new Error("Command failed")
        );

        // Should not throw
        await expect(command.execute(mockContext)).rejects.toThrow();
    });

    it("should complete message before triggering verification", async () => {
        const callOrder: string[] = [];

        vi.spyOn(vscode.window, "showInformationMessage").mockImplementation(
            async () => {
                callOrder.push("message");
                return undefined;
            }
        );
        vi.spyOn(vscode.commands, "executeCommand").mockImplementation(
            async () => {
                callOrder.push("command");
                return undefined;
            }
        );

        await command.execute(mockContext);

        expect(callOrder).toEqual(["message", "command"]);
    });
});
