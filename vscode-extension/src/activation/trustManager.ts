/**
 * Trust Manager Module
 *
 * Handles workspace trust state management and trust-related event listeners
 */

import * as vscode from "vscode";

export interface TrustManagerOptions {
    context: vscode.ExtensionContext;
    outputChannel: vscode.OutputChannel;
    onTrustChanged: (trusted: boolean) => void;
}

/**
 * Sets up workspace trust listeners and initializes trust state
 */
export function setupTrustManager(options: TrustManagerOptions): void {
    const { context, outputChannel, onTrustChanged } = options;

    const initialTrust = vscode.workspace.isTrusted;
    outputChannel.appendLine(
        `[info] Initial workspace trust state: ${initialTrust ? "trusted" : "not trusted"}`
    );

    // Set initial trust state
    onTrustChanged(initialTrust);

    // Listen for trust changes
    context.subscriptions.push(
        vscode.workspace.onDidGrantWorkspaceTrust(() => {
            outputChannel.appendLine(
                "[info] Workspace trust granted"
            );
            onTrustChanged(true);
        })
    );

    outputChannel.appendLine(
        "[info] Trust manager initialized successfully"
    );
}

/**
 * Ensures workspace is trusted before executing a command
 *
 * @param commandName Human-readable name of the command for error messages
 * @returns true if workspace is trusted, false otherwise
 */
export async function ensureWorkspaceTrusted(
    commandName: string
): Promise<boolean> {
    if (!vscode.workspace.isTrusted) {
        const selection = await vscode.window.showWarningMessage(
            `Cannot ${commandName}: workspace is not trusted. Would you like to manage trust settings?`,
            "Manage Trust",
            "Cancel"
        );

        if (selection === "Manage Trust") {
            await vscode.commands.executeCommand(
                "workbench.action.manageTrust"
            );
        }

        return false;
    }

    return true;
}
