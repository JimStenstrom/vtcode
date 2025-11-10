import * as vscode from "vscode";
import {
    AppendIdeContextOptions,
    DocumentContext,
    IDE_CONTEXT_HEADER,
    MAX_IDE_CONTEXT_CHARS,
    MAX_FULL_DOCUMENT_CONTEXT_LINES,
    ACTIVE_EDITOR_CONTEXT_WINDOW,
    MAX_VISIBLE_EDITOR_CONTEXTS,
} from "../types/extensionTypes";

/**
 * Append IDE context to a prompt
 */
export async function appendIdeContextToPrompt(
    prompt: string,
    options: AppendIdeContextOptions = {}
): Promise<string> {
    const contextBlock = await buildIdeContextBlock(options);
    if (!contextBlock) {
        return prompt;
    }

    const trimmedPrompt = prompt.trimEnd();
    const basePrompt = trimmedPrompt.length > 0 ? trimmedPrompt : prompt;

    if (basePrompt.length === 0) {
        return contextBlock;
    }

    return `${basePrompt}\n\n${contextBlock}`;
}

/**
 * Build IDE context block from available context sections
 */
export async function buildIdeContextBlock(
    options: AppendIdeContextOptions = {}
): Promise<string | undefined> {
    const sections = await collectIdeContextSections(options);
    if (sections.length === 0) {
        return undefined;
    }

    return [IDE_CONTEXT_HEADER, ...sections].join("\n\n");
}

/**
 * Collect all IDE context sections
 */
async function collectIdeContextSections(
    options: AppendIdeContextOptions = {}
): Promise<string[]> {
    const sections: string[] = [];
    const seenKeys = new Set<string>();
    const token = options.cancellationToken;

    if (token?.isCancellationRequested) {
        throw new vscode.CancellationError();
    }

    if (options.includeActiveEditor !== false) {
        const activeSection = await buildActiveEditorContextSection(
            seenKeys,
            token
        );
        if (activeSection) {
            sections.push(activeSection);
        }
    }

    if (options.includeVisibleEditors) {
        const visibleSections = await buildVisibleEditorContextSections(
            seenKeys,
            token
        );
        if (visibleSections.length > 0) {
            sections.push(...visibleSections);
        }
    }

    if (options.chatRequest) {
        const referenceSections = await buildReferenceContextSections(
            options.chatRequest,
            seenKeys,
            token
        );
        if (referenceSections.length > 0) {
            sections.push(...referenceSections);
        }
    }

    return sections;
}

/**
 * Build context section for active editor
 */
async function buildActiveEditorContextSection(
    seen: Set<string>,
    token?: vscode.CancellationToken
): Promise<string | undefined> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return undefined;
    }

    if (token?.isCancellationRequested) {
        throw new vscode.CancellationError();
    }

    const document = editor.document;
    const preferredRange = computeActiveEditorRange(editor);
    const context = extractDocumentContext(document, preferredRange);
    if (!context) {
        return undefined;
    }

    const key = createContextKey(document.uri, context.range, "active-editor");
    if (!registerContextKey(seen, key)) {
        return undefined;
    }

    const label = getPathLabel(document.uri);
    const detailParts: string[] = [];
    if (context.range) {
        detailParts.push(`lines ${formatRangeLabel(context.range)}`);
    }
    if (document.isDirty) {
        detailParts.push("unsaved changes");
    }
    if (context.truncated) {
        detailParts.push("truncated");
    }

    const headingDetails =
        detailParts.length > 0 ? ` (${detailParts.join(" • ")})` : "";
    const heading = `### Active Editor: ${label}${headingDetails}`;
    const codeBlock = formatCodeBlock(document.languageId, context.text);
    const notes = context.truncated
        ? "_Context truncated to fit VS Code chat limits._"
        : undefined;

    return [heading, codeBlock, notes].filter(Boolean).join("\n\n");
}

/**
 * Build context sections for visible editors
 */
async function buildVisibleEditorContextSections(
    seen: Set<string>,
    token?: vscode.CancellationToken
): Promise<string[]> {
    const sections: string[] = [];
    const activeEditor = vscode.window.activeTextEditor;
    const activeUri = activeEditor?.document.uri.toString();

    for (const editor of vscode.window.visibleTextEditors) {
        if (sections.length >= MAX_VISIBLE_EDITOR_CONTEXTS) {
            break;
        }

        if (token?.isCancellationRequested) {
            throw new vscode.CancellationError();
        }

        const document = editor.document;
        if (document.uri.toString() === activeUri) {
            continue;
        }

        const context = extractDocumentContext(document, undefined);
        if (!context) {
            continue;
        }

        const key = createContextKey(
            document.uri,
            context.range,
            "visible-editor"
        );
        if (!registerContextKey(seen, key)) {
            continue;
        }

        const label = getPathLabel(document.uri);
        const detailParts: string[] = [];
        if (context.range) {
            detailParts.push(`lines ${formatRangeLabel(context.range)}`);
        }
        if (document.isDirty) {
            detailParts.push("unsaved changes");
        }
        if (context.truncated) {
            detailParts.push("truncated");
        }

        const headingDetails =
            detailParts.length > 0 ? ` (${detailParts.join(" • ")})` : "";
        const heading = `### Editor: ${label}${headingDetails}`;
        const codeBlock = formatCodeBlock(document.languageId, context.text);
        const notes = context.truncated
            ? "_Context truncated to fit VS Code chat limits._"
            : undefined;

        sections.push([heading, codeBlock, notes].filter(Boolean).join("\n\n"));
    }

    return sections;
}

/**
 * Build context sections from chat references
 */
async function buildReferenceContextSections(
    request: vscode.ChatRequest,
    seen: Set<string>,
    token?: vscode.CancellationToken
): Promise<string[]> {
    const sections: string[] = [];

    for (const reference of request.references ?? []) {
        if (token?.isCancellationRequested) {
            throw new vscode.CancellationError();
        }

        const section = await buildReferenceContextSection(
            reference,
            seen,
            token
        );
        if (section) {
            sections.push(section);
        }
    }

    return sections;
}

/**
 * Build context section from a single chat reference
 */
async function buildReferenceContextSection(
    reference: vscode.ChatPromptReference,
    seen: Set<string>,
    token?: vscode.CancellationToken
): Promise<string | undefined> {
    const value = reference.value;

    if (typeof value === "string") {
        const trimmed = value.trim();
        if (!trimmed) {
            return undefined;
        }

        const key = createContextKey(undefined, undefined, `string:${trimmed}`);
        if (!registerContextKey(seen, key)) {
            return undefined;
        }

        const description = reference.modelDescription?.trim();
        const headingLabel =
            description && description.length > 0
                ? description
                : `Reference ${reference.id}`;
        const heading = `### Reference: ${headingLabel}`;
        const block = formatCodeBlock("text", trimmed);
        return `${heading}\n\n${block}`;
    }

    if (value instanceof vscode.Location) {
        const document = await vscode.workspace.openTextDocument(value.uri);
        if (token?.isCancellationRequested) {
            throw new vscode.CancellationError();
        }

        const context = extractDocumentContext(document, value.range);
        if (!context) {
            return undefined;
        }

        const key = createContextKey(value.uri, context.range, reference.id);
        if (!registerContextKey(seen, key)) {
            return undefined;
        }

        const label = getPathLabel(value.uri);
        const description = reference.modelDescription?.trim();
        const headingLabel =
            description && description.length > 0 ? description : label;
        const details: string[] = [`lines ${formatRangeLabel(context.range)}`];
        if (context.truncated) {
            details.push("truncated");
        }
        const detailText =
            details.length > 0 ? ` (${details.join(" • ")})` : "";
        const heading = `### Reference: ${headingLabel}${detailText}`;
        const block = formatCodeBlock(document.languageId, context.text);
        const notes = context.truncated
            ? "_Context truncated to fit VS Code chat limits._"
            : undefined;
        return [heading, block, notes].filter(Boolean).join("\n\n");
    }

    if (value instanceof vscode.Uri) {
        const document = await vscode.workspace.openTextDocument(value);
        if (token?.isCancellationRequested) {
            throw new vscode.CancellationError();
        }

        const context = extractDocumentContext(document, undefined);
        if (!context) {
            return undefined;
        }

        const key = createContextKey(value, context.range, reference.id);
        if (!registerContextKey(seen, key)) {
            return undefined;
        }

        const label = getPathLabel(value);
        const description = reference.modelDescription?.trim();
        const headingLabel =
            description && description.length > 0 ? description : label;
        const details: string[] = [];
        if (context.range) {
            details.push(`lines ${formatRangeLabel(context.range)}`);
        }
        if (context.truncated) {
            details.push("truncated");
        }
        const detailText =
            details.length > 0 ? ` (${details.join(" • ")})` : "";
        const heading = `### Reference: ${headingLabel}${detailText}`;
        const block = formatCodeBlock(document.languageId, context.text);
        const notes = context.truncated
            ? "_Context truncated to fit VS Code chat limits._"
            : undefined;
        return [heading, block, notes].filter(Boolean).join("\n\n");
    }

    return undefined;
}

/**
 * Compute the best range to include for the active editor
 */
function computeActiveEditorRange(
    editor: vscode.TextEditor
): vscode.Range | undefined {
    const document = editor.document;
    if (!editor.selection.isEmpty) {
        return editor.selection;
    }

    const visibleRanges = editor.visibleRanges.filter(
        (range) => !range.isEmpty
    );
    if (visibleRanges.length > 0) {
        const first = visibleRanges[0];
        const last = visibleRanges[visibleRanges.length - 1];
        return new vscode.Range(first.start, last.end);
    }

    if (document.lineCount === 0) {
        return undefined;
    }

    if (document.lineCount <= MAX_FULL_DOCUMENT_CONTEXT_LINES) {
        const lastLineIndex = Math.max(0, document.lineCount - 1);
        const endPosition = document.lineAt(lastLineIndex).range.end;
        return new vscode.Range(new vscode.Position(0, 0), endPosition);
    }

    const activeLine = editor.selection.active.line;
    const halfWindow = Math.max(
        1,
        Math.floor(ACTIVE_EDITOR_CONTEXT_WINDOW / 2)
    );
    const startLine = Math.max(0, activeLine - halfWindow);
    const endLine = Math.min(document.lineCount - 1, activeLine + halfWindow);
    const endPosition = document.lineAt(endLine).range.end;
    return new vscode.Range(new vscode.Position(startLine, 0), endPosition);
}

/**
 * Extract document context from a document and range
 */
function extractDocumentContext(
    document: vscode.TextDocument,
    range: vscode.Range | undefined
): DocumentContext | undefined {
    if (document.lineCount === 0) {
        return undefined;
    }

    let targetRange = range;
    let truncated = false;

    if (!targetRange) {
        const totalLines = document.lineCount;
        const endLineIndex =
            Math.min(totalLines, MAX_FULL_DOCUMENT_CONTEXT_LINES) - 1;
        const endPosition = document.lineAt(Math.max(0, endLineIndex)).range
            .end;
        targetRange = new vscode.Range(new vscode.Position(0, 0), endPosition);
        if (totalLines > MAX_FULL_DOCUMENT_CONTEXT_LINES) {
            truncated = true;
        }
    }

    const rawText = document.getText(targetRange);
    const normalized = normalizeForPrompt(rawText);
    if (!normalized.trim()) {
        return undefined;
    }

    const limited = truncateForPrompt(normalized, MAX_IDE_CONTEXT_CHARS);
    return {
        text: limited.text,
        range: targetRange,
        truncated: truncated || limited.truncated,
    };
}

/**
 * Format code block with language identifier
 */
function formatCodeBlock(
    languageId: string | undefined,
    content: string
): string {
    const language =
        languageId && languageId.trim().length > 0 ? languageId : "text";
    return `\`\`\`${language}\n${content}\n\`\`\``;
}

/**
 * Get path label for a URI
 */
function getPathLabel(uri: vscode.Uri): string {
    if (uri.scheme === "untitled") {
        const segments = uri.path.split("/");
        const name = segments[segments.length - 1] || "untitled";
        return `untitled:${name}`;
    }

    const relative = vscode.workspace.asRelativePath(uri, false);
    if (relative && relative !== uri.toString()) {
        return relative;
    }

    if (uri.scheme === "file") {
        return uri.fsPath;
    }

    return uri.toString(true);
}

/**
 * Format a range label for display
 */
function formatRangeLabel(range: vscode.Range): string {
    const startLine = range.start.line + 1;
    const endLine = range.end.line + 1;
    return startLine === endLine ? `${startLine}` : `${startLine}-${endLine}`;
}

/**
 * Create a unique context key
 */
function createContextKey(
    uri: vscode.Uri | undefined,
    range: vscode.Range | undefined,
    fallback: string
): string {
    const base = uri ? uri.toString() : fallback;
    if (range) {
        return `${base}:${range.start.line}:${range.start.character}-${range.end.line}:${range.end.character}`;
    }
    return base;
}

/**
 * Register a context key to track duplicates
 */
function registerContextKey(seen: Set<string>, key: string): boolean {
    if (seen.has(key)) {
        return false;
    }
    seen.add(key);
    return true;
}

/**
 * Normalize text for prompts (convert line endings)
 */
function normalizeForPrompt(text: string): string {
    return text.replace(/\r\n/g, "\n");
}

/**
 * Truncate text to fit prompt limits
 */
function truncateForPrompt(
    text: string,
    limit: number
): { text: string; truncated: boolean } {
    if (text.length <= limit) {
        return { text, truncated: false };
    }

    return { text: text.slice(0, limit), truncated: true };
}
