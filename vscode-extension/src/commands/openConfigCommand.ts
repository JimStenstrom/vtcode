import * as vscode from "vscode";
import { BaseCommand, type CommandContext } from "../types/command";
import { pickVtcodeConfigUri } from "../vtcodeConfig";

/**
 * Command to open the VTCode configuration file
 */
export class OpenConfigCommand extends BaseCommand {
    public readonly id = "vtcode.openConfig";
    public readonly title = "Open Configuration";
    public readonly description = "Open the vtcode.toml configuration file";
    public readonly icon = "gear";

    async execute(_context: CommandContext): Promise<void> {
        try {
            const configUri = await pickVtcodeConfigUri();
            if (!configUri) {
                void vscode.window.showWarningMessage(
                    "No vtcode.toml file was found in this workspace."
                );
                return;
            }

            const document = await vscode.workspace.openTextDocument(
                configUri
            );
            await vscode.window.showTextDocument(document, {
                preview: false,
            });
        } catch (error) {
            this.handleError("open configuration", error);
        }
    }
}