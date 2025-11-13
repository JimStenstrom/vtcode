import { describe, it, expect, beforeEach } from "vitest";
import { TerminalParticipant } from "../terminalParticipant";
import { ParticipantContext } from "../../types/participant";

describe("TerminalParticipant", () => {
    let participant: TerminalParticipant;

    beforeEach(() => {
        participant = new TerminalParticipant();
    });

    it("should have correct id, displayName, and icon", () => {
        expect(participant.id).toBe("terminal");
        expect(participant.displayName).toBe("Terminal");
        expect(participant.icon).toBe("terminal");
    });

    it("should have description", () => {
        expect(participant.description).toBeDefined();
        expect(participant.description).toContain("terminal");
    });

    it("should handle context with terminal information", () => {
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };
        expect(participant.canHandle(context)).toBe(true);
    });

    it("should not handle context without terminal information", () => {
        const context: ParticipantContext = {};
        expect(participant.canHandle(context)).toBe(false);
    });

    it("should return message unchanged when no @terminal mention", async () => {
        const message = "What was the output?";
        const context: ParticipantContext = {
            terminal: {
                output: "Hello world",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );
        expect(result).toBe(message);
    });

    it("should add terminal context when @terminal mentioned", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "Hello world",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Check");
        expect(result).toContain("## Terminal Context");
        expect(result).toContain("Working directory: /workspace");
    });

    it("should include shell information when available", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
                shell: "/bin/bash",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Shell: /bin/bash");
    });

    it("should show default shell when not specified", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Shell: default");
    });

    it("should include recent terminal output", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "$ npm test\nAll tests passed",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Recent terminal output:");
        expect(result).toContain("$ npm test");
        expect(result).toContain("All tests passed");
    });

    it("should limit terminal output to last 20 lines", async () => {
        const message = "Check @terminal output";
        const lines = Array.from({ length: 50 }, (_, i) => `line ${i + 1}`);
        const output = lines.join("\n");

        const context: ParticipantContext = {
            terminal: {
                output,
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("line 31"); // Last 20 lines start at line 31
        expect(result).toContain("line 50");
        expect(result).not.toContain("line 30");
    });

    it("should not include output section when output is empty", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).not.toContain("Recent terminal output:");
    });

    it("should not include output section when output is only whitespace", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "   \n   \n   ",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).not.toContain("Recent terminal output:");
    });

    it("should include command history when available", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
            commandHistory: ["npm install", "npm test", "git status"],
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Recent commands:");
        expect(result).toContain("1. npm install");
        expect(result).toContain("2. npm test");
        expect(result).toContain("3. git status");
    });

    it("should limit command history to last 5 commands", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
            commandHistory: [
                "cmd1",
                "cmd2",
                "cmd3",
                "cmd4",
                "cmd5",
                "cmd6",
                "cmd7",
                "cmd8",
            ],
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("cmd4");
        expect(result).toContain("cmd8");
        expect(result).not.toContain("cmd3");
        expect(result).not.toContain("cmd1");
    });

    it("should not include command history when empty", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
            commandHistory: [],
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).not.toContain("Recent commands:");
    });

    it("should not include command history when undefined", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).not.toContain("Recent commands:");
    });

    it("should remove @terminal mention from message", async () => {
        const message = "Check @terminal output please";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Check");
        expect(result).toContain("output please");
        const lines = result.split("\n");
        expect(lines[0]).not.toContain("@terminal");
    });

    it("should handle case-insensitive @terminal mentions", async () => {
        const message = "Check @TERMINAL output";
        const context: ParticipantContext = {
            terminal: {
                output: "",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("## Terminal Context");
    });

    it("should return original message when terminal context missing", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {};

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toBe(message);
    });

    it("should format terminal output in code blocks", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "$ ls\nfile1.txt file2.txt",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("```\n$ ls\nfile1.txt file2.txt\n```");
    });

    it("should handle multiline terminal output", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output:
                    "$ npm test\n\n> test\n> vitest\n\n✓ All tests passed",
                cwd: "/workspace",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("$ npm test");
        expect(result).toContain("✓ All tests passed");
    });

    it("should include both output and command history", async () => {
        const message = "Check @terminal output";
        const context: ParticipantContext = {
            terminal: {
                output: "$ npm test\nPassed",
                cwd: "/workspace",
            },
            commandHistory: ["npm install", "npm test"],
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Recent terminal output:");
        expect(result).toContain("Recent commands:");
        expect(result).toContain("$ npm test");
        expect(result).toContain("1. npm install");
    });
});
