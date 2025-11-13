import * as vscode from "vscode";
import { BaseCommand, type CommandContext } from "../types/command";
import { runVtcodeCommand } from "../utils/vtcodeRunner";
import { ValidationHelpers } from "../utils/validationHelpers";

/**
 * Command to ask VTCode about the currently selected code
 */
export class AskSelectionCommand extends BaseCommand {
    public readonly id = "vtcode.askSelection";
    public readonly title = "Ask About Selection";
    public readonly description = "Ask VTCode to explain or analyze the selected code";
    public readonly icon = "comment";

    async execute(context: CommandContext): Promise<void> {
        // Use validation helpers to reduce duplication
        const validation = ValidationHelpers.validateEditorWithSelection();
        if (!validation) {
            return;
        }

        const { editor, text: selectedText } = validation;

        if (!this.ensureCliAvailable(context)) {
            return;
        }

        const defaultQuestion = "Explain the highlighted selection.";
        const question = await vscode.window.showInputBox({
            prompt: "How should VTCode help with the highlighted selection?",
            value: defaultQuestion,
            valueSelection: [0, defaultQuestion.length],
            ignoreFocusOut: true,
        });

        if (question === undefined) {
            return;
        }

        const trimmedQuestion = question.trim() || defaultQuestion;
        const languageId = editor.document.languageId || "text";
        const selection = editor.selection;
        const rangeLabel = `${selection.start.line + 1}-${selection.end.line + 1}`;
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(
            editor.document.uri
        );
        const relativePath = workspaceFolder
            ? vscode.workspace.asRelativePath(editor.document.uri, false)
            : editor.document.fileName;
        const normalizedSelection = selectedText.replace(/\r\n/g, "\n");
        const prompt = `${trimmedQuestion}\n\nFile: ${relativePath}\nLines: ${rangeLabel}\n\n\`\`\`${languageId}\n${normalizedSelection}\n\`\`\``;

        try {
            await runVtcodeCommand(["ask", prompt], {
                title: "Asking VTCode about the selection…",
                output: context.output,
            });
            void vscode.window.showInformationMessage(
                "VTCode processed the highlighted selection. Review the output channel for the response."
            );
        } catch (error) {
            this.handleError("ask about the selection", error);
        }
    }
}