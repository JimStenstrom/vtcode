/**
 * Configuration Limits Helper
 *
 * Centralizes access to configurable limits and timeouts to avoid hardcoded values
 * throughout the codebase. All limits can be configured via VS Code settings.
 */

import * as vscode from "vscode";

export class ConfigLimits {
    private static getConfig(): vscode.WorkspaceConfiguration {
        return vscode.workspace.getConfiguration("vtcode");
    }

    /**
     * File and Document Limits
     */
    static get maxActiveFileLines(): number {
        return this.getConfig().get<number>("limits.maxActiveFileLines", 1000);
    }

    static get maxFullDocumentLines(): number {
        return this.getConfig().get<number>("limits.maxFullDocumentLines", 400);
    }

    static get activeEditorContextWindow(): number {
        return this.getConfig().get<number>("limits.activeEditorContextWindow", 80);
    }

    static get maxVisibleEditorContexts(): number {
        return this.getConfig().get<number>("limits.maxVisibleEditorContexts", 3);
    }

    static get maxIdeContextChars(): number {
        return this.getConfig().get<number>("limits.maxIdeContextChars", 6000);
    }

    /**
     * Conversation Limits
     */
    static get maxConversationMessages(): number {
        return this.getConfig().get<number>("limits.maxConversationMessages", 12);
    }

    static get toolApprovalDetailMaxChars(): number {
        return this.getConfig().get<number>("limits.toolApprovalDetailMaxChars", 1200);
    }

    static get conversationContextMaxChars(): number {
        return this.getConfig().get<number>("limits.conversationContextMaxChars", 2000);
    }

    /**
     * Participant Limits
     */
    static get codeParticipantMaxLines(): number {
        return this.getConfig().get<number>("limits.codeParticipantMaxLines", 50);
    }

    static get terminalOutputLines(): number {
        return this.getConfig().get<number>("limits.terminalOutputLines", 20);
    }

    static get terminalHistoryCommands(): number {
        return this.getConfig().get<number>("limits.terminalHistoryCommands", 5);
    }

    static get workspaceMaxFiles(): number {
        return this.getConfig().get<number>("limits.workspaceMaxFiles", 100);
    }

    /**
     * Timeout Settings
     */
    static get cliDetectionTimeoutMs(): number {
        return this.getConfig().get<number>("timeouts.cliDetectionMs", 4000);
    }

    static get mcpDiscoveryTimeoutMs(): number {
        return this.getConfig().get<number>("timeouts.mcpDiscoveryMs", 5000);
    }

    static get mcpExecutionTimeoutMs(): number {
        return this.getConfig().get<number>("timeouts.mcpExecutionMs", 30000);
    }

    static get ptyCommandTimeoutMs(): number {
        return this.getConfig().get<number>("timeouts.ptyCommandMs", 30000);
    }

    /**
     * Trajectory View Limits
     */
    static get trajectoryMaxLogLines(): number {
        return this.getConfig().get<number>("limits.trajectoryMaxLogLines", 2000);
    }

    static get trajectoryMaxTurns(): number {
        return this.getConfig().get<number>("limits.trajectoryMaxTurns", 50);
    }

    /**
     * Warning Settings
     */
    static get showLimitExceededWarnings(): boolean {
        return this.getConfig().get<boolean>("warnings.showLimitExceededWarnings", true);
    }

    /**
     * Graceful Degradation
     */
    static get enableWithoutTrust(): boolean {
        return this.getConfig().get<boolean>("gracefulDegradation.enableWithoutTrust", false);
    }

    /**
     * Show a warning if enabled
     */
    static showWarningIfEnabled(message: string): void {
        if (this.showLimitExceededWarnings) {
            vscode.window.showWarningMessage(`VTCode: ${message}`);
        }
    }

    /**
     * Show an info message if warnings are enabled
     */
    static showInfoIfEnabled(message: string): void {
        if (this.showLimitExceededWarnings) {
            vscode.window.showInformationMessage(`VTCode: ${message}`);
        }
    }
}
