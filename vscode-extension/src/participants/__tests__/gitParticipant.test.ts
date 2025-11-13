import { describe, it, expect, beforeEach } from "vitest";
import { GitParticipant } from "../gitParticipant";
import { ParticipantContext } from "../../types/participant";

describe("GitParticipant", () => {
    let participant: GitParticipant;

    beforeEach(() => {
        participant = new GitParticipant();
    });

    it("should have correct id, displayName, and icon", () => {
        expect(participant.id).toBe("git");
        expect(participant.displayName).toBe("Git");
        expect(participant.icon).toBe("git-branch");
    });

    it("should have description", () => {
        expect(participant.description).toBeDefined();
        expect(participant.description).toContain("git");
    });

    it("should handle context with git information", () => {
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };
        expect(participant.canHandle(context)).toBe(true);
    });

    it("should not handle context without git information", () => {
        const context: ParticipantContext = {};
        expect(participant.canHandle(context)).toBe(false);
    });

    it("should return message unchanged when no @git mention", async () => {
        const message = "What are the changes?";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );
        expect(result).toBe(message);
    });

    it("should add git context when @git mentioned", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "feature/test",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Show me");
        expect(result).toContain("## Git Context");
        expect(result).toContain("Branch: feature/test");
    });

    it("should include repository path when available", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
                repoPath: "/workspace/.git",
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Repository: /workspace/.git");
    });

    it("should show clean status when no changes", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Working directory is clean");
        expect(result).toContain("Clean working directory");
    });

    it("should list changes when present", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [
                    "M src/test.ts",
                    "A src/new.ts",
                    "D src/old.ts",
                ],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Changes in working directory:");
        expect(result).toContain("1. M src/test.ts");
        expect(result).toContain("2. A src/new.ts");
        expect(result).toContain("3. D src/old.ts");
    });

    it("should provide status summary for modified files", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: ["M src/test.ts", "M src/other.ts"],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Status: 2 modified");
    });

    it("should provide status summary for added files", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: ["A src/new1.ts", "A src/new2.ts", "A src/new3.ts"],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Status: 3 added");
    });

    it("should provide status summary for deleted files", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: ["D src/old.ts"],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Status: 1 deleted");
    });

    it("should provide status summary for untracked files", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: ["?? src/untracked.ts"],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Status: 1 untracked");
    });

    it("should provide combined status summary", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [
                    "M src/modified.ts",
                    "M src/modified2.ts",
                    "A src/added.ts",
                    "D src/deleted.ts",
                    "?? src/untracked.ts",
                ],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Status:");
        expect(result).toContain("2 modified");
        expect(result).toContain("1 added");
        expect(result).toContain("1 deleted");
        expect(result).toContain("1 untracked");
    });

    it("should remove @git mention from message", async () => {
        const message = "Show me @git status please";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Show me");
        expect(result).toContain("status please");
        const lines = result.split("\n");
        expect(lines[0]).not.toContain("@git");
    });

    it("should handle case-insensitive @git mentions", async () => {
        const message = "Show me @GIT status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("## Git Context");
    });

    it("should return original message when git context missing", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {};

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toBe(message);
    });

    it("should handle empty changes array", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: [],
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Clean working directory");
    });

    it("should handle undefined changes gracefully", async () => {
        const message = "Show me @git status";
        const context: ParticipantContext = {
            git: {
                branch: "main",
                changes: undefined as any,
            },
        };

        const result = await participant.resolveReferenceContext(
            message,
            context
        );

        expect(result).toContain("Clean working directory");
    });
});
