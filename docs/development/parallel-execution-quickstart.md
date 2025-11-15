# Phase 3 Parallel Compilation Fixes - Quick Start Guide

**Created:** 2025-11-14
**Purpose:** Launch 4 parallel Claude Code sessions to fix Phase 3 compilation errors in ~2 hours instead of 4-6 hours

---

## TL;DR - What We're Doing

We have ~22 compilation errors in the Phase 3 architecture transformation. Instead of fixing them sequentially (4-6 hours), we'll use **4 parallel Claude Code web sessions** to fix them in **~2 hours wall time** (66-75% faster).

**Each agent works on completely independent code** → **zero merge conflicts** → **massive time savings**

---

## The Setup

### 4 Agents, 4 Independent Tasks

| Agent | Task | File(s) | Time | Conflicts |
|-------|------|---------|------|-----------|
| **Agent 1** | Fix vtcode-llm-gemini | vtcode-llm-gemini/src/provider.rs | 1-1.5h | None |
| **Agent 2** | Fix vtcode-llm-openrouter | vtcode-llm-openrouter/src/provider.rs | 1-1.5h | None |
| **Agent 3** | Fix Microsoft provider | vtcode-core/src/llm/providers/microsoft.rs | 1-1.5h | None |
| **Agent 4** | Fix core imports | vtcode-core/src/lib.rs, runner.rs | 0.5-1h | None* |

\* Agent 4 waits for Agents 1 & 2 to finish (they add constructors Agent 4 needs)

---

## Quick Start (5 Steps)

### Step 1: Set Up Coordination (5 minutes)

**Option A:** GitHub Issue
```bash
# Create a tracking issue
gh issue create --title "Phase 3 Parallel Compilation Fixes" --body "4 agents working in parallel"
```

**Option B:** Discord/Slack Channel
- Create channel: `#phase3-parallel-fixes`
- Invite all agents

### Step 2: Launch 4 Claude Code Web Sessions (5 minutes)

Go to https://claude.com/code and open 4 tabs (or 4 browser windows):

1. **Tab 1** → Agent 1 (Gemini)
2. **Tab 2** → Agent 2 (OpenRouter)
3. **Tab 3** → Agent 3 (Microsoft)
4. **Tab 4** → Agent 4 (Core Imports)

**In each tab:**
```
Hi Claude! I'm Agent [1/2/3/4] working on Phase 3 parallel compilation fixes.

Please read my task brief:
[paste the corresponding prompt file content]

Ready to start!
```

**Prompt Files to Paste:**
- Agent 1: `docs/development/AGENT_PHASE3_1_GEMINI_PROMPT.md`
- Agent 2: `docs/development/AGENT_PHASE3_2_OPENROUTER_PROMPT.md`
- Agent 3: `docs/development/AGENT_PHASE3_3_MICROSOFT_PROMPT.md`
- Agent 4: `docs/development/AGENT_PHASE3_4_CORE_IMPORTS_PROMPT.md`

### Step 3: Start Wave 1 (Agents 1, 2, 3) Simultaneously

**In Agent 1, 2, 3 tabs (all at once):**
```
Let's start! Please begin working on your assigned tasks.
```

**Agent 4 waits!** They will start after Agents 1 & 2 post "COMPLETE" status.

### Step 4: Monitor Progress (every 30 minutes)

**Check coordination channel for status updates:**

```
Agent 1 Update (T+30min):
✅ Added with_model constructor
✅ Added from_config constructor
⏳ Verifying trait implementation
ETA: 30 minutes

Agent 2 Update (T+30min):
✅ Added with_model constructor
✅ Added from_config constructor
⏳ Testing reasoning handling
ETA: 30 minutes

Agent 3 Update (T+30min):
✅ Fixed Usage field names
✅ Added missing Message fields
⏳ Fixing ContentPart access
ETA: 45 minutes
```

### Step 5: Merge All Branches (20 minutes)

**After all agents post "COMPLETE":**

```bash
# Merge Agent 1 (Gemini)
git checkout claude/main
git merge phase3-fix-gemini
git push

# Merge Agent 2 (OpenRouter)
git merge phase3-fix-openrouter
git push

# Merge Agent 3 (Microsoft)
git merge phase3-fix-microsoft
git push

# Merge Agent 4 (Core Imports)
git merge phase3-fix-core-imports
git push

# Verify final build
cargo build --all
cargo test --all
```

---

## Success Criteria

All done when:

- [ ] ✅ All 4 agents posted "COMPLETE" status
- [ ] ✅ All 4 branches pushed to GitHub
- [ ] ✅ All branches merged to claude/main
- [ ] ✅ `cargo build --all` succeeds with zero errors
- [ ] ✅ No merge conflicts (guaranteed by design!)
- [ ] ✅ Tests pass

---

## Expected Timeline

```
T+0:00   Launch 4 Claude sessions
T+0:05   Paste task briefs into each session
T+0:10   Start Agents 1, 2, 3 (Agent 4 waits)
T+0:45   Checkpoint 1 (status updates)
T+1:30   Agents 1, 2, 3 post "COMPLETE"
T+1:35   Start Agent 4
T+2:00   Agent 4 posts "COMPLETE"
T+2:10   Merge all 4 branches
T+2:20   Run final build verification
T+2:25   DONE! ✅ 🎉

TOTAL: ~2.5 hours (vs 4-6 hours sequential)
SPEEDUP: 60-75% faster
```

---

## Troubleshooting

### Problem: Agent stuck or errored out

**Solution:**
- Check their error message
- Ask them to retry from the checkpoint
- If blocked, reassign subtasks to other agents

### Problem: Merge conflict

**This shouldn't happen!** The tasks are designed to touch different files.

**If it does:**
```bash
git merge --abort
# Ask agents which files they modified
# Manually review the conflict
git merge --no-commit phase3-fix-[branch]
# Resolve conflict
git commit
```

### Problem: Build still has errors after all merges

**Solution:**
```bash
# Check which errors remain
cargo build 2>&1 | tee build_errors.txt

# Identify which agent's area has errors
grep -i "gemini" build_errors.txt  # Agent 1
grep -i "openrouter" build_errors.txt  # Agent 2
grep -i "microsoft" build_errors.txt  # Agent 3
grep -i "module.*gemini" build_errors.txt  # Agent 4

# Ask the relevant agent to fix
```

---

## Communication Templates

### Agent Completion Message

```
Agent [1/2/3/4] COMPLETE:
✅ Task: [task name]
✅ Branch: phase3-fix-[name]
✅ Files modified: [list]
✅ Build status: cargo build -p [crate] succeeds
✅ Tests: all pass
✅ Pushed to GitHub: Yes

Ready for merge!
```

### Coordinator Merge Message

```
MERGE COMPLETE:
✅ Merged phase3-fix-gemini → claude/main
✅ Merged phase3-fix-openrouter → claude/main
✅ Merged phase3-fix-microsoft → claude/main
✅ Merged phase3-fix-core-imports → claude/main

Final build: cargo build --all
Result: ✅ SUCCESS (0 errors)

All agents: Thank you! 🎉
Phase 3 compilation fixes complete!
```

---

## Files Reference

**Strategy Document:**
- `docs/development/PHASE_3_PARALLEL_FIXES.md` - Overall strategy and coordination

**Agent Task Briefs:**
- `docs/development/AGENT_PHASE3_1_GEMINI_PROMPT.md` - Agent 1 detailed instructions
- `docs/development/AGENT_PHASE3_2_OPENROUTER_PROMPT.md` - Agent 2 detailed instructions
- `docs/development/AGENT_PHASE3_3_MICROSOFT_PROMPT.md` - Agent 3 detailed instructions
- `docs/development/AGENT_PHASE3_4_CORE_IMPORTS_PROMPT.md` - Agent 4 detailed instructions

**This File:**
- `docs/development/PARALLEL_EXECUTION_QUICKSTART.md` - You are here!

---

## Why This Works

**Zero Conflicts:**
- Agent 1 works on: `vtcode-llm-gemini/src/provider.rs`
- Agent 2 works on: `vtcode-llm-openrouter/src/provider.rs`
- Agent 3 works on: `vtcode-core/src/llm/providers/microsoft.rs`
- Agent 4 works on: `vtcode-core/src/lib.rs`, `vtcode-core/src/core/agent/runner.rs`

**No overlapping files** → **No merge conflicts** → **Clean parallel execution**

**Minimal Dependencies:**
- Agents 1, 2, 3 are completely independent (can run simultaneously)
- Agent 4 depends only on Agents 1 & 2 constructors (simple dependency)

**Clear Task Boundaries:**
- Each agent has detailed, step-by-step instructions
- Success criteria are explicit
- Checkpoints are built-in

---

## Next Steps

1. **Read this guide** ✅ (you're doing it!)
2. **Set up coordination channel** (GitHub issue or Discord/Slack)
3. **Open 4 Claude Code web sessions**
4. **Paste task briefs**
5. **Start Agents 1, 2, 3**
6. **Wait for completions**
7. **Start Agent 4**
8. **Merge all branches**
9. **Celebrate!** 🎉

---

**Ready to parallelize? Let's save 60-75% of the time! 🚀**
