import * as vscode from "vscode";
import { IIdeContextBridge } from "../types/extensionTypes";
import { buildIdeContextBlock } from "../utils/contextUtils";
import { getOutputChannel } from "../utils/processUtils";

/**
 * Manages a file that contains IDE context for VTCode CLI
 */
export class IdeContextFileBridge implements IIdeContextBridge {
    private pendingTimer: NodeJS.Timeout | undefined;
    private currentRefresh: Promise<void> | undefined;
    private disposed = false;

    constructor(private readonly fileUri: vscode.Uri) {}

    dispose(): void {
        this.disposed = true;
        if (this.pendingTimer) {
            clearTimeout(this.pendingTimer);
            this.pendingTimer = undefined;
        }
    }

    scheduleRefresh(delay = 200): void {
        if (this.disposed) {
            return;
        }
        if (this.pendingTimer) {
            clearTimeout(this.pendingTimer);
        }
        this.pendingTimer = setTimeout(() => {
            this.pendingTimer = undefined;
            void this.performRefresh();
        }, delay);
    }

    async flush(): Promise<void> {
        if (this.disposed) {
            return;
        }
        if (this.pendingTimer) {
            clearTimeout(this.pendingTimer);
            this.pendingTimer = undefined;
        }
        await this.performRefresh();
    }

    get filePath(): string | undefined {
        if (this.fileUri.scheme !== "file") {
            return undefined;
        }
        return this.fileUri.fsPath;
    }

    private async performRefresh(): Promise<void> {
        if (this.disposed) {
            return;
        }

        if (this.currentRefresh) {
            await this.currentRefresh;
            return;
        }

        const task = (async () => {
            try {
                const block = await buildIdeContextBlock({
                    includeActiveEditor: true,
                    includeVisibleEditors: true,
                });
                const content = block ? `${block}\n` : "";
                await vscode.workspace.fs.writeFile(
                    this.fileUri,
                    Buffer.from(content, "utf8")
                );
            } catch (error) {
                const message =
                    error instanceof Error ? error.message : String(error);
                getOutputChannel().appendLine(
                    `[warn] Failed to update IDE context snapshot: ${message}`
                );
            }
        })();

        this.currentRefresh = task;
        try {
            await task;
        } finally {
            if (this.currentRefresh === task) {
                this.currentRefresh = undefined;
            }
        }
    }
}

/**
 * Check if a document is currently visible in any editor
 */
export function isDocumentVisible(document: vscode.TextDocument): boolean {
    if (vscode.window.activeTextEditor?.document === document) {
        return true;
    }

    return vscode.window.visibleTextEditors.some(
        (editor) => editor.document === document
    );
}

/**
 * Initialize IDE context bridge and register event listeners
 */
export async function initializeIdeContextBridge(
    context: vscode.ExtensionContext
): Promise<IdeContextFileBridge | undefined> {
    const storageRoot = context.globalStorageUri ?? context.storageUri;
    if (!storageRoot || storageRoot.scheme !== "file") {
        return undefined;
    }

    try {
        await vscode.workspace.fs.createDirectory(storageRoot);
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        getOutputChannel().appendLine(
            `[warn] Failed to prepare IDE context storage: ${message}`
        );
    }

    const fileUri = vscode.Uri.joinPath(storageRoot, "vtcode-ide-context.md");
    const bridge = new IdeContextFileBridge(fileUri);
    context.subscriptions.push(bridge);

    await bridge.flush();

    const scheduleRefresh = () => bridge.scheduleRefresh();
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(() => scheduleRefresh()),
        vscode.window.onDidChangeVisibleTextEditors(() => scheduleRefresh()),
        vscode.window.onDidChangeTextEditorSelection(() => scheduleRefresh()),
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (isDocumentVisible(event.document)) {
                scheduleRefresh();
            }
        }),
        vscode.workspace.onDidSaveTextDocument((document) => {
            if (isDocumentVisible(document)) {
                scheduleRefresh();
            }
        }),
        vscode.workspace.onDidCloseTextDocument(() => scheduleRefresh()),
        vscode.workspace.onDidChangeWorkspaceFolders(() => scheduleRefresh())
    );

    return bridge;
}
