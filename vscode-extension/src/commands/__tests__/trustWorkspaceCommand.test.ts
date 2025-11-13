import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { TrustWorkspaceCommand } from "../trustWorkspaceCommand";
import { CommandContext } from "../../types/command";
import * as vscode from "vscode";

describe("TrustWorkspaceCommand", () => {
    let command: TrustWorkspaceCommand;
    let mockContext: CommandContext;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        command = new TrustWorkspaceCommand();
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
        expect(command.id).toBe("vtcode.trustWorkspace");
        expect(command.title).toBe("Trust Workspace");
        expect(command.icon).toBe("shield");
    });

    it("should have description", () => {
        expect(command.description).toBeDefined();
        expect(command.description).toContain("workspace trust");
    });

    it("should show message if workspace is already trusted", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(true);
        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("already trusted")
        );
    });

    it("should request workspace trust when not trusted", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);

        // Mock requestWorkspaceTrust API
        const mockWorkspace = vscode.workspace as any;
        mockWorkspace.requestWorkspaceTrust = vi.fn().mockResolvedValue(true);

        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(mockWorkspace.requestWorkspaceTrust).toHaveBeenCalledWith(
            expect.objectContaining({
                modal: true,
                message: expect.stringContaining("VTCode requires"),
            })
        );
        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("Workspace trust granted")
        );
    });

    it("should offer trust management when trust request fails", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);

        // Mock requestWorkspaceTrust API to return false
        const mockWorkspace = vscode.workspace as any;
        mockWorkspace.requestWorkspaceTrust = vi.fn().mockResolvedValue(false);

        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("still required"),
            "Manage Workspace Trust"
        );
    });

    it("should open trust management when user selects option", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get")
            .mockReturnValueOnce(false) // Initial check
            .mockReturnValueOnce(false) // After request
            .mockReturnValueOnce(false) // Before management
            .mockReturnValueOnce(true); // After management

        const mockWorkspace = vscode.workspace as any;
        mockWorkspace.requestWorkspaceTrust = vi.fn().mockResolvedValue(false);

        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValueOnce("Manage Workspace Trust" as any)
            .mockResolvedValue(undefined);

        const executeCommandSpy = vi
            .spyOn(vscode.commands, "executeCommand")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(executeCommandSpy).toHaveBeenCalledWith(
            "workbench.action.manageTrust"
        );
        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("Workspace trust granted")
        );
    });

    it("should not open trust management when user cancels", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);

        const mockWorkspace = vscode.workspace as any;
        mockWorkspace.requestWorkspaceTrust = vi.fn().mockResolvedValue(false);

        vi.spyOn(vscode.window, "showInformationMessage").mockResolvedValue(
            undefined
        );

        const executeCommandSpy = vi
            .spyOn(vscode.commands, "executeCommand")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(executeCommandSpy).not.toHaveBeenCalled();
    });

    it("should handle missing requestWorkspaceTrust API gracefully", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);

        // Remove requestWorkspaceTrust API
        const mockWorkspace = vscode.workspace as any;
        delete mockWorkspace.requestWorkspaceTrust;

        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("still required"),
            "Manage Workspace Trust"
        );
    });

    it("should handle requestWorkspaceTrust throwing errors", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);

        const mockWorkspace = vscode.workspace as any;
        mockWorkspace.requestWorkspaceTrust = vi
            .fn()
            .mockRejectedValue(new Error("API error"));

        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        // Should fall back to manual trust management
        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("still required"),
            "Manage Workspace Trust"
        );
    });

    it("should return early when workspace becomes trusted", async () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(true);

        const mockWorkspace = vscode.workspace as any;
        const requestSpy = vi.fn();
        mockWorkspace.requestWorkspaceTrust = requestSpy;

        await command.execute(mockContext);

        expect(requestSpy).not.toHaveBeenCalled();
    });

    it("should check workspace trust on canExecute", () => {
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(true);
        expect(command.canExecute(mockContext)).toBe(true);

        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(false);
        expect(command.canExecute(mockContext)).toBe(false);
    });
});
