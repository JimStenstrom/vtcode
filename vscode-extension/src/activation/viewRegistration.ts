/**
 * View Registration Module
 *
 * Handles registration of all VS Code views, tree data providers, and status bar items
 */

import * as vscode from "vscode";
import { ChatViewProvider } from "../chatView";
import { VtcodeBackend } from "../vtcodeBackend";
import { ParticipantRegistry } from "../participantRegistry";
import { VtcodeConfigSummary } from "../vtcodeConfig";

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

class QuickActionTreeItem extends vscode.TreeItem {
    constructor(public readonly action: QuickActionDescription) {
        super(action.label, vscode.TreeItemCollapsibleState.None);
        this.description = action.description;
        this.iconPath = new vscode.ThemeIcon(action.icon ?? "rocket");
        this.command = {
            command: action.command,
            title: action.label,
            arguments: action.args,
        };
        this.contextValue = "vtcodeQuickAction";
    }
}

class QuickActionTreeDataProvider
    implements vscode.TreeDataProvider<QuickActionTreeItem>
{
    private readonly onDidChangeTreeDataEmitter =
        new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this.onDidChangeTreeDataEmitter.event;

    constructor(private readonly getActions: () => QuickActionDescription[]) {}

    getTreeItem(element: QuickActionTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(): vscode.ProviderResult<QuickActionTreeItem[]> {
        return this.getActions().map(
            (action) => new QuickActionTreeItem(action)
        );
    }

    refresh(): void {
        this.onDidChangeTreeDataEmitter.fire();
    }
}

class WorkspaceInsightTreeItem extends vscode.TreeItem {
    constructor(public readonly insight: WorkspaceInsightDescription) {
        super(insight.label, vscode.TreeItemCollapsibleState.None);
        this.description = insight.description;
        this.iconPath = new vscode.ThemeIcon(insight.icon);
        this.command = insight.command;
        if (insight.tooltip) {
            this.tooltip = insight.tooltip;
        }
    }
}

class WorkspaceInsightsTreeDataProvider
    implements vscode.TreeDataProvider<WorkspaceInsightTreeItem>
{
    private readonly onDidChangeTreeDataEmitter =
        new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this.onDidChangeTreeDataEmitter.event;

    constructor(
        private readonly getInsights: () => WorkspaceInsightDescription[]
    ) {}

    getTreeItem(element: WorkspaceInsightTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(): vscode.ProviderResult<WorkspaceInsightTreeItem[]> {
        return this.getInsights().map(
            (insight) => new WorkspaceInsightTreeItem(insight)
        );
    }

    refresh(): void {
        this.onDidChangeTreeDataEmitter.fire();
    }
}

export interface ViewRegistrationOptions {
    context: vscode.ExtensionContext;
    chatBackend: VtcodeBackend;
    participantRegistry: ParticipantRegistry;
    outputChannel: vscode.OutputChannel;
    workspaceTrusted: boolean;
    getQuickActions: () => QuickActionDescription[];
    getWorkspaceInsights: () => WorkspaceInsightDescription[];
}

export interface RegisteredViews {
    chatViewProvider: ChatViewProvider;
    statusBarItem: vscode.StatusBarItem;
    quickActionsProvider: QuickActionTreeDataProvider;
    workspaceInsightsProvider: WorkspaceInsightsTreeDataProvider;
}

/**
 * Registers all views and providers
 */
export function registerViews(options: ViewRegistrationOptions): RegisteredViews {
    const {
        context,
        chatBackend,
        participantRegistry,
        outputChannel,
        workspaceTrusted,
        getQuickActions,
        getWorkspaceInsights,
    } = options;

    // Register chat view provider
    outputChannel.appendLine("[info] Registering chat view provider...");
    const chatViewProvider = new ChatViewProvider(
        context.extensionUri,
        chatBackend,
        outputChannel,
        context
    );

    // Register participants with the chat view
    chatViewProvider.registerParticipants(participantRegistry);

    const viewId = "vtcodeChatView"; // Hardcoded to match package.json
    outputChannel.appendLine(`[info] Registering view with ID: ${viewId}`);
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(
            viewId,
            chatViewProvider
        )
    );
    outputChannel.appendLine(
        "[info] Chat view provider registered successfully"
    );

    chatViewProvider.setWorkspaceTrusted(workspaceTrusted);

    // Create status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Left,
        100
    );
    statusBarItem.name = "VTCode Quick Actions";
    statusBarItem.accessibilityInformation = {
        role: "button",
        label: "Open VTCode quick actions or installation guide",
    };
    context.subscriptions.push(statusBarItem);

    // Register workspace insights provider
    const workspaceInsightsProvider = new WorkspaceInsightsTreeDataProvider(
        getWorkspaceInsights
    );
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider(
            "vtcodeWorkspaceInsights",
            workspaceInsightsProvider
        )
    );

    // Register quick actions provider
    const quickActionsProvider = new QuickActionTreeDataProvider(
        getQuickActions
    );
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider(
            "vtcodeQuickActions",
            quickActionsProvider
        )
    );

    return {
        chatViewProvider,
        statusBarItem,
        quickActionsProvider,
        workspaceInsightsProvider,
    };
}
