import { type SpawnOptionsWithoutStdio } from "node:child_process";
import * as vscode from "vscode";
import { IDE_CONTEXT_ENV_VARIABLE } from "../types/extensionTypes";

/**
 * Get the configured VTCode command path from settings
 */
export function getConfiguredCommandPath(): string {
    return (
        vscode.workspace
            .getConfiguration("vtcode")
            .get<string>("commandPath", "vtcode")
            .trim() || "vtcode"
    );
}

/**
 * Get the workspace root directory
 */
export function getWorkspaceRoot(): string | undefined {
    const activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(
            activeEditor.document.uri
        );
        if (workspaceFolder) {
            return workspaceFolder.uri.fsPath;
        }
    }

    const [firstWorkspace] = vscode.workspace.workspaceFolders ?? [];
    return firstWorkspace?.uri.fsPath;
}

/**
 * Get the primary workspace folder
 */
export function getPrimaryWorkspaceFolder(): vscode.WorkspaceFolder | undefined {
    const activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const folder = vscode.workspace.getWorkspaceFolder(
            activeEditor.document.uri
        );
        if (folder) {
            return folder;
        }
    }

    const [firstWorkspace] = vscode.workspace.workspaceFolders ?? [];
    return firstWorkspace;
}

/**
 * Create environment variables for VTCode processes
 */
export function getVtcodeEnvironment(
    overrides: NodeJS.ProcessEnv = {},
    ideContextPath?: string
): Record<string, string> {
    const merged = { ...process.env, ...overrides };
    const env: Record<string, string> = {};
    for (const [key, value] of Object.entries(merged)) {
        if (typeof value === "string") {
            env[key] = value;
        }
    }

    if (ideContextPath) {
        env[IDE_CONTEXT_ENV_VARIABLE] = ideContextPath;
    } else {
        delete env[IDE_CONTEXT_ENV_VARIABLE];
    }

    return env;
}

/**
 * Create spawn options with VTCode environment
 */
export function createSpawnOptions(
    overrides: Partial<SpawnOptionsWithoutStdio> = {},
    ideContextPath?: string
): SpawnOptionsWithoutStdio {
    const { env: overrideEnv, ...rest } = overrides;
    return {
        env: getVtcodeEnvironment(overrideEnv ?? {}, ideContextPath),
        ...rest,
    };
}

/**
 * Format arguments for logging (with proper quoting)
 */
export function formatArgsForLogging(args: string[]): string {
    return args
        .map((arg) => {
            const value = String(arg);
            return /(\s|"|')/.test(value) ? JSON.stringify(value) : value;
        })
        .join(" ");
}

/**
 * Format arguments for shell execution (with proper escaping)
 */
export function formatArgsForShell(args: string[]): string {
    return args
        .map((arg) => {
            const value = String(arg);
            return quoteForShell(value);
        })
        .filter((value) => value.length > 0)
        .join(" ");
}

/**
 * Quote a value for shell execution if needed
 */
function quoteForShell(value: string): string {
    if (!/[\s"'\\$`]/.test(value)) {
        return value;
    }

    return `"${value.replace(/(["\\$`])/g, "\\$1")}"`;
}

/**
 * Handle command errors with user-friendly messages
 */
export function handleCommandError(contextLabel: string, error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(
        `Failed to ${contextLabel} with VTCode: ${message}`
    );
}

/**
 * Get or create the output channel
 */
let outputChannelInstance: vscode.OutputChannel | undefined;

export function getOutputChannel(): vscode.OutputChannel {
    if (!outputChannelInstance) {
        outputChannelInstance = vscode.window.createOutputChannel("VTCode");
    }
    return outputChannelInstance;
}

export function setOutputChannel(channel: vscode.OutputChannel | undefined): void {
    outputChannelInstance = channel;
}
