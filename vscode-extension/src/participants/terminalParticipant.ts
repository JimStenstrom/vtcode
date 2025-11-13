import * as vscode from "vscode";
import { BaseParticipant, type ParticipantContext } from "../types/participant";
import { ConfigLimits } from "../configLimits";

/**
 * Terminal participant provides terminal context and recent command history
 */
export class TerminalParticipant extends BaseParticipant {
    public readonly id = "terminal";
    public readonly displayName = "Terminal";
    public readonly description = "Provides terminal context and recent command output";
    public readonly icon = "terminal";

    canHandle(context: ParticipantContext): boolean {
        // Available when terminal context is provided
        return context.terminal !== undefined;
    }

    async resolveReferenceContext(message: string, context: ParticipantContext): Promise<string> {
        if (!this.extractMention(message, this.id)) {
            return message;
        }

        const terminal = context.terminal;
        if (!terminal) {
            return message;
        }

        // Clean the message first
        const cleanedMessage = this.cleanMessage(message, this.id);

        // Build terminal context
        let terminalContext = `\n\n## Terminal Context\n`;
        terminalContext += `Working directory: ${terminal.cwd}\n`;
        terminalContext += `Shell: ${terminal.shell || 'default'}\n`;

        // Add recent output if available
        if (terminal.output) {
            const outputLines = terminal.output.split('\n');
            const maxOutputLines = ConfigLimits.terminalOutputLines;
            const recentOutput = outputLines.slice(-maxOutputLines).join('\n');
            if (recentOutput.trim()) {
                terminalContext += `\nRecent terminal output (last ${maxOutputLines} lines):\n\`\`\`\n${recentOutput}\n\`\`\`\n`;
            }
        }

        // Add command history if available
        if (context.commandHistory && context.commandHistory.length > 0) {
            const maxCommands = ConfigLimits.terminalHistoryCommands;
            const recentCommands = context.commandHistory.slice(-maxCommands);
            terminalContext += `\nRecent commands (last ${maxCommands}):\n`;
            recentCommands.forEach((cmd, index) => {
                terminalContext += `${index + 1}. ${cmd}\n`;
            });
        }

        return `${cleanedMessage}${terminalContext}`;
    }
}