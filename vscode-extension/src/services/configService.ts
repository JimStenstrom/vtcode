import * as vscode from "vscode";
import { VtcodeConfigSummary } from "../vtcodeConfig";
import { state } from "../core/stateManager";
import { updateStatusBarItem } from "../core/statusBar";
import { getConfiguredCommandPath, getOutputChannel } from "../utils/processUtils";
import { FULL_AUTO_WARNING_STATE_KEY } from "../types/extensionTypes";

/**
 * Get workspace storage key
 */
function getWorkspaceStorageKey(base: string): string {
    const workspaceId =
        vscode.workspace.workspaceFolders?.[0]?.uri.toString() ?? "global";
    return `${base}:${workspaceId}`;
}

/**
 * Maybe show warning about full-auto mode
 */
async function maybeWarnAboutFullAuto(
    summary: VtcodeConfigSummary
): Promise<void> {
    if (!state.activationContext || summary.automationFullAutoEnabled !== true) {
        return;
    }

    const storageKey = getWorkspaceStorageKey(FULL_AUTO_WARNING_STATE_KEY);
    if (state.activationContext.globalState.get<boolean>(storageKey)) {
        return;
    }

    await state.activationContext.globalState.update(storageKey, true);
    const selection = await vscode.window.showWarningMessage(
        "VTCode full-auto mode is blocked in VS Code. The extension will continue to require manual oversight before executing tools.",
        "Open vtcode.toml",
        "Dismiss"
    );

    if (selection === "Open vtcode.toml") {
        await vscode.commands.executeCommand("vtcode.openConfig");
    }
}

/**
 * Maybe warn about provider/model mismatch
 */
function maybeWarnAboutProviderModel(summary: VtcodeConfigSummary): void {
    const provider = summary.agentProvider?.trim();
    const defaultModel = summary.agentDefaultModel?.trim();

    if (!provider || !defaultModel) {
        state.lastProviderModelWarningKey = undefined;
        return;
    }

    const key = `${provider}|${defaultModel}`;
    const providerLower = provider.toLowerCase();
    const modelLower = defaultModel.toLowerCase();

    let mismatch = false;
    if (providerLower === "ollama") {
        mismatch = defaultModel.includes(":");
    } else if (providerLower === "openrouter") {
        mismatch = false;
    } else if (modelLower.startsWith("gpt-oss:")) {
        mismatch = true;
    }

    if (!mismatch) {
        state.lastProviderModelWarningKey = undefined;
        return;
    }

    if (state.lastProviderModelWarningKey === key) {
        return;
    }

    state.lastProviderModelWarningKey = key;
    const channel = getOutputChannel();
    channel.appendLine(
        `[warn] VTCode config mismatch: provider "${provider}" may not support default_model "${defaultModel}". Update vtcode.toml or configure the required API credentials to avoid CLI failures.`
    );
}

/**
 * Handle configuration update
 */
export function handleConfigUpdate(summary: VtcodeConfigSummary): void {
    state.currentConfigSummary = summary;
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    state.chatBackend?.updateConfiguration(
        getConfiguredCommandPath(),
        workspaceRoot,
        summary
    );

    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.configAvailable",
        summary.hasConfig
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.hitlEnabled",
        summary.humanInTheLoop === true
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.toolPoliciesConfigured",
        (summary.toolPoliciesCount ?? 0) > 0
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.mcpConfigured",
        summary.mcpProviders.length > 0
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.mcpEnabled",
        summary.mcpEnabled === true
    );
    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.fullAutoEnabled",
        summary.automationFullAutoEnabled === true
    );

    const configUriString = summary.uri?.toString();
    if (summary.parseError && summary.parseError !== state.lastConfigParseError) {
        const channel = getOutputChannel();
        channel.appendLine(
            `[warn] Failed to parse vtcode.toml: ${summary.parseError}`
        );
    } else if (
        !summary.parseError &&
        configUriString &&
        configUriString !== state.lastConfigUri
    ) {
        const channel = getOutputChannel();
        const label = summary.uri
            ? vscode.workspace.asRelativePath(summary.uri, false)
            : "vtcode.toml";
        channel.appendLine(`[info] Using VTCode configuration from ${label}.`);
    }

    if (summary.hasConfig) {
        const enabled = summary.automationFullAutoEnabled === true;
        if (enabled && state.lastAutomationFullAutoEnabled !== true) {
            const channel = getOutputChannel();
            channel.appendLine(
                "[warn] VTCode extension: Full-auto mode is blocked inside VS Code. Human-in-the-loop approval remains required."
            );
            void maybeWarnAboutFullAuto(summary);
        } else if (!enabled && state.lastAutomationFullAutoEnabled !== false) {
            const channel = getOutputChannel();
            channel.appendLine(
                "[info] automation.full_auto is disabled; VTCode will rely on CLI safeguards for prompt execution."
            );
        }
        state.lastAutomationFullAutoEnabled = enabled;
        maybeWarnAboutProviderModel(summary);
    } else {
        state.lastAutomationFullAutoEnabled = undefined;
        state.lastProviderModelWarningKey = undefined;
    }

    state.chatViewProvider?.updateConfig(summary);

    state.lastConfigUri = configUriString;
    state.lastConfigParseError = summary.parseError;

    updateStatusBarItem(getConfiguredCommandPath(), state.cliAvailable);
    state.quickActionsProvider?.refresh();
    state.workspaceInsightsProvider?.refresh();
    state.fireMcpDefinitionsChanged();
}
