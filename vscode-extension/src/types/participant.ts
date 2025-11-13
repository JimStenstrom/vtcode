import * as vscode from "vscode";

/**
 * Context provided to participants for context resolution
 */
export interface ParticipantContext {
    /** Active file information */
    activeFile?: {
        path: string;
        language: string;
        content?: string;
        selection?: vscode.Range;
    };
    /** Workspace folder */
    workspace?: vscode.WorkspaceFolder;
    /** Terminal state */
    terminal?: { 
        output: string; 
        cwd: string;
        shell?: string;
    };
    /** Git repository state */
    git?: { 
        branch: string; 
        changes: string[];
        repoPath?: string;
    };
    /** Recent commands history */
    commandHistory?: string[];
}

/**
 * Base interface for all VTCode chat participants
 */
export interface ChatParticipant {
    /** Unique participant identifier (e.g., '@workspace') */
    readonly id: string;
    /** Human-readable display name */
    readonly displayName: string;
    /** Optional description */
    readonly description?: string;
    /** Optional icon */
    readonly icon?: string;
    
    /**
     * Check if this participant can handle the given context
     * @param context Participant context
     * @returns true if participant can provide context
     */
    canHandle(context: ParticipantContext): boolean;
    
    /**
     * Resolve context for a message by adding participant-specific information
     * @param message Original user message
     * @param context Participant context
     * @returns Enhanced message with participant context
     */
    resolveReferenceContext(
        message: string,
        context: ParticipantContext
    ): Promise<string>;
}

/**
 * Base class for VTCode participants providing common functionality
 */
export abstract class BaseParticipant implements ChatParticipant {
    public abstract readonly id: string;
    public abstract readonly displayName: string;
    public readonly description?: string;
    public readonly icon?: string;

    /**
     * Check if a file is within the workspace
     */
    protected isFileInWorkspace(filePath: string, context: ParticipantContext): boolean {
        if (!context.workspace) {
            return false;
        }
        return filePath.startsWith(context.workspace.uri.fsPath);
    }

    /**
     * Extract @mention from message
     */
    protected extractMention(message: string, mention: string): boolean {
        const regex = new RegExp(`@${mention}\\b`, 'i');
        return regex.test(message);
    }

    /**
     * Remove @mention from message to prevent duplication
     */
    protected cleanMessage(message: string, mention: string): string {
        const regex = new RegExp(`@${mention}\\b\\s*`, 'gi');
        return message.replace(regex, '').trim();
    }

    /**
     * Build contextual message using a common pattern
     * Eliminates duplicate validation and message cleaning logic across all participants
     *
     * @param message Original user message
     * @param context Participant context
     * @param contextExtractor Function to extract relevant context data
     * @param contextBuilder Function to build context string from data
     * @returns Enhanced message with context
     */
    protected async buildContextualMessage<T>(
        message: string,
        context: ParticipantContext,
        contextExtractor: (context: ParticipantContext) => T | undefined,
        contextBuilder: (data: T) => Promise<string> | string
    ): Promise<string> {
        // Check if participant is mentioned
        if (!this.extractMention(message, this.id)) {
            return message;
        }

        // Extract context data
        const data = contextExtractor(context);
        if (!data) {
            return message;
        }

        // Clean the message
        const cleanedMessage = this.cleanMessage(message, this.id);

        // Build context string
        const contextString = await contextBuilder(data);

        // Return enhanced message
        return `${cleanedMessage}${contextString}`;
    }

    abstract canHandle(context: ParticipantContext): boolean;
    abstract resolveReferenceContext(message: string, context: ParticipantContext): Promise<string>;
}