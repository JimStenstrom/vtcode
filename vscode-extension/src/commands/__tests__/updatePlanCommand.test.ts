import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { UpdatePlanCommand } from "../updatePlanCommand";
import { CommandContext } from "../../types/command";
import * as vscode from "vscode";

describe("UpdatePlanCommand", () => {
    let command: UpdatePlanCommand;
    let mockContext: CommandContext;
    let mockOutput: vscode.OutputChannel;

    beforeEach(() => {
        command = new UpdatePlanCommand();
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
        expect(command.id).toBe("vtcode.runUpdatePlanTask");
        expect(command.title).toBe("Update Plan");
        expect(command.icon).toBe("checklist");
    });

    it("should have description", () => {
        expect(command.description).toBeDefined();
        expect(command.description).toContain("plan");
    });

    it("should show warning when no update-plan tasks exist", async () => {
        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([]);
        const showWarningSpy = vi
            .spyOn(vscode.window, "showWarningMessage")
            .mockResolvedValue(undefined);

        await command.execute(mockContext);

        expect(showWarningSpy).toHaveBeenCalledWith(
            expect.stringContaining("No VTCode update plan tasks")
        );
    });

    it("should execute single update-plan task automatically", async () => {
        const mockTask: vscode.Task = {
            name: "Update Plan",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
            execution: undefined,
            isBackground: false,
            presentationOptions: {},
            problemMatchers: [],
        } as any;

        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([mockTask]);
        const executeTaskSpy = vi
            .spyOn(vscode.tasks, "executeTask")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(executeTaskSpy).toHaveBeenCalledWith(mockTask);
    });

    it("should prompt user to select when multiple tasks exist", async () => {
        const mockTask1: vscode.Task = {
            name: "Update Plan 1",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        const mockTask2: vscode.Task = {
            name: "Update Plan 2",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([
            mockTask1,
            mockTask2,
        ]);

        const showQuickPickSpy = vi
            .spyOn(vscode.window, "showQuickPick")
            .mockResolvedValue({ label: "Update Plan 1", task: mockTask1 });

        const executeTaskSpy = vi
            .spyOn(vscode.tasks, "executeTask")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(showQuickPickSpy).toHaveBeenCalledWith(
            expect.arrayContaining([
                expect.objectContaining({ label: "Update Plan 1" }),
                expect.objectContaining({ label: "Update Plan 2" }),
            ]),
            expect.objectContaining({
                placeHolder: expect.stringContaining("Select"),
            })
        );
        expect(executeTaskSpy).toHaveBeenCalledWith(mockTask1);
    });

    it("should not execute task when user cancels selection", async () => {
        const mockTask1: vscode.Task = {
            name: "Update Plan 1",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        const mockTask2: vscode.Task = {
            name: "Update Plan 2",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([
            mockTask1,
            mockTask2,
        ]);

        vi.spyOn(vscode.window, "showQuickPick").mockResolvedValue(undefined);

        const executeTaskSpy = vi
            .spyOn(vscode.tasks, "executeTask")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(executeTaskSpy).not.toHaveBeenCalled();
    });

    it("should filter for update-plan tasks only", async () => {
        const updatePlanTask: vscode.Task = {
            name: "Update Plan",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        const analyzeTask: vscode.Task = {
            name: "Analyze",
            definition: { type: "vtcode", command: "analyze" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        const otherTask: vscode.Task = {
            name: "Other",
            definition: { type: "npm" },
            scope: vscode.TaskScope.Workspace,
            source: "npm",
        } as any;

        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([
            updatePlanTask,
            analyzeTask,
            otherTask,
        ]);

        const executeTaskSpy = vi
            .spyOn(vscode.tasks, "executeTask")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        // Should only execute the update-plan task
        expect(executeTaskSpy).toHaveBeenCalledWith(updatePlanTask);
        expect(executeTaskSpy).toHaveBeenCalledTimes(1);
    });

    it("should fetch vtcode type tasks", async () => {
        const fetchTasksSpy = vi
            .spyOn(vscode.tasks, "fetchTasks")
            .mockResolvedValue([]);
        vi.spyOn(vscode.window, "showWarningMessage").mockResolvedValue(
            undefined
        );

        await command.execute(mockContext);

        expect(fetchTasksSpy).toHaveBeenCalledWith({ type: "vtcode" });
    });

    it("should handle fetchTasks errors gracefully", async () => {
        vi.spyOn(vscode.tasks, "fetchTasks").mockRejectedValue(
            new Error("Failed to fetch tasks")
        );

        // Should propagate the error
        await expect(command.execute(mockContext)).rejects.toThrow(
            "Failed to fetch tasks"
        );
    });

    it("should check CLI availability before execution", async () => {
        const mockTask: vscode.Task = {
            name: "Update Plan",
            definition: { type: "vtcode", command: "update-plan" },
            scope: vscode.TaskScope.Workspace,
            source: "vtcode",
        } as any;

        vi.spyOn(vscode.tasks, "fetchTasks").mockResolvedValue([mockTask]);

        // Mock ensureCliAvailable to return false
        const ensureSpy = vi
            .spyOn(command as any, "ensureCliAvailable")
            .mockReturnValue(false);

        const executeTaskSpy = vi
            .spyOn(vscode.tasks, "executeTask")
            .mockResolvedValue({} as any);

        await command.execute(mockContext);

        expect(ensureSpy).toHaveBeenCalled();
        expect(executeTaskSpy).not.toHaveBeenCalled();
    });
});
