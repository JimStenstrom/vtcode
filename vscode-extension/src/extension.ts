import { spawn, type SpawnOptionsWithoutStdio } from "node:child_process";
import * as vscode from "vscode";
import { ChatViewProvider } from "./chatView";
import { registerVtcodeLanguageFeatures } from "./languageFeatures";
import { VtcodeBackend } from "./vtcodeBackend";
import { CommandRegistry } from "./commandRegistry";
import { ParticipantRegistry } from "./participantRegistry";
import {
    AskCommand,
    AskSelectionCommand,
    AnalyzeCommand,
    UpdatePlanCommand,
    OpenConfigCommand,
    TrustWorkspaceCommand,
    RefreshCommand,
} from "./commands";
import {
    WorkspaceParticipant,
    CodeParticipant,
    TerminalParticipant,
    GitParticipant,
} from "./participants";
import {
    appendMcpProvider,
    loadConfigSummaryFromUri,
    pickVtcodeConfigUri,
    registerVtcodeConfigWatcher,
    revealMcpSection,
    revealToolsPolicySection,
    setHumanInTheLoop,
    setMcpProviderEnabled,
    VtcodeConfigSummary,
} from "./vtcodeConfig";

// Import all extracted modules
import { state } from "./core/stateManager";
import {
    refreshCliAvailability,
    ensureCliAvailableForCommand,
} from "./core/cliDetection";
import { setStatusBarChecking } from "./core/statusBar";
import {
    ensureWorkspaceTrustedForCommand,
    promptForWorkspaceTrustOnActivation,
    updateWorkspaceTrustState,
    initializeContextKeys,
} from "./services/trustService";
import { handleConfigUpdate } from "./services/configService";
import { launchAgentTerminal } from "./services/terminalService";
import {
    QuickActionTreeDataProvider,
    QuickActionItem,
} from "./providers/quickActionsProvider";
import { WorkspaceInsightsTreeDataProvider } from "./providers/workspaceInsightsProvider";
import { initializeIdeContextBridge } from "./providers/ideContextBridge";
import {
    createQuickActions,
    createWorkspaceInsights,
} from "./factories/uiFactories";
import {
    getConfiguredCommandPath,
    getWorkspaceRoot,
    getPrimaryWorkspaceFolder,
    getVtcodeEnvironment,
    createSpawnOptions,
    formatArgsForLogging,
    handleCommandError,
    getOutputChannel,
    setOutputChannel,
} from "./utils/processUtils";
import { appendIdeContextToPrompt } from "./utils/contextUtils";
import {
    ensureStableApi,
    logExtensionHostContext,
    openToolsPolicyGuide,
    openMcpGuide,
} from "./utils/extensionHelpers";
import {
    VT_CODE_CHAT_PARTICIPANT_ID,
    VT_CODE_UPDATE_PLAN_TOOL,
    VT_CODE_MCP_PROVIDER_ID,
    RunVtcodeCommandOptions,
    UpdatePlanToolInput,
    VtcodeTaskDefinition,
} from "./types/extensionTypes";

export function activate(context: vscode.ExtensionContext) {
    state.activationContext = context;

    // Initialize output channel
    const outputChannel = vscode.window.createOutputChannel("VTCode");
    state.outputChannel = outputChannel;
    setOutputChannel(outputChannel);
    context.subscriptions.push(outputChannel);

    // Initialize command registry with modular commands
    const commandRegistry = new CommandRegistry();
    commandRegistry.registerAll([
        new AskCommand(),
        new AskSelectionCommand(),
        new AnalyzeCommand(),
        new UpdatePlanCommand(),
        new OpenConfigCommand(),
        new TrustWorkspaceCommand(),
        new RefreshCommand(),
    ]);
    context.subscriptions.push(commandRegistry);

    // Initialize participant registry
    const participantRegistry = new ParticipantRegistry();
    participantRegistry.registerAll([
        new WorkspaceParticipant(),
        new CodeParticipant(),
        new TerminalParticipant(),
        new GitParticipant(),
    ]);
    context.subscriptions.push(participantRegistry);

    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    const chatBackend = new VtcodeBackend(
        getConfiguredCommandPath(),
        workspaceRoot,
        outputChannel
    );
    chatBackend.setEnvironmentProvider(() =>
        getVtcodeEnvironment({}, state.ideContextBridge?.filePath)
    );
    state.chatBackend = chatBackend;

    outputChannel.appendLine("[info] Registering chat view provider...");
    const chatViewProviderInstance = new ChatViewProvider(
        context.extensionUri,
        chatBackend,
        outputChannel,
        context
    );
    state.chatViewProvider = chatViewProviderInstance;

    chatViewProviderInstance.registerParticipants(participantRegistry);
    const viewId = "vtcodeChatView";
    outputChannel.appendLine(`[info] Registering view with ID: ${viewId}`);
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(viewId, chatViewProviderInstance)
    );
    outputChannel.appendLine("[info] Chat view provider registered successfully");

    chatViewProviderInstance.setWorkspaceTrusted(state.workspaceTrusted);

    ensureStableApi(context);
    logExtensionHostContext(context);
    registerVtcodeLanguageFeatures(context);

    void initializeIdeContextBridge(context).then((bridge) => {
        state.ideContextBridge = bridge;
    });

    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Left,
        100
    );
    statusBarItem.name = "VTCode Quick Actions";
    statusBarItem.accessibilityInformation = {
        role: "button",
        label: "Open VTCode quick actions or installation guide",
    };
    state.statusBarItem = statusBarItem;
    context.subscriptions.push(statusBarItem);

    const workspaceInsightsProvider = new WorkspaceInsightsTreeDataProvider(() =>
        createWorkspaceInsights(
            state.workspaceTrusted,
            state.cliAvailable,
            state.currentConfigSummary
        )
    );
    state.workspaceInsightsProvider = workspaceInsightsProvider;
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider(
            "vtcodeWorkspaceStatusView",
            workspaceInsightsProvider
        )
    );

    const quickActionsProviderInstance = new QuickActionTreeDataProvider(() =>
        createQuickActions(
            state.cliAvailable,
            state.currentConfigSummary,
            state.workspaceTrusted
        )
    );
    state.quickActionsProvider = quickActionsProviderInstance;
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider(
            "vtcodeQuickActionsView",
            quickActionsProviderInstance
        )
    );

    initializeContextKeys();

    void registerVtcodeConfigWatcher(context, handleConfigUpdate);

    updateWorkspaceTrustState(state.workspaceTrusted);
    void promptForWorkspaceTrustOnActivation(context);

    if (state.workspaceTrusted) {
        setStatusBarChecking(getConfiguredCommandPath());
    }

    context.subscriptions.push(
        vscode.workspace.onDidGrantWorkspaceTrust(async () => {
            updateWorkspaceTrustState(true);
            setStatusBarChecking(getConfiguredCommandPath());
            await refreshCliAvailability("manual");
        })
    );

    const workspaceFolderWatcher = vscode.workspace.onDidChangeWorkspaceFolders(() => {
        state.quickActionsProvider?.refresh();
        state.workspaceInsightsProvider?.refresh();
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        state.chatBackend?.updateConfiguration(
            getConfiguredCommandPath(),
            workspaceRoot,
            state.currentConfigSummary
        );
        void refreshCliAvailability("manual");
    });

    if (vscode.env.uiKind === vscode.UIKind.Web) {
        void vscode.window.showWarningMessage(
            "VTCode Companion is running in VS Code for the Web. Command execution features are disabled, but documentation and configuration helpers remain available."
        );
    }

    context.subscriptions.push(
        vscode.commands.registerCommand("vtcode.openQuickActions", async () => {
            const actions = createQuickActions(
                state.cliAvailable,
                state.currentConfigSummary,
                state.workspaceTrusted
            );
            const pickItems: QuickActionItem[] = actions.map((action) => ({
                label: `${action.icon ? `$(${action.icon}) ` : ""}${action.label}`,
                description: action.description,
                run: () =>
                    vscode.commands.executeCommand(action.command, ...(action.args ?? [])),
            }));

            const selection = await vscode.window.showQuickPick(pickItems, {
                placeHolder: "Choose a VTCode action to run",
            });

            if (selection) {
                await selection.run();
            }
        })
    );

    const verifyWorkspaceTrustCommand = vscode.commands.registerCommand(
        "vtcode.verifyWorkspaceTrust",
        async () => {
            if (vscode.workspace.isTrusted) {
                void vscode.window.showInformationMessage(
                    "Workspace is trusted. VTCode chat prompts will avoid the trust modal while tool executions still require approval."
                );
            } else {
                void vscode.window.showWarningMessage(
                    "Workspace is not trusted. VTCode requires a trusted workspace to execute commands and access tools."
                );
            }

            const outputChannel = getOutputChannel();
            outputChannel.appendLine(
                `[info] Workspace trust status: ${vscode.workspace.isTrusted ? "trusted" : "not trusted"}`
            );

            if (!vscode.workspace.isTrusted) {
                const selection = await vscode.window.showInformationMessage(
                    "Trust is required for VTCode. Would you like to manage workspace trust settings?",
                    "Manage Trust Settings",
                    "Learn More"
                );

                if (selection === "Manage Trust Settings") {
                    await vscode.commands.executeCommand("workbench.action.manageTrust");
                } else if (selection === "Learn More") {
                    await vscode.env.openExternal(
                        vscode.Uri.parse(
                            "https://code.visualstudio.com/docs/editor/workspace-trust"
                        )
                    );
                }
            }
        }
    );

    const flushIdeContextCommand = vscode.commands.registerCommand(
        "vtcode.flushIdeContextSnapshot",
        async () => {
            if (!state.ideContextBridge) {
                return false;
            }
            await state.ideContextBridge.flush();
            return true;
        }
    );

    const openDocumentation = vscode.commands.registerCommand(
        "vtcode.openDocumentation",
        async () => {
            await vscode.env.openExternal(
                vscode.Uri.parse("https://github.com/vinhnx/vtcode#readme")
            );
        }
    );

    const openDeepWiki = vscode.commands.registerCommand("vtcode.openDeepWiki", async () => {
        await vscode.env.openExternal(
            vscode.Uri.parse("https://deepwiki.com/vinhnx/vtcode")
        );
    });

    const openWalkthrough = vscode.commands.registerCommand(
        "vtcode.openWalkthrough",
        async () => {
            await vscode.commands.executeCommand(
                "workbench.action.openWalkthrough",
                "vtcode.walkthrough"
            );
        }
    );

    const openInstallGuide = vscode.commands.registerCommand(
        "vtcode.openInstallGuide",
        async () => {
            await vscode.env.openExternal(
                vscode.Uri.parse("https://github.com/vinhnx/vtcode#installation")
            );
        }
    );

    const openChatCommand = vscode.commands.registerCommand("vtcode.openChat", async () => {
        if (!(await ensureWorkspaceTrustedForCommand("chat with VTCode in VS Code"))) {
            return;
        }

        try {
            await vscode.commands.executeCommand("workbench.view.extension.vtcode");
        } catch {
            // If the container is unavailable we still try to focus the view directly.
        }

        try {
            await vscode.commands.executeCommand(`${ChatViewProvider.viewId}.focus`);
        } catch {
            void vscode.window.showWarningMessage(
                "The VTCode Chat view is unavailable in this environment."
            );
            return;
        }

        if (!state.chatLaunchHintShown) {
            state.chatLaunchHintShown = true;
            void vscode.window.showInformationMessage(
                "Start a conversation with VTCode using the chat panel."
            );
        }
    });

    const toggleHumanInTheLoopCommand = vscode.commands.registerCommand(
        "vtcode.toggleHumanInTheLoop",
        async () => {
            if (
                !(await ensureWorkspaceTrustedForCommand(
                    "change VTCode human-in-the-loop settings"
                ))
            ) {
                return;
            }

            try {
                const configUri = await pickVtcodeConfigUri(state.currentConfigSummary?.uri);
                if (!configUri) {
                    void vscode.window.showWarningMessage(
                        "No vtcode.toml file was found in this workspace."
                    );
                    return;
                }

                const activeSummary =
                    state.currentConfigSummary &&
                    state.currentConfigSummary.uri?.toString() === configUri.toString()
                        ? state.currentConfigSummary
                        : await loadConfigSummaryFromUri(configUri);

                const newValue = activeSummary.humanInTheLoop === false;
                const updated = await setHumanInTheLoop(configUri, newValue);
                if (!updated) {
                    void vscode.window.showWarningMessage(
                        "Failed to update human_in_the_loop in vtcode.toml."
                    );
                    return;
                }

                const relativePath = vscode.workspace.asRelativePath(configUri, false);
                const channel = getOutputChannel();
                channel.appendLine(
                    `[info] human_in_the_loop set to ${newValue} in ${relativePath}.`
                );
                void vscode.window.showInformationMessage(
                    `Human-in-the-loop safeguards are now ${newValue ? "enabled" : "disabled"} in vtcode.toml.`
                );
            } catch (error) {
                handleCommandError("toggle human-in-the-loop mode", error);
            }
        }
    );

    const openToolsPolicyGuideCommand = vscode.commands.registerCommand(
        "vtcode.openToolsPolicyGuide",
        async () => {
            try {
                await openToolsPolicyGuide();
            } catch (error) {
                handleCommandError("open tool policy guide", error);
            }
        }
    );

    const openToolsPolicyConfigCommand = vscode.commands.registerCommand(
        "vtcode.openToolsPolicyConfig",
        async () => {
            try {
                const configUri = await pickVtcodeConfigUri(state.currentConfigSummary?.uri);
                if (!configUri) {
                    void vscode.window.showWarningMessage(
                        "No vtcode.toml file was found in this workspace."
                    );
                    return;
                }

                await revealToolsPolicySection(configUri);
            } catch (error) {
                handleCommandError("open tool policy configuration", error);
            }
        }
    );

    const configureMcpProvidersCommand = vscode.commands.registerCommand(
        "vtcode.configureMcpProviders",
        async () => {
            if (
                !(await ensureWorkspaceTrustedForCommand("edit VTCode MCP provider settings"))
            ) {
                return;
            }

            try {
                const configUri = await pickVtcodeConfigUri(state.currentConfigSummary?.uri);
                if (!configUri) {
                    void vscode.window.showWarningMessage(
                        "No vtcode.toml file was found in this workspace."
                    );
                    return;
                }

                const activeSummary =
                    state.currentConfigSummary &&
                    state.currentConfigSummary.uri?.toString() === configUri.toString()
                        ? state.currentConfigSummary
                        : await loadConfigSummaryFromUri(configUri);

                const providers = activeSummary.mcpProviders;
                const enabledCount = providers.filter(
                    (provider) => provider.enabled !== false
                ).length;

                const quickItems: Array<
                    vscode.QuickPickItem & {
                        action: "toggle" | "add" | "guide" | "open";
                        providerName?: string;
                    }
                > = providers.map((provider) => ({
                    label: `${provider.enabled === false ? "$(circle-slash)" : "$(check)"} ${provider.name}`,
                    description: provider.command ?? "No command configured",
                    detail:
                        provider.args && provider.args.length > 0
                            ? `Args: ${provider.args.join(" ")}`
                            : provider.enabled === false
                            ? "Provider disabled"
                            : undefined,
                    action: "toggle",
                    providerName: provider.name,
                }));

                quickItems.push(
                    {
                        label: "$(add) Add MCP provider",
                        description: "Define a new Model Context Protocol provider entry.",
                        action: "add",
                    },
                    {
                        label: "$(gear) Open MCP configuration",
                        description: "Edit the MCP section in vtcode.toml.",
                        action: "open",
                    },
                    {
                        label: "$(book) Open MCP integration guide",
                        description: "Read the VTCode MCP configuration walkthrough.",
                        action: "guide",
                    }
                );

                const selection = await vscode.window.showQuickPick(quickItems, {
                    placeHolder:
                        providers.length > 0
                            ? `Manage ${providers.length} MCP provider${providers.length === 1 ? "" : "s"} (${enabledCount} enabled)`
                            : "No MCP providers defined. Add one to enable external tools.",
                });

                if (!selection) {
                    return;
                }

                switch (selection.action) {
                    case "toggle": {
                        if (!selection.providerName) {
                            return;
                        }

                        const provider = providers.find(
                            (candidate) => candidate.name === selection.providerName
                        );
                        if (!provider) {
                            void vscode.window.showWarningMessage(
                                `Provider "${selection.providerName}" is no longer available.`
                            );
                            return;
                        }

                        const newState = provider.enabled === false;
                        const result = await setMcpProviderEnabled(
                            configUri,
                            selection.providerName,
                            newState
                        );
                        if (result === "notfound") {
                            void vscode.window.showWarningMessage(
                                `Provider "${selection.providerName}" was not found in vtcode.toml.`
                            );
                            return;
                        }

                        if (result === "updated") {
                            const channel = getOutputChannel();
                            const relativePath = vscode.workspace.asRelativePath(
                                configUri,
                                false
                            );
                            channel.appendLine(
                                `[info] MCP provider "${selection.providerName}" enabled=${newState} in ${relativePath}.`
                            );
                            void vscode.window.showInformationMessage(
                                `MCP provider "${selection.providerName}" is now ${newState ? "enabled" : "disabled"}.`
                            );
                        }
                        break;
                    }
                    case "add": {
                        const name = await vscode.window.showInputBox({
                            prompt: "Provider name",
                            ignoreFocusOut: true,
                        });

                        if (!name || !name.trim()) {
                            return;
                        }

                        if (
                            providers.some(
                                (provider) =>
                                    provider.name.toLowerCase() === name.trim().toLowerCase()
                            )
                        ) {
                            void vscode.window.showWarningMessage(
                                `An MCP provider named "${name.trim()}" already exists.`
                            );
                            return;
                        }

                        const command = await vscode.window.showInputBox({
                            prompt: "Command used to launch the provider",
                            value: "uvx",
                            ignoreFocusOut: true,
                        });

                        if (!command || !command.trim()) {
                            return;
                        }

                        const argsInput = await vscode.window.showInputBox({
                            prompt: "Arguments (separate with spaces, leave blank for none)",
                            ignoreFocusOut: true,
                        });

                        const args = argsInput
                            ? argsInput
                                  .split(" ")
                                  .map((value) => value.trim())
                                  .filter((value) => value.length > 0)
                            : [];

                        const enableChoice = await vscode.window.showQuickPick(
                            ["Enable provider", "Keep disabled"],
                            {
                                placeHolder: "Should the provider start enabled?",
                            }
                        );

                        if (!enableChoice) {
                            return;
                        }

                        const appended = await appendMcpProvider(configUri, {
                            name: name.trim(),
                            command: command.trim(),
                            args,
                            enabled: enableChoice === "Enable provider",
                        });

                        if (appended) {
                            const channel = getOutputChannel();
                            const relativePath = vscode.workspace.asRelativePath(
                                configUri,
                                false
                            );
                            channel.appendLine(
                                `[info] Added MCP provider "${name.trim()}" to ${relativePath}.`
                            );
                            void vscode.window.showInformationMessage(
                                `Added MCP provider "${name.trim()}" to vtcode.toml.`
                            );
                        } else {
                            void vscode.window.showWarningMessage(
                                `Provider "${name.trim()}" already exists in vtcode.toml.`
                            );
                        }
                        break;
                    }
                    case "guide": {
                        await openMcpGuide();
                        break;
                    }
                    case "open": {
                        await revealMcpSection(configUri);
                        break;
                    }
                }
            } catch (error) {
                handleCommandError("configure MCP providers", error);
            }
        }
    );

    const launchAgentTerminalCommand = vscode.commands.registerCommand(
        "vtcode.launchAgentTerminal",
        async () => {
            if (!(await ensureCliAvailableForCommand())) {
                return;
            }

            const cwd = getWorkspaceRoot();
            await launchAgentTerminal(cwd);
        }
    );

    const taskProvider = vscode.tasks.registerTaskProvider("vtcode", {
        provideTasks: provideVtcodeTasks,
        resolveTask: resolveVtcodeTask,
    });

    const configurationWatcher = vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("vtcode.commandPath")) {
            void refreshCliAvailability("configuration");
        }
    });

    context.subscriptions.push(
        verifyWorkspaceTrustCommand,
        flushIdeContextCommand,
        openDocumentation,
        openDeepWiki,
        openWalkthrough,
        openInstallGuide,
        openChatCommand,
        toggleHumanInTheLoopCommand,
        openToolsPolicyGuideCommand,
        openToolsPolicyConfigCommand,
        configureMcpProvidersCommand,
        launchAgentTerminalCommand,
        taskProvider,
        configurationWatcher,
        workspaceFolderWatcher
    );

    registerVtcodeAiIntegrations(context);

    void refreshCliAvailability("activation");
}

export function deactivate() {
    state.dispose();
}

// Task provider functions (TODO: Extract to separate module in next iteration)
async function provideVtcodeTasks(): Promise<vscode.Task[]> {
    if (vscode.env.uiKind === vscode.UIKind.Web) {
        return [];
    }

    if (!state.workspaceTrusted) {
        return [];
    }

    const folder = getPrimaryWorkspaceFolder();
    if (!folder) {
        return [];
    }

    if (state.ideContextBridge) {
        await state.ideContextBridge.flush();
    }

    const definition: VtcodeTaskDefinition = {
        type: "vtcode",
        command: "update-plan",
        label: "Update plan with VTCode",
    };

    return [createUpdatePlanTask(folder, definition)];
}

function resolveVtcodeTask(task: vscode.Task): vscode.Task | undefined {
    const definition = task.definition as VtcodeTaskDefinition;
    if (definition.command !== "update-plan") {
        return undefined;
    }

    const scope = task.scope;
    let folder: vscode.WorkspaceFolder | undefined;
    if (scope && typeof scope !== "number") {
        folder = scope as vscode.WorkspaceFolder;
    } else {
        folder = getPrimaryWorkspaceFolder();
    }

    if (!folder) {
        return undefined;
    }

    state.ideContextBridge?.scheduleRefresh();

    return createUpdatePlanTask(folder, definition);
}

function createUpdatePlanTask(
    folder: vscode.WorkspaceFolder,
    definition: VtcodeTaskDefinition
): vscode.Task {
    const resolvedDefinition: VtcodeTaskDefinition = {
        type: "vtcode",
        command: "update-plan",
        summary: definition.summary,
        steps: definition.steps,
        label: definition.label,
    };

    const label = definition.label ?? "Update plan with VTCode";
    const prompt = buildUpdatePlanPrompt(resolvedDefinition);
    const args = [...getConfigArguments(), "exec", prompt];

    const execution = new vscode.ProcessExecution(getConfiguredCommandPath(), args, {
        cwd: folder.uri.fsPath,
        env: getVtcodeEnvironment({}, state.ideContextBridge?.filePath),
    });

    const task = new vscode.Task(resolvedDefinition, folder, label, "VTCode", execution);
    task.detail =
        "Runs `vtcode exec` to synchronize the workspace TODO plan via the update_plan tool.";
    task.presentationOptions = {
        reveal: vscode.TaskRevealKind.Always,
        echo: true,
        focus: false,
        panel: vscode.TaskPanelKind.Shared,
    };

    return task;
}

function buildUpdatePlanPrompt(definition: VtcodeTaskDefinition): string {
    const summary =
        definition.summary?.trim() ??
        "Refresh the current TODO plan using the VTCode update_plan tool.";
    const steps = (definition.steps ?? [])
        .map((step) => step.trim())
        .filter((step) => step.length > 0);

    const lines = [
        summary,
        "Use the update_plan tool to synchronize tasks and mark step status accurately.",
    ];

    if (steps.length > 0) {
        lines.push("", "Plan inputs:");
        steps.forEach((step, index) => {
            lines.push(`${index + 1}. ${step}`);
        });
    }

    return lines.join("\n");
}

function getConfigArguments(): string[] {
    const uri = state.currentConfigSummary?.uri;
    if (!uri) {
        return [];
    }

    return ["--config", uri.fsPath];
}

// Command execution (TODO: Extract to separate module in next iteration)
async function runVtcodeCommand(
    args: string[],
    options: RunVtcodeCommandOptions = {}
): Promise<void> {
    if (vscode.env.uiKind === vscode.UIKind.Web) {
        throw new Error(
            "VTCode commands that spawn the CLI are not available in the web extension host."
        );
    }

    if (!state.workspaceTrusted) {
        throw new Error("Trust this workspace to run VTCode CLI commands from VS Code.");
    }

    const commandPath = getConfiguredCommandPath();
    const cwd = getWorkspaceRoot();
    if (!cwd) {
        throw new Error("Open a workspace folder before running VTCode commands.");
    }

    if (state.ideContextBridge) {
        await state.ideContextBridge.flush();
    }

    if (options.cancellationToken?.isCancellationRequested) {
        throw new vscode.CancellationError();
    }

    const channel = getOutputChannel();
    const configArgs = getConfigArguments();
    const normalizedArgs = args.map((arg) => String(arg));
    const finalArgs = [...configArgs, ...normalizedArgs];
    const displayArgs = formatArgsForLogging(finalArgs);
    const revealOutput = options.revealOutput ?? true;

    if (revealOutput) {
        channel.show(true);
    }
    channel.appendLine(`$ ${commandPath} ${displayArgs}`);

    const runCommand = async () =>
        new Promise<void>((resolve, reject) => {
            const child = spawn(
                commandPath,
                finalArgs,
                createSpawnOptions({ cwd }, state.ideContextBridge?.filePath)
            );

            let cancellationRegistration: vscode.Disposable | undefined;
            let cancelled = false;
            if (options.cancellationToken) {
                cancellationRegistration = options.cancellationToken.onCancellationRequested(
                    () => {
                        cancelled = true;
                        if (!child.killed) {
                            child.kill();
                        }
                    }
                );
            }

            child.stdout.on("data", (data: Buffer) => {
                const text = data.toString();
                channel.append(text);
                options.onStdout?.(text);
            });

            child.stderr.on("data", (data: Buffer) => {
                const text = data.toString();
                channel.append(text);
                options.onStderr?.(text);
            });

            child.on("error", (error: Error) => {
                cancellationRegistration?.dispose();
                reject(error);
            });

            child.on("close", (code) => {
                cancellationRegistration?.dispose();
                if (cancelled) {
                    reject(new vscode.CancellationError());
                    return;
                }

                if (code === 0) {
                    resolve();
                } else {
                    reject(new Error(`VTCode exited with code ${code ?? "unknown"}`));
                }
            });
        });

    if (options.showProgress === false) {
        await runCommand();
        return;
    }

    await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: options.title ?? "Running VTCode…",
        },
        runCommand
    );
}

// AI Integrations (TODO: Extract to separate module in next iteration)
function registerVtcodeAiIntegrations(context: vscode.ExtensionContext): void {
    context.subscriptions.push({
        dispose: () => {
            // Event emitter is managed by state manager
        },
    });

    if ("lm" in vscode && typeof vscode.lm?.registerTool === "function") {
        const toolDisposable = vscode.lm.registerTool<UpdatePlanToolInput>(
            VT_CODE_UPDATE_PLAN_TOOL,
            {
                prepareInvocation: async (options) => {
                    const summary = options.input.summary?.trim();
                    const invocationMessage = summary
                        ? `Updating VTCode plan: ${summary}`
                        : "Updating VTCode plan with vtcode exec.";
                    return { invocationMessage };
                },
                invoke: async (options, token) => {
                    if (vscode.env.uiKind === vscode.UIKind.Web) {
                        throw new Error(
                            "VTCode CLI commands are not available in VS Code for the Web."
                        );
                    }

                    if (!state.workspaceTrusted) {
                        throw new Error(
                            "Trust this workspace to allow VTCode to update the TODO plan."
                        );
                    }

                    await refreshCliAvailability("manual");
                    if (!state.cliAvailable) {
                        const commandPath = getConfiguredCommandPath();
                        throw new Error(
                            `The VTCode CLI ("${commandPath}") is not available. Install the CLI or update the vtcode.commandPath setting.`
                        );
                    }

                    const input = options.input ?? {};
                    const summary =
                        typeof input.summary === "string" ? input.summary.trim() : undefined;
                    const steps = Array.isArray(input.steps)
                        ? input.steps
                              .map((value) => String(value).trim())
                              .filter((value) => value.length > 0)
                        : undefined;

                    const definition: VtcodeTaskDefinition = {
                        type: "vtcode",
                        command: "update-plan",
                        summary,
                        steps,
                    };
                    const prompt = buildUpdatePlanPrompt(definition);
                    const outputChunks: string[] = [];

                    try {
                        await runVtcodeCommand(["exec", prompt], {
                            title: "Updating VTCode plan…",
                            revealOutput: false,
                            showProgress: false,
                            cancellationToken: token,
                            onStdout: (text) => outputChunks.push(text),
                            onStderr: (text) => outputChunks.push(text),
                        });
                    } catch (error) {
                        if (error instanceof vscode.CancellationError) {
                            throw error;
                        }

                        throw error;
                    }

                    const combined = outputChunks.join("");
                    const normalized = combined.replace(/\r\n/g, "\n").trim();
                    const content =
                        normalized.length > 0
                            ? `\`\`\`\n${normalized}\n\`\`\``
                            : "VTCode completed the update_plan request but did not emit any output.";

                    return {
                        content: [new vscode.LanguageModelTextPart(content)],
                    };
                },
            }
        );
        context.subscriptions.push(toolDisposable);
    }

    if (
        "lm" in vscode &&
        typeof vscode.lm?.registerMcpServerDefinitionProvider === "function"
    ) {
        const mcpProvider: vscode.McpServerDefinitionProvider<vscode.McpStdioServerDefinition> =
            {
                onDidChangeMcpServerDefinitions: state.mcpDefinitionsChanged,
                provideMcpServerDefinitions: async () => {
                    if (!state.workspaceTrusted) {
                        return [];
                    }

                    const providers = state.currentConfigSummary?.mcpProviders ?? [];
                    return providers
                        .filter((provider) => provider.command)
                        .map(
                            (provider) =>
                                new vscode.McpStdioServerDefinition(
                                    provider.name,
                                    provider.command ?? "",
                                    provider.args ?? []
                                )
                        );
                },
                resolveMcpServerDefinition: async (server) => {
                    if (!state.workspaceTrusted) {
                        throw new Error(
                            "Trust this workspace before starting VTCode MCP servers."
                        );
                    }

                    return server;
                },
            };

        const disposable = vscode.lm.registerMcpServerDefinitionProvider(
            VT_CODE_MCP_PROVIDER_ID,
            mcpProvider
        );
        context.subscriptions.push(disposable);
    }

    if ("chat" in vscode && typeof vscode.chat?.createChatParticipant === "function") {
        const participant = vscode.chat.createChatParticipant(
            VT_CODE_CHAT_PARTICIPANT_ID,
            async (request, _context, response, token) => {
                const basePrompt = request.prompt.trim();
                if (!basePrompt) {
                    response.markdown(
                        "Ask a question or describe a task for the VTCode agent to begin."
                    );
                    return;
                }

                if (vscode.env.uiKind === vscode.UIKind.Web) {
                    const message =
                        "The VTCode CLI is not available in VS Code for the Web. Open a desktop workspace to chat with the CLI-driven agent.";
                    response.markdown(message);
                    return {
                        errorDetails: { message },
                    };
                }

                if (!state.workspaceTrusted) {
                    const message =
                        "Trust this workspace to allow VTCode to run CLI commands from chat.";
                    response.markdown(message);
                    return {
                        errorDetails: { message },
                    };
                }

                await refreshCliAvailability("manual");
                if (!state.cliAvailable) {
                    const commandPath = getConfiguredCommandPath();
                    const message = `The VTCode CLI (\`${commandPath}\`) is not available. Install it or update the \`vtcode.commandPath\` setting.`;
                    response.markdown(message);
                    return {
                        errorDetails: { message },
                    };
                }

                const promptWithContext = await appendIdeContextToPrompt(basePrompt, {
                    includeActiveEditor: true,
                    chatRequest: request,
                    cancellationToken: token,
                });

                response.progress("Running \`vtcode ask\`…");

                const collected: string[] = [];
                try {
                    await runVtcodeCommand(["ask", promptWithContext], {
                        title: "Asking VTCode…",
                        revealOutput: false,
                        showProgress: false,
                        cancellationToken: token,
                        onStdout: (text) => collected.push(text),
                        onStderr: (text) => collected.push(text),
                    });
                } catch (error) {
                    if (error instanceof vscode.CancellationError) {
                        response.progress("VTCode chat request cancelled.");
                        return;
                    }

                    const message = error instanceof Error ? error.message : String(error);
                    response.markdown(
                        `VTCode encountered an error while running the CLI: ${message}`
                    );
                    return {
                        errorDetails: { message },
                    };
                }

                const combined = collected.join("");
                const normalized = combined.replace(/\r\n/g, "\n").trim();
                if (normalized.length > 0) {
                    response.markdown(`\`\`\`\n${normalized}\n\`\`\``);
                } else {
                    response.markdown(
                        "VTCode completed the request but did not emit any output."
                    );
                }

                return {
                    metadata: {
                        command: "ask",
                    },
                };
            }
        );
        participant.iconPath = new vscode.ThemeIcon("rocket");
        participant.followupProvider = {
            provideFollowups: async () => [
                {
                    prompt: "Summarize the current TODO plan.",
                    label: "Summarize TODO plan",
                },
                {
                    prompt:
                        "Review configured MCP providers and highlight anything that needs attention.",
                    label: "Audit MCP providers",
                },
                {
                    prompt:
                        "Suggest the next high-priority tasks VTCode should tackle in this workspace.",
                    label: "Suggest next tasks",
                },
            ],
        };
        context.subscriptions.push(participant);
    }
}
