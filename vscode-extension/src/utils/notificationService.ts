/**
 * Centralized notification service
 * Wraps VS Code notification APIs and integrates with error handling
 * Eliminates 80+ instances of repetitive notification calls
 */

import * as vscode from 'vscode';
import { ErrorPresentationHandler } from '../error/errorPresentation';
import { getErrorMessage } from './errorUtils';

export class NotificationService {
    /**
     * Show an error message
     */
    static error(message: string, ...actions: string[]): vscode.Thenable<string | undefined> {
        return vscode.window.showErrorMessage(message, ...actions);
    }

    /**
     * Show a warning message
     */
    static warning(message: string, ...actions: string[]): vscode.Thenable<string | undefined> {
        return vscode.window.showWarningMessage(message, ...actions);
    }

    /**
     * Show an information message
     */
    static info(message: string, ...actions: string[]): vscode.Thenable<string | undefined> {
        return vscode.window.showInformationMessage(message, ...actions);
    }

    /**
     * Show an error with optional context
     * Integrates with ErrorPresentationHandler for better user experience
     */
    static fromError(error: unknown, context?: string): vscode.Thenable<string | undefined> {
        const presentation = ErrorPresentationHandler.format(error);
        const message = context
            ? `Failed to ${context}: ${presentation.message}`
            : presentation.message;

        const actions: string[] = [];
        if (presentation.suggestion) {
            // Could add action buttons based on suggestions
        }

        return this.error(message, ...actions);
    }

    /**
     * Show an error with full presentation details
     */
    static fromErrorDetailed(error: unknown): vscode.Thenable<string | undefined> {
        const presentation = ErrorPresentationHandler.format(error);
        let message = `${presentation.title}\n\n${presentation.message}`;

        if (presentation.suggestion) {
            message += `\n\nSuggestion: ${presentation.suggestion}`;
        }

        return this.error(message);
    }

    /**
     * Show a command error (common pattern in commands)
     */
    static commandError(context: string, error: unknown): void {
        const message = getErrorMessage(error);
        void this.error(`Failed to ${context} with VTCode: ${message}`);
    }
}
