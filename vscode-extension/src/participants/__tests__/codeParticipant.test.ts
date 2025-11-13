import { describe, it, expect, beforeEach, vi } from "vitest";
import { CodeParticipant } from "../codeParticipant";
import { ParticipantContext } from "../../types/participant";
import * as vscode from "vscode";

describe("CodeParticipant", () => {
    let participant: CodeParticipant;

    beforeEach(() => {
        participant = new CodeParticipant();
    });

    it("should have correct id, displayName, and icon", () => {
        expect(participant.id).toBe("code");
        expect(participant.displayName).toBe("Code");
        expect(participant.icon).toBe("code");
    });

    it("should have description", () => {
        expect(participant.description).toBeDefined();
        expect(participant.description).toContain("code");
    });

    it("should handle context with active code file", () => {
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };
        expect(participant.canHandle(context)).toBe(true);
    });

    it("should not handle context without active file", () => {
        const context: ParticipantContext = {};
        expect(participant.canHandle(context)).toBe(false);
    });

    it("should not handle text files", () => {
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/test.txt",
                language: "text",
            },
        };
        expect(participant.canHandle(context)).toBe(false);
    });

    it("should not handle markdown files", () => {
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/README.md",
                language: "markdown",
            },
        };
        expect(participant.canHandle(context)).toBe(false);
    });

    it("should return message unchanged when no @code mention", async () => {
        const message = "What does this do?";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );
        expect(result).toBe(message);
    });

    it("should add code context when @code mentioned", async () => {
        const message = "Explain @code";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Explain");
        expect(result).toContain("## Code Context");
        expect(result).toContain("File:");
        expect(result).toContain("Language: typescript");
    });

    it("should include relative path when file is in workspace", async () => {
        const message = "Explain @code";
        const mockWorkspace = {
            uri: { fsPath: "/workspace" },
        } as vscode.WorkspaceFolder;

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
            workspace: mockWorkspace,
        };

        vi.spyOn(vscode.workspace, "asRelativePath").mockReturnValue(
            "src/test.ts"
        );

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("src/test.ts");
    });

    it("should include selection information when available", async () => {
        const message = "Explain @code";
        const selection = new vscode.Range(
            new vscode.Position(5, 0),
            new vscode.Position(10, 0)
        );

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
                selection,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Selection: Lines 6-11");
    });

    it("should include selected code snippet", async () => {
        const message = "Explain @code";
        const selection = new vscode.Range(
            new vscode.Position(0, 0),
            new vscode.Position(2, 0)
        );

        const content = "function test() {\n  console.log('hello');\n  return true;\n}";

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
                content,
                selection,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Selected code:");
        expect(result).toContain("```typescript");
        expect(result).toContain("function test()");
        expect(result).toContain("console.log('hello')");
    });

    it("should include file snippet when no selection", async () => {
        const message = "Explain @code";
        const content = "function test() {\n  return true;\n}";

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
                content,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("File snippet:");
        expect(result).toContain("```typescript");
        expect(result).toContain("function test()");
    });

    it("should limit file snippet to 50 lines", async () => {
        const message = "Explain @code";
        const lines = Array.from({ length: 100 }, (_, i) => `line ${i + 1}`);
        const content = lines.join("\n");

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
                content,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("line 1");
        expect(result).toContain("line 50");
        expect(result).not.toContain("line 51");
    });

    it("should include language info for known languages", async () => {
        const message = "Explain @code";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.py",
                language: "python",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Language details:");
        expect(result).toContain("Interpreted, high-level");
    });

    it("should handle TypeScript language info", async () => {
        const message = "Explain @code";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Statically typed superset of JavaScript");
    });

    it("should handle Rust language info", async () => {
        const message = "Explain @code";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/main.rs",
                language: "rust",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("memory safety");
    });

    it("should not add language info for unknown languages", async () => {
        const message = "Explain @code";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.xyz",
                language: "xyz",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).not.toContain("Language details:");
    });

    it("should remove @code mention from message", async () => {
        const message = "Explain @code please";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Explain");
        expect(result).toContain("please");
        const lines = result.split("\n");
        expect(lines[0]).not.toContain("@code");
    });

    it("should handle case-insensitive @code mentions", async () => {
        const message = "Explain @CODE";
        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("## Code Context");
    });

    it("should handle empty selection gracefully", async () => {
        const message = "Explain @code";
        const selection = new vscode.Range(
            new vscode.Position(5, 0),
            new vscode.Position(5, 0)
        );

        const context: ParticipantContext = {
            activeFile: {
                path: "/workspace/src/test.ts",
                language: "typescript",
                content: "function test() {}",
                selection,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        // Should fall back to file snippet since selection is empty
        expect(result).toContain("File snippet:");
    });
});
