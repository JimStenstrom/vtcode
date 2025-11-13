/**
 * Configuration Watcher Module
 *
 * Handles registration and management of vtcode.toml configuration file watchers
 */

import * as vscode from "vscode";
import {
    registerVtcodeConfigWatcher,
    VtcodeConfigSummary,
} from "../vtcodeConfig";

export interface ConfigWatcherOptions {
    context: vscode.ExtensionContext;
    outputChannel: vscode.OutputChannel;
    onConfigUpdate: (summary: VtcodeConfigSummary) => void;
}

/**
 * Registers configuration file watcher
 *
 * Sets up a file system watcher that monitors vtcode.toml files
 * and triggers updates when configuration changes
 */
export function setupConfigWatcher(options: ConfigWatcherOptions): void {
    const { context, outputChannel, onConfigUpdate } = options;

    outputChannel.appendLine("[info] Setting up configuration watcher...");

    const disposable = registerVtcodeConfigWatcher((summary) => {
        outputChannel.appendLine(
            `[info] Configuration file changed: ${summary.uri?.toString() ?? "unknown"}`
        );
        onConfigUpdate(summary);
    });

    context.subscriptions.push(disposable);

    outputChannel.appendLine(
        "[info] Configuration watcher registered successfully"
    );
}
