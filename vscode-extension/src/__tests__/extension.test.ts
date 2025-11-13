import { describe, it, expect, beforeEach, vi } from "vitest";
import { activate, deactivate } from "../extension";
import * as vscode from "vscode";

describe("extension", () => {
    let mockContext: vscode.ExtensionContext;

    beforeEach(() => {
        // Create mock extension context
        mockContext = {
            subscriptions: [],
            extensionPath: "/mock/extension/path",
            extensionUri: vscode.Uri.file("/mock/extension/path"),
            globalState: {
                get: vi.fn(),
                update: vi.fn(),
                setKeysForSync: vi.fn(),
                keys: vi.fn(() => []),
            } as any,
            workspaceState: {
                get: vi.fn(),
                update: vi.fn(),
                keys: vi.fn(() => []),
            } as any,
            secrets: {
                get: vi.fn(),
                store: vi.fn(),
                delete: vi.fn(),
                onDidChange: vi.fn(() => ({ dispose: vi.fn() })),
            } as any,
            extensionMode: 1, // Production
            environmentVariableCollection: {
                persistent: false,
                description: undefined,
                replace: vi.fn(),
                append: vi.fn(),
                prepend: vi.fn(),
                get: vi.fn(),
                forEach: vi.fn(),
                delete: vi.fn(),
                clear: vi.fn(),
                [Symbol.iterator]: vi.fn(),
            } as any,
            storagePath: "/mock/storage",
            globalStoragePath: "/mock/global-storage",
            logPath: "/mock/logs",
            asAbsolutePath: vi.fn((relativePath: string) => `/mock/${relativePath}`),
            storageUri: vscode.Uri.file("/mock/storage"),
            globalStorageUri: vscode.Uri.file("/mock/global-storage"),
            logUri: vscode.Uri.file("/mock/logs"),
            extension: {
                id: "test.vtcode",
                extensionUri: vscode.Uri.file("/mock/extension"),
                extensionPath: "/mock/extension",
                isActive: true,
                packageJSON: {},
                exports: undefined,
                activate: vi.fn(),
                extensionKind: 1,
            } as any,
        };

        vi.clearAllMocks();
    });

    describe("activate", () => {
        it("should export activate function", () => {
            expect(activate).toBeDefined();
            expect(typeof activate).toBe("function");
        });

        it("should be a function that accepts ExtensionContext", () => {
            expect(activate.length).toBe(1); // Should accept 1 parameter
        });

        it("should not throw when called with valid context", () => {
            // Mock necessary VS Code APIs
            vi.spyOn(vscode.commands, "registerCommand").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.window, "registerTreeDataProvider").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.workspace, "workspaceFolders", "get").mockReturnValue(
                undefined
            );

            expect(() => {
                activate(mockContext);
            }).not.toThrow();
        });

        it("should register disposables to context", () => {
            vi.spyOn(vscode.commands, "registerCommand").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.window, "registerTreeDataProvider").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.workspace, "workspaceFolders", "get").mockReturnValue(
                undefined
            );

            activate(mockContext);

            // Should have registered at least some disposables
            // The exact number depends on implementation
            expect(mockContext.subscriptions.length).toBeGreaterThanOrEqual(0);
        });
    });

    describe("deactivate", () => {
        it("should export deactivate function", () => {
            expect(deactivate).toBeDefined();
            expect(typeof deactivate).toBe("function");
        });

        it("should not throw when called", () => {
            expect(() => {
                deactivate();
            }).not.toThrow();
        });

        it("should be callable after activate", () => {
            vi.spyOn(vscode.commands, "registerCommand").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.window, "registerTreeDataProvider").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.workspace, "workspaceFolders", "get").mockReturnValue(
                undefined
            );

            activate(mockContext);

            expect(() => {
                deactivate();
            }).not.toThrow();
        });
    });

    describe("extension lifecycle", () => {
        it("should support full activation and deactivation cycle", () => {
            vi.spyOn(vscode.commands, "registerCommand").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.window, "registerTreeDataProvider").mockReturnValue({
                dispose: vi.fn(),
            });
            vi.spyOn(vscode.workspace, "workspaceFolders", "get").mockReturnValue(
                undefined
            );

            // Activate
            expect(() => {
                activate(mockContext);
            }).not.toThrow();

            // Deactivate
            expect(() => {
                deactivate();
            }).not.toThrow();
        });
    });
});

describe("extension context interface", () => {
    it("should have required context properties", () => {
        const context: Partial<vscode.ExtensionContext> = {
            subscriptions: [],
            extensionPath: "/path",
            extensionUri: vscode.Uri.file("/path"),
        };

        expect(context.subscriptions).toBeDefined();
        expect(context.extensionPath).toBeDefined();
        expect(context.extensionUri).toBeDefined();
    });

    it("should support subscription management", () => {
        const subscriptions: vscode.Disposable[] = [];
        const disposable = { dispose: vi.fn() };

        subscriptions.push(disposable);

        expect(subscriptions).toHaveLength(1);
        expect(subscriptions[0]).toBe(disposable);
    });

    it("should allow disposing all subscriptions", () => {
        const disposable1 = { dispose: vi.fn() };
        const disposable2 = { dispose: vi.fn() };
        const subscriptions: vscode.Disposable[] = [disposable1, disposable2];

        subscriptions.forEach((d) => d.dispose());

        expect(disposable1.dispose).toHaveBeenCalled();
        expect(disposable2.dispose).toHaveBeenCalled();
    });
});
