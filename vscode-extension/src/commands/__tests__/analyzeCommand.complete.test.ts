import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { AnalyzeCommand } from "../analyzeCommand";
import { CommandContext } from "../../types/command";
import * as vscode from "vscode";
import * as vtcodeRunner from "../../utils/vtcodeRunner";

vi.mock("../../utils/vtcodeRunner");

describe("AnalyzeCommand", () => {
    let command: AnalyzeCommand;
    let mockContext: CommandContext;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        command = new AnalyzeCommand();
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
        expect(command.id).toBe("vtcode.runAnalyze");
        expect(command.title).toBe("Analyze Workspace");
        expect(command.icon).toBe("pulse");
    });

    it("should have description", () => {
        expect(command.description).toBeDefined();
        expect(command.description).toContain("Analyze");
    });

    it("should execute analyze command successfully", async () => {
        const runVtcodeCommandSpy = vi
            .spyOn(vtcodeRunner, "runVtcodeCommand")
            .mockResolvedValue(undefined);
        const showInfoSpy = vi
            .spyOn(vscode.window, "showInformationMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(runVtcodeCommandSpy).toHaveBeenCalledWith(
            ["analyze"],
            expect.objectContaining({
                title: "Analyzing workspace with VTCode…",
                output: mockOutput,
            })
        );
        expect(showInfoSpy).toHaveBeenCalledWith(
            expect.stringContaining("finished analyzing")
        );
    });

    it("should handle errors during command execution", async () => {
        const error = new Error("CLI not found");
        vi.spyOn(vtcodeRunner, "runVtcodeCommand").mockRejectedValue(error);
        const showErrorSpy = vi
            .spyOn(vscode.window, "showErrorMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showErrorSpy).toHaveBeenCalledWith(
            expect.stringContaining("Failed to analyze")
        );
        expect(showErrorSpy).toHaveBeenCalledWith(
            expect.stringContaining("CLI not found")
        );
    });

    it("should handle non-Error exceptions", async () => {
        vi.spyOn(vtcodeRunner, "runVtcodeCommand").mockRejectedValue(
            "string error"
        );
        const showErrorSpy = vi
            .spyOn(vscode.window, "showErrorMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showErrorSpy).toHaveBeenCalledWith(
            expect.stringContaining("string error")
        );
    });

    it("should pass output channel to vtcode runner", async () => {
        const runVtcodeCommandSpy = vi
            .spyOn(vtcodeRunner, "runVtcodeCommand")
            .mockResolvedValue(undefined);
        vi.spyOn(vscode.window, "showInformationMessage").mockResolvedValue(
            undefined
        );

        await command.execute(mockContext);

        const callArgs = runVtcodeCommandSpy.mock.calls[0];
        expect(callArgs[1]).toMatchObject({
            output: mockOutput,
        });
    });

    it("should include title in runner options", async () => {
        const runVtcodeCommandSpy = vi
            .spyOn(vtcodeRunner, "runVtcodeCommand")
            .mockResolvedValue(undefined);
        vi.spyOn(vscode.window, "showInformationMessage").mockResolvedValue(
            undefined
        );

        await command.execute(mockContext);

        const callArgs = runVtcodeCommandSpy.mock.calls[0];
        expect(callArgs[1]).toHaveProperty("title");
        expect(callArgs[1].title).toContain("Analyzing");
    });

    it("should be able to execute in trusted context", () => {
        const trustedContext: CommandContext = {
            ...mockContext,
        };
        // Mock workspace as trusted
        vi.spyOn(vscode.workspace, "isTrusted", "get").mockReturnValue(true);

        expect(command.canExecute(trustedContext)).toBe(true);
    });
});
