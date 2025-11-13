import { spawn } from "node:child_process";
import * as vscode from "vscode";
import { CLI_DETECTION_TIMEOUT_MS } from "../types/extensionTypes";
import { state } from "./stateManager";
import { createSpawnOptions, getWorkspaceRoot, getConfiguredCommandPath } from "../utils/processUtils";

/**
 * Detect if the VTCode CLI is available
 */
export async function detectCliAvailability(commandPath: string): Promise<boolean> {
    if (!commandPath) {
        return false;
    }

    const cwd = getWorkspaceRoot();
    const spawnOptions = cwd
        ? createSpawnOptions({ cwd })
        : createSpawnOptions();

    return new Promise((resolve) => {
        let resolved = false;

        const complete = (value: boolean) => {
            if (!resolved) {
                resolved = true;
                resolve(value);
            }
        };

        try {
            const child = spawn(commandPath, ["--version"], spawnOptions);

            const timer = setTimeout(() => {
                child.kill();
                complete(false);
            }, CLI_DETECTION_TIMEOUT_MS);

            child.on("error", () => {
                clearTimeout(timer);
                complete(false);
            });

            child.on("close", (code) => {
                clearTimeout(timer);
                complete(code === 0);
            });
        } catch {
            complete(false);
        }
    });
}

/**
 * Update CLI availability state and trigger UI updates
 */
export function updateCliAvailabilityState(
    available: boolean,
    commandPath: string,
    reason?: "untrusted"
): void {
    const workspaceTrusted = state.workspaceTrusted;
    const normalizedAvailable = available && workspaceTrusted;
    const previous = state.cliAvailable;
    state.cliAvailable = normalizedAvailable;

    void vscode.commands.executeCommand(
        "setContext",
        "vtcode.cliAvailable",
        normalizedAvailable && workspaceTrusted
    );

    if (normalizedAvailable) {
        state.missingCliWarningShown = false;
    }

    if (previous !== normalizedAvailable) {
        const channel = state.outputChannel;
        if (!channel) {
            return;
        }

        if (!workspaceTrusted && reason === "untrusted") {
            channel.appendLine(
                "[info] VTCode CLI checks are paused until the workspace is trusted."
            );
        } else if (normalizedAvailable) {
            channel.appendLine(
                `[info] Detected VTCode CLI using "${commandPath}".`
            );
        } else {
            channel.appendLine(
                `[warn] VTCode CLI not found using "${commandPath}".`
            );
        }
    }

    state.quickActionsProvider?.refresh();
    state.workspaceInsightsProvider?.refresh();
}

/**
 * Refresh CLI availability detection
 */
export async function refreshCliAvailability(
    trigger: "activation" | "configuration" | "manual"
): Promise<void> {
    if (state.cliAvailabilityCheck) {
        await state.cliAvailabilityCheck;
        return;
    }

    const commandPath = getConfiguredCommandPath();
    const workspaceTrusted = state.workspaceTrusted;

    if (!workspaceTrusted) {
        updateCliAvailabilityState(false, commandPath, "untrusted");
        return;
    }

    state.cliAvailabilityCheck = (async () => {
        if (vscode.env.uiKind === vscode.UIKind.Web) {
            updateCliAvailabilityState(false, commandPath);
            return;
        }

        const available = await detectCliAvailability(commandPath);
        updateCliAvailabilityState(available, commandPath);

        if (!available && trigger === "activation" && workspaceTrusted) {
            await maybeShowMissingCliWarning(commandPath);
        }
    })();

    try {
        await state.cliAvailabilityCheck;
    } finally {
        state.cliAvailabilityCheck = undefined;
    }
}

/**
 * Show missing CLI warning if not already shown
 */
async function maybeShowMissingCliWarning(commandPath: string): Promise<void> {
    if (state.missingCliWarningShown || vscode.env.uiKind === vscode.UIKind.Web) {
        return;
    }

    state.missingCliWarningShown = true;
    const selection = await vscode.window.showWarningMessage(
        `VTCode CLI was not found on PATH as "${commandPath}".`,
        "Open Installation Guide"
    );

    if (selection === "Open Installation Guide") {
        await vscode.commands.executeCommand("vtcode.openInstallGuide");
    }
}

/**
 * Ensure CLI is available before executing a command
 */
export async function ensureCliAvailableForCommand(): Promise<boolean> {
    await refreshCliAvailability("manual");

    if (state.cliAvailable) {
        return true;
    }

    const commandPath = getConfiguredCommandPath();
    const selection = await vscode.window.showWarningMessage(
        `The VTCode CLI ("${commandPath}") is not available. Install the CLI or update the "vtcode.commandPath" setting to run this command.`,
        "Open Installation Guide"
    );

    if (selection === "Open Installation Guide") {
        await vscode.commands.executeCommand("vtcode.openInstallGuide");
    }

    return false;
}
