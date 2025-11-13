/**
 * Centralized validation helpers
 * Eliminates duplicate validation patterns across commands
 */

import * as vscode from 'vscode';
import { NotificationService } from './notificationService';

export class ValidationHelpers {
    /**
     * Require an active text editor
     * Shows warning if none is available
     */
    static requireActiveEditor(): vscode.TextEditor | undefined {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            void NotificationService.warning(
                "Open a text editor to continue."
            );
        }
        return editor;
    }

    /**
     * Require a non-empty selection in the active editor
     * Shows warning if selection is empty
     */
    static requireSelection(editor: vscode.TextEditor): boolean {
        if (editor.selection.isEmpty) {
            void NotificationService.warning(
                "Highlight text first to continue."
            );
            return false;
        }
        return true;
    }

    /**
     * Require non-empty text selection
     * Shows warning if text is empty or whitespace
     */
    static requireNonEmptyText(text: string, context: string = "text"): boolean {
        if (!text.trim()) {
            void NotificationService.warning(
                `Please select some ${context} to continue.`
            );
            return false;
        }
        return true;
    }

    /**
     * Require a trusted workspace
     * Shows warning if workspace is not trusted
     */
    static requireTrustedWorkspace(): boolean {
        if (!vscode.workspace.isTrusted) {
            void NotificationService.warning(
                "VTCode requires a trusted workspace to execute this operation. Click 'Trust Workspace' in VS Code to enable full functionality."
            );
            return false;
        }
        return true;
    }

    /**
     * Require a workspace folder to be open
     * Shows warning if no workspace is open
     */
    static requireWorkspace(): boolean {
        if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
            void NotificationService.warning(
                "Please open a workspace or folder to continue."
            );
            return false;
        }
        return true;
    }

    /**
     * Validate active editor with selection
     * Combined validation for common command pattern
     */
    static validateEditorWithSelection(): { editor: vscode.TextEditor; text: string } | undefined {
        const editor = this.requireActiveEditor();
        if (!editor) {
            return undefined;
        }

        if (!this.requireSelection(editor)) {
            return undefined;
        }

        const text = editor.document.getText(editor.selection);
        if (!this.requireNonEmptyText(text)) {
            return undefined;
        }

        return { editor, text };
    }
}
