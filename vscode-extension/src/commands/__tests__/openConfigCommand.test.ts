import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { OpenConfigCommand } from "../openConfigCommand";
import { CommandContext } from "../../types/command";
import * as vscode from "vscode";
import * as vtcodeConfig from "../../vtcodeConfig";

vi.mock("../../vtcodeConfig");

describe("OpenConfigCommand", () => {
    let command: OpenConfigCommand;
    let mockContext: CommandContext;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        command = new OpenConfigCommand();
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
        expect(command.id).toBe("vtcode.openConfig");
        expect(command.title).toBe("Open Configuration");
        expect(command.icon).toBe("gear");
    });

    it("should have description", () => {
        expect(command.description).toBeDefined();
        expect(command.description).toContain("vtcode.toml");
    });

    it("should open config file when found", async () => {
        const mockUri = vscode.Uri.file("/workspace/vtcode.toml");
        const mockDocument = { uri: mockUri } as vscode.TextDocument;

        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            mockUri
        );
        const openTextDocumentSpy = vi
            .spyOn(vscode.workspace, "openTextDocument")
            .mockResolvedValue(mockDocument);
        const showTextDocumentSpy = vi
            .spyOn(vscode.window, "showTextDocument")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(openTextDocumentSpy).toHaveBeenCalledWith(mockUri);
        expect(showTextDocumentSpy).toHaveBeenCalledWith(mockDocument, {
            preview: false,
        });
    });

    it("should show warning when no config file found", async () => {
        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            undefined
        );
        const showWarningSpy = vi
            .spyOn(vscode.window, "showWarningMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showWarningSpy).toHaveBeenCalledWith(
            expect.stringContaining("No vtcode.toml")
        );
    });

    it("should handle errors when opening config", async () => {
        const mockUri = vscode.Uri.file("/workspace/vtcode.toml");
        const error = new Error("File not found");

        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            mockUri
        );
        vi.spyOn(vscode.workspace, "openTextDocument").mockRejectedValue(
            error
        );
        const showErrorSpy = vi
            .spyOn(vscode.window, "showErrorMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showErrorSpy).toHaveBeenCalledWith(
            expect.stringContaining("Failed to open configuration")
        );
        expect(showErrorSpy).toHaveBeenCalledWith(
            expect.stringContaining("File not found")
        );
    });

    it("should not call showTextDocument if config uri is undefined", async () => {
        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            undefined
        );
        vi.spyOn(vscode.window, "showWarningMessage").mockResolvedValue(
            undefined
        );
        const showTextDocumentSpy = vi.spyOn(
            vscode.window,
            "showTextDocument"
        );

        await command.execute(mockContext);

        expect(showTextDocumentSpy).not.toHaveBeenCalled();
    });

    it("should open document without preview", async () => {
        const mockUri = vscode.Uri.file("/workspace/vtcode.toml");
        const mockDocument = { uri: mockUri } as vscode.TextDocument;

        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            mockUri
        );
        vi.spyOn(vscode.workspace, "openTextDocument").mockResolvedValue(
            mockDocument
        );
        const showTextDocumentSpy = vi
            .spyOn(vscode.window, "showTextDocument")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(showTextDocumentSpy).toHaveBeenCalledWith(
            mockDocument,
            expect.objectContaining({ preview: false })
        );
    });

    it("should handle non-Error exceptions", async () => {
        const mockUri = vscode.Uri.file("/workspace/vtcode.toml");
        vi.spyOn(vtcodeConfig, "pickVtcodeConfigUri").mockResolvedValue(
            mockUri
        );
        vi.spyOn(vscode.workspace, "openTextDocument").mockRejectedValue(
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
});
