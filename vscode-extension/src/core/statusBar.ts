import * as vscode from "vscode";
import { state } from "./stateManager";

/**
 * Set status bar to checking state
 */
export function setStatusBarChecking(commandPath: string): void {
    const statusBarItem = state.statusBarItem;
    if (!statusBarItem) {
        return;
    }

    if (!state.workspaceTrusted) {
        updateStatusBarItem(commandPath, false);
        return;
    }

    statusBarItem.text = "$(sync~spin) VTCode";
    statusBarItem.tooltip = `Checking availability of "${commandPath}"`;
    statusBarItem.command = undefined;
    statusBarItem.backgroundColor = undefined;
    statusBarItem.color = undefined;
    statusBarItem.show();
}

/**
 * Update status bar item based on CLI availability and trust state
 */
export function updateStatusBarItem(commandPath: string, available: boolean): void {
    const statusBarItem = state.statusBarItem;
    if (!statusBarItem) {
        return;
    }

    const workspaceTrusted = state.workspaceTrusted;

    if (!workspaceTrusted) {
        statusBarItem.text = "$(shield) Trust VTCode Workspace";
        statusBarItem.tooltip = createStatusBarTooltip(
            commandPath,
            available,
            false
        );
        statusBarItem.command = "workbench.action.manageTrust";
        statusBarItem.backgroundColor = new vscode.ThemeColor(
            "statusBarItem.prominentBackground"
        );
        statusBarItem.color = new vscode.ThemeColor(
            "statusBarItem.prominentForeground"
        );
        statusBarItem.show();
        return;
    }

    if (available) {
        const hitlEnabled = state.currentConfigSummary?.humanInTheLoop !== false;
        const suffix = state.currentConfigSummary?.hasConfig
            ? hitlEnabled
                ? " $(person)"
                : " $(run-all)"
            : "";
        statusBarItem.text = `$(chevron-right) VTCode${suffix}`;

        statusBarItem.tooltip = createStatusBarTooltip(commandPath, true, true);
        statusBarItem.command = "vtcode.openQuickActions";
        statusBarItem.backgroundColor = new vscode.ThemeColor(
            "vtcode.statusBarBackground"
        );
        statusBarItem.color = new vscode.ThemeColor(
            "vtcode.statusBarForeground"
        );
    } else {
        statusBarItem.text = "$(warning) VTCode CLI Missing";
        statusBarItem.tooltip = createStatusBarTooltip(
            commandPath,
            false,
            true
        );
        statusBarItem.command = "vtcode.openInstallGuide";
        statusBarItem.backgroundColor = new vscode.ThemeColor(
            "statusBarItem.warningBackground"
        );
        statusBarItem.color = new vscode.ThemeColor(
            "statusBarItem.warningForeground"
        );
    }

    statusBarItem.show();
}

/**
 * Create status bar tooltip
 */
function createStatusBarTooltip(
    commandPath: string,
    available: boolean,
    trusted: boolean
): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString(undefined, true);
    tooltip.appendMarkdown("**VTCode Workspace**\n\n");
    tooltip.appendMarkdown(
        `• Workspace trust: ${trusted ? "Trusted" : "Restricted"}\n`
    );

    tooltip.appendMarkdown("\n**VTCode CLI**\n\n");
    tooltip.appendMarkdown(`• Path: \`${commandPath}\`\n`);
    const cliStatus = !trusted
        ? "Blocked by workspace trust"
        : available
        ? "Available"
        : "Missing";
    tooltip.appendMarkdown(`• Status: ${cliStatus}\n`);

    if (!trusted) {
        tooltip.appendMarkdown(
            "\nGrant workspace trust to enable VTCode CLI automation.\n"
        );
        tooltip.isTrusted = true;
        return tooltip;
    }

    const currentConfigSummary = state.currentConfigSummary;
    if (currentConfigSummary?.hasConfig) {
        tooltip.appendMarkdown("\n**Configuration**\n");
        if (currentConfigSummary.uri) {
            const relative = vscode.workspace.asRelativePath(
                currentConfigSummary.uri,
                false
            );
            tooltip.appendMarkdown(`• File: \`${relative}\`\n`);
        }

        if (currentConfigSummary.humanInTheLoop !== undefined) {
            tooltip.appendMarkdown(
                `• Human-in-the-loop: ${
                    currentConfigSummary.humanInTheLoop ? "Enabled" : "Disabled"
                }\n`
            );
        }

        if (currentConfigSummary.toolDefaultPolicy) {
            tooltip.appendMarkdown(
                `• Default tool policy: \`${currentConfigSummary.toolDefaultPolicy}\`\n`
            );
        }

        const providerCount = currentConfigSummary.mcpProviders.length;
        const enabledCount = currentConfigSummary.mcpProviders.filter(
            (provider) => provider.enabled !== false
        ).length;
        if (providerCount > 0) {
            tooltip.appendMarkdown(
                `• MCP providers: ${enabledCount}/${providerCount} enabled\n`
            );
        } else {
            tooltip.appendMarkdown("• MCP providers: none configured\n");
        }
    } else {
        tooltip.appendMarkdown(
            "\nNo vtcode.toml detected in this workspace.\n"
        );
    }

    tooltip.isTrusted = true;
    return tooltip;
}
