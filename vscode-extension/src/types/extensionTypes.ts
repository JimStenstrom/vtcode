import * as vscode from "vscode";
import { VtcodeConfigSummary } from "../vtcodeConfig";

/**
 * Shared types and interfaces for the VTCode extension
 */

export interface QuickActionDescription {
    readonly label: string;
    readonly description: string;
    readonly command: string;
    readonly icon?: string;
    readonly args?: unknown[];
}

export interface WorkspaceInsightDescription {
    readonly label: string;
    readonly description: string;
    readonly icon: string;
    readonly command?: vscode.Command;
    readonly tooltip?: string | vscode.MarkdownString;
}

export interface RunVtcodeCommandOptions {
    readonly title?: string;
    readonly revealOutput?: boolean;
    readonly showProgress?: boolean;
    readonly onStdout?: (text: string) => void;
    readonly onStderr?: (text: string) => void;
    readonly cancellationToken?: vscode.CancellationToken;
}

export interface UpdatePlanToolInput {
    readonly summary?: string;
    readonly steps?: string[];
}

export interface VtcodeTaskDefinition extends vscode.TaskDefinition {
    type: "vtcode";
    command: "update-plan";
    summary?: string;
    steps?: string[];
    label?: string;
}

export interface AppendIdeContextOptions {
    readonly includeActiveEditor?: boolean;
    readonly includeVisibleEditors?: boolean;
    readonly chatRequest?: vscode.ChatRequest;
    readonly cancellationToken?: vscode.CancellationToken;
}

export interface DocumentContext {
    readonly text: string;
    readonly range: vscode.Range;
    readonly truncated: boolean;
}

/**
 * Global extension state
 */
export interface ExtensionState {
    outputChannel?: vscode.OutputChannel;
    statusBarItem?: vscode.StatusBarItem;
    agentTerminal?: vscode.Terminal;
    terminalCloseListener?: vscode.Disposable;
    cliAvailable: boolean;
    missingCliWarningShown: boolean;
    cliAvailabilityCheck?: Promise<void>;
    currentConfigSummary?: VtcodeConfigSummary;
    lastConfigUri?: string;
    lastConfigParseError?: string;
    lastAutomationFullAutoEnabled?: boolean;
    workspaceTrusted: boolean;
    lastProviderModelWarningKey?: string;
    chatLaunchHintShown: boolean;
    ideContextBridge?: IIdeContextBridge;
}

export interface IIdeContextBridge extends vscode.Disposable {
    scheduleRefresh(delay?: number): void;
    flush(): Promise<void>;
    readonly filePath: string | undefined;
}

/**
 * Constants
 */
export const CLI_DETECTION_TIMEOUT_MS = 4000;
export const VT_CODE_CHAT_PARTICIPANT_ID = "vtcode.agent";
export const VT_CODE_UPDATE_PLAN_TOOL = "vtcode-updatePlan";
export const VT_CODE_MCP_PROVIDER_ID = "vtcode.workspaceMcp";
export const TRUST_PROMPT_STATE_KEY = "vtcode.trustPromptShown";
export const FULL_AUTO_WARNING_STATE_KEY = "vtcode.fullAutoWarningShown";
export const IDE_CONTEXT_ENV_VARIABLE = "VT_VSCODE_CONTEXT_FILE";
export const IDE_CONTEXT_HEADER = "## VS Code Context";
export const MAX_IDE_CONTEXT_CHARS = 6000;
export const MAX_FULL_DOCUMENT_CONTEXT_LINES = 400;
export const ACTIVE_EDITOR_CONTEXT_WINDOW = 80;
export const MAX_VISIBLE_EDITOR_CONTEXTS = 3;
