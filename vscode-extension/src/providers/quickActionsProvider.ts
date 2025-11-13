import * as vscode from "vscode";
import { QuickActionDescription } from "../types/extensionTypes";

export type QuickActionItem = vscode.QuickPickItem & {
    run: () => Thenable<unknown> | void;
};

export class QuickActionTreeItem extends vscode.TreeItem {
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

export class QuickActionTreeDataProvider
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
