import * as vscode from "vscode";
import { WorkspaceInsightDescription } from "../types/extensionTypes";

export class WorkspaceInsightTreeItem extends vscode.TreeItem {
    constructor(public readonly insight: WorkspaceInsightDescription) {
        super(insight.label, vscode.TreeItemCollapsibleState.None);
        this.description = insight.description;
        this.iconPath = new vscode.ThemeIcon(insight.icon);
        this.command = insight.command;
        if (insight.tooltip) {
            this.tooltip = insight.tooltip;
        }
        this.contextValue = "vtcodeWorkspaceInsight";
    }
}

export class WorkspaceInsightsTreeDataProvider
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
