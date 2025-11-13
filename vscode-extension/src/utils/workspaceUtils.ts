/**
 * Centralized workspace utilities
 * Consolidates workspace path resolution and related operations
 */

import * as vscode from 'vscode';

/**
 * Get the workspace root directory
 * Prefers the active editor's workspace, falls back to first workspace
 * Consolidates duplicate getWorkspaceRoot implementations
 */
export function getWorkspaceRoot(): string | undefined {
    const activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(
            activeEditor.document.uri
        );
        return workspaceFolder?.uri.fsPath;
    }

    const [firstWorkspace] = vscode.workspace.workspaceFolders ?? [];
    return firstWorkspace?.uri.fsPath;
}

/**
 * Get workspace folder for a specific URI
 */
export function getWorkspaceFolderForUri(uri: vscode.Uri): vscode.WorkspaceFolder | undefined {
    return vscode.workspace.getWorkspaceFolder(uri);
}

/**
 * Convert a file path to a workspace-relative path
 */
export function toWorkspaceRelativePath(filePath: string): string {
    return vscode.workspace.asRelativePath(filePath, false);
}

/**
 * Check if a workspace is open
 */
export function hasWorkspace(): boolean {
    return (vscode.workspace.workspaceFolders?.length ?? 0) > 0;
}

/**
 * Get the first workspace folder (if any)
 */
export function getFirstWorkspaceFolder(): vscode.WorkspaceFolder | undefined {
    return vscode.workspace.workspaceFolders?.[0];
}
