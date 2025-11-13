import * as vscode from "vscode";
import { state } from "../core/stateManager";
import { TRUST_PROMPT_STATE_KEY } from "../types/extensionTypes";
import { updateCliAvailabilityState } from "../core/cliDetection";
import { getConfiguredCommandPath } from "../utils/processUtils";

type WorkspaceTrustApi = typeof vscode.workspace & {
    requestWorkspaceTrust?: (opts?: {
        message?: string;
        modal?: boolean;
        buttons?: ReadonlyArray<vscode.MessageItem>;
    }) => Thenable<boolean | undefined>;
};

/**
 * Request workspace trust from user
 */
export async function requestWorkspaceTrust(action: string): Promise<boolean> {
    if (state.workspaceTrusted) {
        return true;
    }

    const trustApi = vscode.workspace as WorkspaceTrustApi;
    const requestFn = trustApi.requestWorkspaceTrust;
    if (typeof requestFn === "function") {
        try {
            const granted = await requestFn({
                message: `VTCode requires a trusted workspace to ${action}.`,
                modal: true,
            });
            if (granted) {
                updateWorkspaceTrustState(true);
                return true;
            }
        } catch (error) {
            const channel = state.outputChannel;
            const details =
                error instanceof Error ? error.message : String(error);
            channel?.appendLine(
                `[warn] Workspace trust request failed: ${details}`
            );
        }
    }

    return false;
}

/**
 * Ensure workspace is trusted before executing a command
 */
export async function ensureWorkspaceTrustedForCommand(
    action: string
): Promise<boolean> {
    if (state.workspaceTrusted) {
        return true;
    }

    if (await requestWorkspaceTrust(action)) {
        return true;
    }

    const selection = await vscode.window.showWarningMessage(
        `VTCode requires a trusted workspace to ${action}.`,
        "Manage Workspace Trust"
    );

    if (selection === "Manage Workspace Trust") {
        await vscode.commands.executeCommand("workbench.action.manageTrust");
        const trustedNow = vscode.workspace.isTrusted;
        if (trustedNow) {
            updateWorkspaceTrustState(trustedNow);
            return true;
        }
    }

    return false;
}

/**
 * Prompt for workspace trust on activation
 */
export async function promptForWorkspaceTrustOnActivation(
    context: vscode.ExtensionContext
): Promise<void> {
    if (state.workspaceTrusted || vscode.env.uiKind === vscode.UIKind.Web) {
        return;
    }

    const workspaceId =
        vscode.workspace.workspaceFolders?.[0]?.uri.toString() ?? "global";
    const storageKey = `${TRUST_PROMPT_STATE_KEY}:${workspaceId}`;
    const alreadyPrompted = context.globalState.get<boolean>(storageKey);
    if (alreadyPrompted) {
        return;
    }

    await context.globalState.update(storageKey, true);
    try {
        const granted = await requestWorkspaceTrust(
            "allow VTCode to process prompts with human-in-the-loop safeguards"
        );
        if (!granted && !state.workspaceTrusted) {
            const selection = await vscode.window.showInformationMessage(
                "VTCode requires workspace trust to process prompts. Open workspace trust settings?",
                "Manage Workspace Trust"
            );
            if (selection === "Manage Workspace Trust") {
                await vscode.commands.executeCommand(
                    "workbench.action.manageTrust"
                );
                if (vscode.workspace.isTrusted) {
                    updateWorkspaceTrustState(true);
                }
            }
        }
    } catch (error) {
        const channel = state.outputChannel;
        const details = error instanceof Error ? error.message : String(error);
        channel?.appendLine(
            `[warn] Automatic workspace trust prompt failed: ${details}`
        );
    }
}

/**
 * Update workspace trust state and trigger UI updates
 */
export function updateWorkspaceTrustState(trusted: boolean): void {
    state.workspaceTrusted = trusted;
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.workspaceTrusted",
        trusted
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.workspaceTrustVerified",
        trusted
    );
    state.chatViewProvider?.setWorkspaceTrusted(trusted);

    const commandPath = getConfiguredCommandPath();
    updateCliAvailabilityState(
        state.cliAvailable,
        commandPath,
        trusted ? undefined : "untrusted"
    );
    state.fireMcpDefinitionsChanged();
    state.quickActionsProvider?.refresh();
    state.workspaceInsightsProvider?.refresh();
}

/**
 * Initialize VS Code context keys
 */
export function initializeContextKeys(): void {
    const contextDefaults: Array<[string, boolean]> = [
        ["vtcode.workspaceTrusted", state.workspaceTrusted],
        ["vtcode.cliAvailable", false],
        ["vtcode.configAvailable", false],
        ["vtcode.hitlEnabled", false],
        ["vtcode.toolPoliciesConfigured", false],
        ["vtcode.mcpConfigured", false],
        ["vtcode.mcpEnabled", false],
        ["vtcode.fullAutoEnabled", false],
        ["vtcode.workspaceTrustVerified", state.workspaceTrusted],
    ];

    for (const [key, value] of contextDefaults) {
        void vscode.commands.executeCommand("setContext", key, value);
    }
}
