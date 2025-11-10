import * as vscode from "vscode";
import { QuickActionDescription, WorkspaceInsightDescription } from "../types/extensionTypes";
import { VtcodeConfigSummary } from "../vtcodeConfig";
import { getConfiguredCommandPath } from "../utils/processUtils";

/**
 * Create quick actions based on current state
 */
export function createQuickActions(
    cliAvailableState: boolean,
    summary: VtcodeConfigSummary | undefined,
    trusted: boolean
): QuickActionDescription[] {
    const actions: QuickActionDescription[] = [];

    if (!trusted) {
        actions.push(
            {
                label: "Trust this workspace for VTCode",
                description:
                    "Grant workspace trust to enable VTCode automation and CLI access.",
                command: "vtcode.trustWorkspace",
                icon: "shield",
            },
            {
                label: "Verify workspace trust flow",
                description:
                    "Run the VTCode trust checklist so chat prompts stop requesting trust while tools stay gated.",
                command: "vtcode.verifyWorkspaceTrust",
                icon: "shield-check",
            },
            {
                label: "Review VTCode CLI requirements",
                description:
                    "Learn how the VTCode CLI integrates once the workspace is trusted.",
                command: "vtcode.openInstallGuide",
                icon: "tools",
            }
        );
    }

    if (trusted && cliAvailableState) {
        actions.push(
            {
                label: "Verify workspace trust flow",
                description:
                    "Confirm VTCode chat prompts avoid the trust modal while tool executions still require approval.",
                command: "vtcode.verifyWorkspaceTrust",
                icon: "shield-check",
            },
            {
                label: "Refresh IDE context snapshot",
                description:
                    "Force a new .vtcode/ide-context block so the agent sees your latest editor state.",
                command: "vtcode.flushIdeContextSnapshot",
                icon: "history",
            },
            {
                label: "Ask the VTCode agent…",
                description:
                    "Send a one-off question and stream the answer in VS Code.",
                command: "vtcode.askAgent",
                icon: "comment-discussion",
            },
            {
                label: "Open a VTCode chat session",
                description:
                    "Launch the VS Code Chat view and collaborate with the VTCode participant.",
                command: "vtcode.openChat",
                icon: "comment-quote",
            },
            {
                label: "Ask about highlighted selection",
                description:
                    "Right-click or trigger VTCode to explain the selected text.",
                command: "vtcode.askSelection",
                icon: "comment",
            },
            {
                label: "Update VTCode task plan",
                description:
                    "Run the predefined VS Code task that drives the update_plan tool.",
                command: "vtcode.runUpdatePlanTask",
                icon: "checklist",
            },
            {
                label: "Launch interactive VTCode terminal",
                description:
                    "Open an integrated terminal session running vtcode chat.",
                command: "vtcode.launchAgentTerminal",
                icon: "terminal",
            },
            {
                label: "Analyze workspace with VTCode",
                description:
                    "Run vtcode analyze and stream the report to the VTCode output channel.",
                command: "vtcode.runAnalyze",
                icon: "pulse",
            }
        );
    } else if (trusted) {
        actions.push({
            label: "Review VTCode CLI installation",
            description:
                "Open the VTCode CLI installation instructions required for automation.",
            command: "vtcode.openInstallGuide",
            icon: "tools",
        });
    }

    if (trusted && summary?.hasConfig) {
        if (summary.automationFullAutoEnabled === true) {
            actions.push({
                label: "Full-auto automation detected (blocked)",
                description:
                    "Open vtcode.toml to disable [automation.full_auto]; VS Code will not run autonomous tasks.",
                command: "vtcode.openConfig",
                icon: "shield-off",
            });
        }

        const hitlEnabled = summary.humanInTheLoop !== false;
        actions.push({
            label: hitlEnabled
                ? "Disable human-in-the-loop safeguards"
                : "Enable human-in-the-loop safeguards",
            description: hitlEnabled
                ? "Allow VTCode to automate tool execution without manual approval."
                : "Require confirmation before VTCode executes high-impact tools.",
            command: "vtcode.toggleHumanInTheLoop",
            icon: "shield",
        });

        const providerCount = summary.mcpProviders.length;
        const enabledCount = summary.mcpProviders.filter(
            (provider) => provider.enabled !== false
        ).length;
        actions.push({
            label:
                providerCount > 0
                    ? "Manage MCP providers"
                    : "Configure MCP providers",
            description:
                providerCount > 0
                    ? `Adjust ${enabledCount}/${providerCount} enabled Model Context Protocol providers.`
                    : "Connect VTCode to external Model Context Protocol tools.",
            command: "vtcode.configureMcpProviders",
            icon: "plug",
        });

        const toolPoliciesCount = summary.toolPoliciesCount ?? 0;
        actions.push({
            label: "Review tool policy configuration",
            description:
                toolPoliciesCount > 0
                    ? `Inspect ${toolPoliciesCount} explicit tool policy overrides.`
                    : "Define allow/prompt/deny rules for VTCode tools.",
            command: "vtcode.openToolsPolicyConfig",
            icon: "law",
        });
    }

    const toolGuideDescription =
        summary?.hasConfig && trusted
            ? "Read documentation covering VTCode tool governance and HITL flows."
            : "Learn how VTCode enforces tool governance and human-in-the-loop safeguards.";
    actions.push({
        label: "Open VTCode tool policy guide",
        description: toolGuideDescription,
        command: "vtcode.openToolsPolicyGuide",
        icon: "book",
    });

    const configDescription = summary?.uri
        ? `Open ${vscode.workspace.asRelativePath(
              summary.uri,
              false
          )} to adjust VTCode settings.`
        : "Jump directly to the workspace VTCode configuration file.";

    actions.push(
        {
            label: "Open vtcode.toml",
            description: configDescription,
            command: "vtcode.openConfig",
            icon: "gear",
        },
        {
            label: "View VTCode documentation",
            description: "Open the VTCode README in your browser.",
            command: "vtcode.openDocumentation",
            icon: "book",
        },
        {
            label: "Review VTCode DeepWiki overview",
            description: "Open the DeepWiki page for VTCode capabilities.",
            command: "vtcode.openDeepWiki",
            icon: "globe",
        },
        {
            label: "Explore the VTCode walkthrough",
            description:
                "Open the getting-started walkthrough to learn about VTCode features.",
            command: "vtcode.openWalkthrough",
            icon: "rocket",
        }
    );

    return actions;
}

/**
 * Create workspace insights based on current state
 */
export function createWorkspaceInsights(
    trusted: boolean,
    cliAvailableState: boolean,
    summary: VtcodeConfigSummary | undefined
): WorkspaceInsightDescription[] {
    const insights: WorkspaceInsightDescription[] = [];

    insights.push({
        label: trusted ? "Workspace trust granted" : "Workspace trust required",
        description: trusted
            ? "VTCode can run CLI automation in this workspace."
            : "Grant trust to enable VTCode CLI commands and automation features.",
        icon: trusted ? "shield" : "shield-off",
        command: trusted
            ? undefined
            : {
                  command: "vtcode.trustWorkspace",
                  title: "Trust Workspace for VTCode",
              },
        tooltip: trusted
            ? "Workspace trust allows VTCode to spawn CLI processes."
            : "Security-sensitive features are disabled until this workspace is trusted.",
    });

    if (!trusted) {
        insights.push({
            label: "CLI access blocked",
            description:
                "Trust the workspace to allow VTCode to detect and launch the CLI.",
            icon: "circle-slash",
            command: {
                command: "vtcode.openInstallGuide",
                title: "Review CLI Installation",
            },
        });
    } else {
        const commandPath = getConfiguredCommandPath();
        insights.push({
            label: cliAvailableState
                ? "VTCode CLI detected"
                : "VTCode CLI unavailable",
            description: cliAvailableState
                ? `Using ${commandPath}`
                : `Check ${commandPath} or adjust vtcode.commandPath`,
            icon: cliAvailableState ? "check" : "warning",
            command: cliAvailableState
                ? {
                      command: "vtcode.openQuickActions",
                      title: "Open Quick Actions",
                  }
                : {
                      command: "vtcode.openInstallGuide",
                      title: "Open Installation Guide",
                  },
            tooltip: createStatusBarTooltipText(
                commandPath,
                cliAvailableState,
                trusted
            ),
        });
    }

    if (summary?.hasConfig) {
        const configPath = summary.uri
            ? vscode.workspace.asRelativePath(summary.uri, false)
            : "vtcode.toml";
        insights.push({
            label: "VTCode configuration detected",
            description: configPath,
            icon: "gear",
            command: {
                command: "vtcode.openConfig",
                title: "Open vtcode.toml",
            },
        });

        if (summary.agentProvider) {
            const provider = summary.agentProvider;
            const defaultModel = summary.agentDefaultModel;
            const providerLower = provider.toLowerCase();
            const modelLower = defaultModel?.toLowerCase() ?? "";
            const mismatch =
                (providerLower === "ollama" &&
                    (defaultModel?.includes(":") ?? false)) ||
                (providerLower !== "openrouter" &&
                    modelLower.startsWith("gpt-oss:"));

            insights.push({
                label: `Agent provider: ${provider}`,
                description: defaultModel
                    ? `Default model: ${defaultModel}`
                    : "No default model configured",
                icon: mismatch ? "alert" : "globe",
                command: mismatch
                    ? {
                          command: "vtcode.openConfig",
                          title: "Review agent provider configuration",
                      }
                    : undefined,
                tooltip: mismatch
                    ? "Provider and default_model may require different credentials. Update vtcode.toml to avoid CLI failures."
                    : undefined,
            });
        }

        const fullAutoEnabled = summary.automationFullAutoEnabled === true;
        const allowedTools = summary.automationFullAutoAllowedTools;
        const automationDescription = fullAutoEnabled
            ? allowedTools && allowedTools.length > 0
                ? `Allowed tools: ${allowedTools.join(
                      ", "
                  )}. VS Code blocks autonomous execution; disable automation.full_auto to avoid warnings.`
                : "automation.full_auto is enabled. VS Code blocks autonomous execution; disable the setting to silence this warning."
            : "automation.full_auto is disabled. VTCode prompts require explicit approval.";
        insights.push({
            label: fullAutoEnabled
                ? "Full-auto automation detected (blocked)"
                : "Full-auto automation disabled",
            description: automationDescription,
            icon: fullAutoEnabled ? "shield-off" : "shield",
            command: fullAutoEnabled
                ? {
                      command: "vtcode.openConfig",
                      title: "Disable automation.full_auto",
                  }
                : undefined,
        });

        const hitlStatus =
            summary.humanInTheLoop === false
                ? "Disabled (manual approvals required)"
                : "Enabled";
        insights.push({
            label: "Human-in-the-loop safeguards",
            description: hitlStatus,
            icon: summary.humanInTheLoop === false ? "person" : "shield",
            command:
                trusted && summary.uri
                    ? {
                          command: "vtcode.toggleHumanInTheLoop",
                          title: "Toggle human-in-the-loop safeguards",
                      }
                    : undefined,
        });

        const providerCount = summary.mcpProviders.length;
        const enabledCount = summary.mcpProviders.filter(
            (provider) => provider.enabled !== false
        ).length;
        insights.push({
            label: "MCP providers",
            description:
                providerCount > 0
                    ? `${enabledCount}/${providerCount} enabled`
                    : "No providers configured",
            icon: "plug",
            command:
                trusted && summary.uri
                    ? {
                          command: "vtcode.configureMcpProviders",
                          title: "Configure MCP providers",
                      }
                    : undefined,
        });

        const toolPoliciesCount = summary.toolPoliciesCount ?? 0;
        const toolPolicyLabel = summary.toolDefaultPolicy
            ? `Default: ${summary.toolDefaultPolicy}`
            : "No default policy set";
        insights.push({
            label: "Tool policy coverage",
            description:
                toolPoliciesCount > 0
                    ? `${toolPoliciesCount} overrides · ${toolPolicyLabel}`
                    : `No overrides · ${toolPolicyLabel}`,
            icon: "law",
            command: {
                command: "vtcode.openToolsPolicyConfig",
                title: "Review tool policy configuration",
            },
        });

        if (summary.parseError) {
            insights.push({
                label: "Configuration parsing error",
                description: summary.parseError,
                icon: "error",
                command: summary.uri
                    ? {
                          command: "vtcode.openConfig",
                          title: "Open vtcode.toml",
                      }
                    : undefined,
            });
        }
    } else {
        insights.push({
            label: "No vtcode.toml detected",
            description:
                "Use VTCode: Open Configuration to create a workspace configuration.",
            icon: "file",
            command: {
                command: "vtcode.openConfig",
                title: "Create vtcode.toml",
            },
        });
    }

    return insights;
}

function createStatusBarTooltipText(
    commandPath: string,
    available: boolean,
    trusted: boolean
): string {
    return `VTCode CLI: ${commandPath} (${available ? "Available" : "Missing"})`;
}
