import * as vscode from "vscode";
import { getOutputChannel } from "./processUtils";

/**
 * Ensure stable API (log warnings for proposed APIs)
 */
export function ensureStableApi(context: vscode.ExtensionContext): void {
    const manifest = context.extension.packageJSON as
        | { enabledApiProposals?: string[] }
        | undefined;
    const proposals = manifest?.enabledApiProposals ?? [];

    if (proposals.length > 0) {
        const channel = getOutputChannel();
        channel.appendLine(
            `[warn] Proposed VS Code APIs enabled: ${proposals.join(", ")}.`
        );
    }
}

/**
 * Log extension host context information
 */
export function logExtensionHostContext(context: vscode.ExtensionContext): void {
    const channel = getOutputChannel();
    const remoteName = vscode.env.remoteName
        ? `remote (${vscode.env.remoteName})`
        : "local";
    const hostKind =
        vscode.env.uiKind === vscode.UIKind.Web ? "web" : "desktop";
    const modeLabel = getExtensionModeLabel(context.extensionMode);
    channel.appendLine(
        `[info] VTCode Companion activated in ${remoteName} ${hostKind} host (${modeLabel} mode).`
    );
}

/**
 * Get extension mode label
 */
function getExtensionModeLabel(mode: vscode.ExtensionMode): string {
    switch (mode) {
        case vscode.ExtensionMode.Development:
            return "development";
        case vscode.ExtensionMode.Test:
            return "test";
        case vscode.ExtensionMode.Production:
        default:
            return "production";
    }
}

/**
 * Open tools policy guide
 */
export async function openToolsPolicyGuide(): Promise<void> {
    const [guide] = await vscode.workspace.findFiles(
        "docs/vtcode_tools_policy.md",
        "**/{node_modules,dist,out,.git,target}/**",
        1
    );
    if (guide) {
        const document = await vscode.workspace.openTextDocument(guide);
        await vscode.window.showTextDocument(document, { preview: false });
        return;
    }

    await vscode.env.openExternal(
        vscode.Uri.parse(
            "https://github.com/vinhnx/vtcode/blob/main/docs/vtcode_tools_policy.md"
        )
    );
}

/**
 * Open MCP integration guide
 */
export async function openMcpGuide(): Promise<void> {
    const [guide] = await vscode.workspace.findFiles(
        "docs/guides/mcp-integration.md",
        "**/{node_modules,dist,out,.git,target}/**",
        1
    );
    if (guide) {
        const document = await vscode.workspace.openTextDocument(guide);
        await vscode.window.showTextDocument(document, { preview: false });
        return;
    }

    await vscode.env.openExternal(
        vscode.Uri.parse(
            "https://github.com/vinhnx/vtcode/blob/main/docs/guides/mcp-integration.md"
        )
    );
}
