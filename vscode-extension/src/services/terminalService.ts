import * as vscode from "vscode";
import { state } from "../core/stateManager";
import { getVtcodeEnvironment, formatArgsForShell, getConfiguredCommandPath } from "../utils/processUtils";
import { VtcodeConfigSummary } from "../vtcodeConfig";

/**
 * Get config arguments for CLI commands
 */
function getConfigArguments(): string[] {
    const uri = state.currentConfigSummary?.uri;
    if (!uri) {
        return [];
    }

    return ["--config", uri.fsPath];
}

/**
 * Ensure agent terminal exists, creating it if necessary
 */
export function ensureAgentTerminal(
    commandPath: string,
    cwd: string
): { terminal: vscode.Terminal; created: boolean } {
    if (state.agentTerminal) {
        return { terminal: state.agentTerminal, created: false };
    }

    const ideContextPath = state.ideContextBridge?.filePath;
    const terminal = vscode.window.createTerminal({
        name: "VTCode Agent",
        cwd,
        env: getVtcodeEnvironment({}, ideContextPath),
        iconPath: new vscode.ThemeIcon("comment-discussion"),
    });

    // Use a delay to allow any auto-activation to complete
    setTimeout(() => {
        void (async () => {
            if (state.ideContextBridge) {
                await state.ideContextBridge.flush();
            }
            const quotedCommandPath = /\s/.test(commandPath)
                ? `"${commandPath.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`
                : commandPath;
            const configArgs = getConfigArguments();
            const terminalArgs = ["chat", ...configArgs];
            const argsText = formatArgsForShell(terminalArgs);
            const commandText =
                argsText.length > 0
                    ? `${quotedCommandPath} ${argsText}`
                    : quotedCommandPath;
            terminal.sendText(commandText, true);
        })();
    }, 800); // 800ms delay to allow environment activation

    state.agentTerminal = terminal;

    if (!state.terminalCloseListener) {
        state.terminalCloseListener = vscode.window.onDidCloseTerminal((closed) => {
            if (closed === state.agentTerminal) {
                state.agentTerminal = undefined;
                state.terminalCloseListener?.dispose();
                state.terminalCloseListener = undefined;
            }
        });
    }

    return { terminal, created: true };
}

/**
 * Launch agent terminal command
 */
export async function launchAgentTerminal(cwd: string | undefined): Promise<void> {
    if (!cwd) {
        void vscode.window.showWarningMessage(
            "Open a workspace folder before launching the VTCode agent terminal."
        );
        return;
    }

    const commandPath = getConfiguredCommandPath();
    const { terminal, created } = ensureAgentTerminal(commandPath, cwd);
    terminal.show(true);
    if (created) {
        const channel = state.outputChannel;
        channel?.appendLine(
            `[info] Launching VTCode agent terminal with "${commandPath} chat" in ${cwd}.`
        );
    }
}
