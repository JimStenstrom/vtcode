import * as vscode from "vscode";
import { VtcodeConfigSummary } from "../vtcodeConfig";
import { IIdeContextBridge } from "../types/extensionTypes";
import { VtcodeBackend } from "../vtcodeBackend";
import { ChatViewProvider } from "../chatView";
import { QuickActionTreeDataProvider } from "../providers/quickActionsProvider";
import { WorkspaceInsightsTreeDataProvider } from "../providers/workspaceInsightsProvider";

/**
 * Centralized state management for the VTCode extension
 */
export class StateManager {
    // UI Components
    private _outputChannel?: vscode.OutputChannel;
    private _statusBarItem?: vscode.StatusBarItem;
    private _quickActionsProvider?: QuickActionTreeDataProvider;
    private _workspaceInsightsProvider?: WorkspaceInsightsTreeDataProvider;

    // Terminal state
    private _agentTerminal?: vscode.Terminal;
    private _terminalCloseListener?: vscode.Disposable;

    // CLI state
    private _cliAvailable = false;
    private _missingCliWarningShown = false;
    private _cliAvailabilityCheck?: Promise<void>;

    // Configuration state
    private _currentConfigSummary?: VtcodeConfigSummary;
    private _lastConfigUri?: string;
    private _lastConfigParseError?: string;
    private _lastAutomationFullAutoEnabled?: boolean;
    private _lastProviderModelWarningKey?: string;

    // Trust state
    private _workspaceTrusted = vscode.workspace.isTrusted;

    // Backend and UI instances
    private _chatBackend?: VtcodeBackend;
    private _chatViewProvider?: ChatViewProvider;
    private _activationContext?: vscode.ExtensionContext;
    private _ideContextBridge?: IIdeContextBridge;

    // UI flags
    private _chatLaunchHintShown = false;

    // Event emitters
    private _mcpDefinitionsChanged = new vscode.EventEmitter<void>();
    readonly mcpDefinitionsChanged = this._mcpDefinitionsChanged.event;

    // Output Channel
    get outputChannel(): vscode.OutputChannel | undefined {
        return this._outputChannel;
    }

    set outputChannel(value: vscode.OutputChannel | undefined) {
        this._outputChannel = value;
    }

    // Status Bar Item
    get statusBarItem(): vscode.StatusBarItem | undefined {
        return this._statusBarItem;
    }

    set statusBarItem(value: vscode.StatusBarItem | undefined) {
        this._statusBarItem = value;
    }

    // Quick Actions Provider
    get quickActionsProvider(): QuickActionTreeDataProvider | undefined {
        return this._quickActionsProvider;
    }

    set quickActionsProvider(value: QuickActionTreeDataProvider | undefined) {
        this._quickActionsProvider = value;
    }

    // Workspace Insights Provider
    get workspaceInsightsProvider(): WorkspaceInsightsTreeDataProvider | undefined {
        return this._workspaceInsightsProvider;
    }

    set workspaceInsightsProvider(value: WorkspaceInsightsTreeDataProvider | undefined) {
        this._workspaceInsightsProvider = value;
    }

    // Agent Terminal
    get agentTerminal(): vscode.Terminal | undefined {
        return this._agentTerminal;
    }

    set agentTerminal(value: vscode.Terminal | undefined) {
        this._agentTerminal = value;
    }

    // Terminal Close Listener
    get terminalCloseListener(): vscode.Disposable | undefined {
        return this._terminalCloseListener;
    }

    set terminalCloseListener(value: vscode.Disposable | undefined) {
        this._terminalCloseListener = value;
    }

    // CLI Available
    get cliAvailable(): boolean {
        return this._cliAvailable;
    }

    set cliAvailable(value: boolean) {
        this._cliAvailable = value;
    }

    // Missing CLI Warning Shown
    get missingCliWarningShown(): boolean {
        return this._missingCliWarningShown;
    }

    set missingCliWarningShown(value: boolean) {
        this._missingCliWarningShown = value;
    }

    // CLI Availability Check
    get cliAvailabilityCheck(): Promise<void> | undefined {
        return this._cliAvailabilityCheck;
    }

    set cliAvailabilityCheck(value: Promise<void> | undefined) {
        this._cliAvailabilityCheck = value;
    }

    // Current Config Summary
    get currentConfigSummary(): VtcodeConfigSummary | undefined {
        return this._currentConfigSummary;
    }

    set currentConfigSummary(value: VtcodeConfigSummary | undefined) {
        this._currentConfigSummary = value;
    }

    // Last Config URI
    get lastConfigUri(): string | undefined {
        return this._lastConfigUri;
    }

    set lastConfigUri(value: string | undefined) {
        this._lastConfigUri = value;
    }

    // Last Config Parse Error
    get lastConfigParseError(): string | undefined {
        return this._lastConfigParseError;
    }

    set lastConfigParseError(value: string | undefined) {
        this._lastConfigParseError = value;
    }

    // Last Automation Full Auto Enabled
    get lastAutomationFullAutoEnabled(): boolean | undefined {
        return this._lastAutomationFullAutoEnabled;
    }

    set lastAutomationFullAutoEnabled(value: boolean | undefined) {
        this._lastAutomationFullAutoEnabled = value;
    }

    // Last Provider Model Warning Key
    get lastProviderModelWarningKey(): string | undefined {
        return this._lastProviderModelWarningKey;
    }

    set lastProviderModelWarningKey(value: string | undefined) {
        this._lastProviderModelWarningKey = value;
    }

    // Workspace Trusted
    get workspaceTrusted(): boolean {
        return this._workspaceTrusted;
    }

    set workspaceTrusted(value: boolean) {
        this._workspaceTrusted = value;
    }

    // Chat Backend
    get chatBackend(): VtcodeBackend | undefined {
        return this._chatBackend;
    }

    set chatBackend(value: VtcodeBackend | undefined) {
        this._chatBackend = value;
    }

    // Chat View Provider
    get chatViewProvider(): ChatViewProvider | undefined {
        return this._chatViewProvider;
    }

    set chatViewProvider(value: ChatViewProvider | undefined) {
        this._chatViewProvider = value;
    }

    // Activation Context
    get activationContext(): vscode.ExtensionContext | undefined {
        return this._activationContext;
    }

    set activationContext(value: vscode.ExtensionContext | undefined) {
        this._activationContext = value;
    }

    // IDE Context Bridge
    get ideContextBridge(): IIdeContextBridge | undefined {
        return this._ideContextBridge;
    }

    set ideContextBridge(value: IIdeContextBridge | undefined) {
        this._ideContextBridge = value;
    }

    // Chat Launch Hint Shown
    get chatLaunchHintShown(): boolean {
        return this._chatLaunchHintShown;
    }

    set chatLaunchHintShown(value: boolean) {
        this._chatLaunchHintShown = value;
    }

    /**
     * Fire MCP definitions changed event
     */
    fireMcpDefinitionsChanged(): void {
        this._mcpDefinitionsChanged.fire();
    }

    /**
     * Dispose of all disposable resources
     */
    dispose(): void {
        this._outputChannel?.dispose();
        this._statusBarItem?.dispose();
        this._agentTerminal?.dispose();
        this._terminalCloseListener?.dispose();
        this._mcpDefinitionsChanged.dispose();
        this._ideContextBridge?.dispose();
    }
}

// Create and export singleton instance
export const state = new StateManager();
