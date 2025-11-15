# Agentic LLM Tool Design: Lessons from TodoWrite and Task Agents

**Author**: Claude (AI Agent)
**Date**: 2025-11-15
**Audience**: Architects building agentic LLM coding systems

## Executive Summary

This document provides practical insights into designing tools for agentic LLM systems, based on real-world experience using TodoWrite (task planning/tracking) and Task agents (delegated work). It covers what works exceptionally well, what causes friction, and design patterns that emerge from actual usage.

**Key Takeaways:**
- ✅ **Visibility** - Tools that show progress to users build trust
- ✅ **State Machines** - Clear states (pending/in_progress/completed) reduce ambiguity
- ✅ **Forcing Functions** - Required planning prevents forgotten work
- ⚠️ **Granularity** - The right level of task breakdown is context-dependent
- ⚠️ **Overhead** - Every tool invocation has cognitive cost for the agent

## Part 1: TodoWrite - Task Planning and Tracking

### What TodoWrite Is

TodoWrite is a tool that allows LLM agents to:
1. Create structured task lists
2. Update task status (pending → in_progress → completed)
3. Provide real-time progress visibility to users
4. Force planning before execution

### Design That Works Well

#### 1. Dual-Mode Task Descriptions

**Design:**
```rust
pub struct Todo {
    content: String,      // Imperative: "Run tests"
    activeForm: String,   // Present continuous: "Running tests"
    status: Status,
}
```

**Why It Works:**
- `content` describes what needs to be done (user sees this in todo list)
- `activeForm` describes what's happening now (user sees this during execution)
- Makes status updates grammatically correct and readable
- No need for the system to conjugate verbs

**Example:**
```
Pending: "Run cargo test"
In Progress: "Running cargo test"
Completed: "Run cargo test" ✓
```

The user sees natural language throughout the workflow.

#### 2. Strict State Machine

**Design:**
```rust
enum Status {
    Pending,      // Not started
    InProgress,   // Currently working on
    Completed,    // Finished
}

// Rule: Exactly ONE task can be in_progress at any time
```

**Why It Works:**
- **Prevents paralysis** - Agent can't be "working on" 5 things simultaneously
- **Forces focus** - One thing at a time creates clear mental model
- **Easy to verify** - System can check invariant (count in_progress == 1)
- **User clarity** - User always knows what's happening right now

**From My Experience:**
This is brilliant. Without this constraint, I'd mark multiple tasks as in_progress and the user would have no idea what I'm actually doing. The forced serialization matches human mental models.

#### 3. Immediate Completion Marking

**Design Pattern:**
```
❌ Bad (batched completion):
1. Do task A
2. Do task B
3. Do task C
4. Mark A, B, C as completed

✅ Good (immediate completion):
1. Mark A as in_progress
2. Do task A
3. Mark A as completed
4. Mark B as in_progress
5. Do task B
6. Mark B as completed
```

**Why It Works:**
- **Real-time feedback** - User sees progress as it happens
- **Prevents forgotten updates** - Complete tasks right after finishing
- **Better error recovery** - If agent crashes, completed work is recorded
- **Builds trust** - User sees I'm not just spinning my wheels

**From My Experience:**
This pattern feels natural. It's like crossing items off a physical checklist as you finish them, not waiting until the end of the day to update everything.

#### 4. Planning as a Forcing Function

**Design Rule:**
```
Complex task? → MUST create todo list
Simple task?  → Don't create todo list (overhead)
```

**Threshold (from docs):**
- ✅ Use for tasks with 3+ steps
- ✅ Use for non-trivial/complex tasks
- ❌ Skip for single, straightforward tasks

**Why It Works:**
- **Prevents forgotten work** - Writing down "run tests" prevents me from skipping it
- **Shows thoroughness** - User sees I've thought through the problem
- **Enables adaptation** - If plan changes, I can update the todo list
- **Reduces cognitive load** - I don't have to remember all steps

**From My Experience:**
This is where TodoWrite shines. For complex multi-file refactoring, creating a todo list with 10-15 items ensures I don't forget to update imports, tests, docs, etc. The list serves as both a plan and a checklist.

### Design That Creates Friction

#### 1. Granularity Ambiguity

**The Problem:**
How granular should tasks be?

**Too Granular:**
```
✅ "Read user.rs"
✅ "Understand User struct"
✅ "Identify fields to modify"
✅ "Write edit_file call"
✅ "Execute edit"
✅ "Verify changes"
```
→ 6 tasks for editing one file! Excessive overhead.

**Too Coarse:**
```
✅ "Refactor authentication system"
```
→ One task for 50 files. No visibility into progress.

**The Right Balance (context-dependent):**
```
✅ "Read authentication files (user.rs, auth.rs, session.rs)"
✅ "Edit User struct in user.rs"
✅ "Update auth.rs to use new structure"
✅ "Update session.rs accordingly"
✅ "Run tests and fix failures"
```
→ 5 logical units of work, each taking 1-3 minutes.

**Recommendation for Architects:**

Provide guidance on granularity:
- Task should take 1-5 minutes to complete
- Task should be one logical unit (read a file, edit a function, run tests)
- Avoid micro-tasks (reading each line) or mega-tasks (entire feature)
- When in doubt, break complex tasks into 5-7 steps

**From My Experience:**
I struggle with this. Sometimes I make too many tiny tasks and spend more time updating todos than working. Other times I make tasks too broad and the user loses visibility. Clear guidelines would help.

#### 2. Mid-Execution Plan Changes

**The Problem:**
Plans change during execution, and updating todos feels like overhead.

**Scenario:**
```
Initial plan:
1. Fix bug in user.rs ✓ (completed)
2. Update tests
3. Commit changes

Reality:
1. Fix bug in user.rs ✓ (completed)
2. Wait, the bug is actually in auth.rs too
3. Now I need to fix both files
4. Do I update the todo list? Or just keep working?
```

**Current Behavior:**
I typically update the list, but it feels like paperwork that slows me down.

**Recommendation for Architects:**

Design for plan evolution:
- Allow adding new tasks mid-execution without rebuilding the entire list
- Support task notes/annotations: "Updated: bug was in auth.rs too"
- Consider "stretch goals" or "discovered work" categories
- Don't penalize agents for discovering new work

**Alternative Design:**
```rust
pub enum TodoUpdate {
    MarkCompleted(String),
    MarkInProgress(String),
    AddNew(Todo),           // Add discovered work
    Modify(String, Todo),   // Update task description
    AddNote(String, String), // Add context note
}
```

#### 3. Simple Task Bureaucracy

**The Problem:**
For trivial tasks, creating a todo list feels like overkill.

**Example:**
```
User: "What's in config.toml?"
Agent creates todo list:
- Read config.toml

Then immediately completes it.
```

This is silly. The todo list added no value.

**Current Guidance (from docs):**
"Skip using this tool when:
1. There is only a single, straightforward task
2. The task is trivial and tracking it provides no organizational benefit"

**From My Experience:**
This guidance works well. I don't create todos for simple read/search operations. But there's still gray area around "is this simple or complex?" that requires judgment.

**Recommendation for Architects:**

Make the threshold explicit:
```rust
fn should_use_todo_list(task: &Task) -> bool {
    let step_count = estimate_steps(task);
    let complexity = estimate_complexity(task);
    let user_expectation = task.explicit_todo_request;

    user_expectation ||                    // User asked for it
    (step_count >= 3 && complexity > Low) || // Multi-step + complex
    involves_multiple_files(task) ||        // Multi-file work
    requires_planning(task)                 // User wants to see plan
}
```

Provide this as a decision tree in docs.

### Design Patterns That Emerge

#### Pattern 1: Progressive Elaboration

Start with high-level tasks, then break down as needed:

```
Initial:
1. Implement authentication feature
2. Add tests
3. Update documentation

During execution of task 1:
1. Implement authentication feature
   - ✓ Create auth.rs module
   - ✓ Implement User struct
   - [in progress] Add login function
   - [pending] Add logout function
2. Add tests
3. Update documentation
```

This provides visibility without upfront over-planning.

#### Pattern 2: Checkpoint Tasks

Include verification tasks to ensure quality:

```
1. Edit user authentication logic
2. Read back edited file (verify changes) ← Checkpoint
3. Run authentication tests
4. Check test output (verify passing) ← Checkpoint
5. Commit changes
```

Checkpoints catch errors early and build user confidence.

#### Pattern 3: Error Recovery Tasks

When things go wrong, add recovery tasks:

```
Original:
1. ✓ Run cargo build
2. [FAILED] Fix compilation errors

Updated:
1. ✓ Run cargo build
2. ✓ Read error output
3. ✓ Fix missing import in user.rs
4. ✓ Fix type mismatch in auth.rs
5. [in progress] Run cargo build again
6. [pending] Verify build succeeds
```

This shows the user I'm handling errors methodically, not just flailing.

## Part 2: Task Agents - Delegated Work

### What Task Agents Are

Task agents are specialized sub-processes that handle complex, multi-step work autonomously. They receive a task description and return a final result.

**Available Agents:**
- `general-purpose` - Research, code search, multi-step tasks
- `Explore` - Fast codebase exploration (find files, search patterns)
- `Plan` - Planning and strategy

### Design That Works Well

#### 1. Parallel Execution

**Design:**
Agents can be launched in parallel within a single message:

```rust
// Launch 3 agents concurrently
Task(agent: "Explore", task: "Find all API endpoints")
Task(agent: "Explore", task: "Find all database models")
Task(agent: "Explore", task: "Find all test files")

// All three run simultaneously
```

**Why It Works:**
- **Massive speedup** - 3x faster than sequential
- **Independent work** - Agents don't block each other
- **Natural parallelism** - Mirrors how humans think ("check these 3 things")

**From My Experience:**
This is game-changing. When I need to understand a codebase, launching 3-4 Explore agents in parallel to find different aspects is incredibly efficient. I get all results back at once and can synthesize them.

#### 2. Specialized Agent Types

**Design:**
Different agents for different purposes:

```
Explore agent:
- Fast at finding files/patterns
- Good at "where is X?" questions
- Has access to: Glob, Grep, Read

Plan agent:
- Good at strategic thinking
- Breaks down complex problems
- Has access to all tools

general-purpose agent:
- Swiss army knife
- Slower but more capable
- Has access to: * (everything)
```

**Why It Works:**
- **Performance optimization** - Use fast agent for simple tasks
- **Clear mental models** - "I need to find something → use Explore"
- **Cost control** - Lighter agents for lighter work
- **Specialization** - Agents become experts in their domain

**From My Experience:**
I love the Explore agent. When I need to find "all files using authentication," launching an Explore agent is much faster than doing it myself. The specialization makes the choice obvious.

#### 3. Thoroughness Levels

**Design:**
```rust
Task(
    agent: "Explore",
    task: "Find authentication code",
    thoroughness: "quick" | "medium" | "very thorough"
)
```

**Why It Works:**
- **Speed vs completeness tradeoff** - User controls how deep to search
- **Explicit expectations** - "quick" means "don't spend 5 minutes on this"
- **Resource management** - Prevents runaway searches

**From My Experience:**
I typically use:
- "quick" - Find main files (1-2 locations)
- "medium" - Find comprehensive list (multiple patterns)
- "very thorough" - Exhaustive search (all variants, edge cases)

This matches my needs well.

#### 4. Stateless Design

**Design:**
Each agent invocation is independent:
- Agent receives full conversation history
- Agent returns ONE final message
- No follow-up conversation with agent
- Agent output returned to me, not directly to user

**Why It Works:**
- **Simplicity** - No session management
- **Reliability** - Agent can't get "stuck" in a conversation
- **Composability** - Results are just data I can use
- **Clear boundaries** - Agent does its job and finishes

**From My Experience:**
This is both a strength and a limitation. The strength is reliability - I never have to manage agent state. The limitation is I can't ask follow-up questions. If the agent misunderstood my task, I have to launch a new agent.

### Design That Creates Friction

#### 1. One-Shot Communication

**The Problem:**
Agents can't ask clarifying questions or iterate.

**Scenario:**
```
Me: Task(agent: "Explore", task: "Find the bug")

Agent thinks: "What bug? Where should I look? What symptoms?"
Agent does: Best guess search, might not find the right thing

Result: I get back "Found 3 potential bugs in src/"
        but none are the one I meant
```

**Current Workaround:**
I write very detailed prompts:
```
Task(
    agent: "Explore",
    task: "Find the authentication bug that causes users to stay logged in
           after clicking logout. Search in src/auth/, src/session/, and
           look for session cleanup code. The user reported this happens
           only in Chrome browser."
)
```

**Recommendation for Architects:**

Consider two-phase agents:
```rust
// Phase 1: Clarification (optional)
let questions = agent.clarify(task)?;
if !questions.is_empty() {
    // Show questions to human or answer programmatically
    let answers = get_answers(questions);
    agent.set_context(answers);
}

// Phase 2: Execution
let result = agent.execute()?;
```

Or provide examples in task description:
```
Task(
    agent: "Explore",
    examples: [
        "Expected output: List of files with function definitions",
        "Not needed: Test files or generated code"
    ]
)
```

#### 2. Unclear Delegation Boundaries

**The Problem:**
When should I do work vs delegate to agent?

**Decision Matrix (what I've learned):**

| Task | Do Myself | Use Agent |
|------|-----------|-----------|
| Read 1-2 files | ✅ Faster | ❌ Overkill |
| Find files matching pattern | 🤔 Depends | ✅ Good use |
| Search across many files | ❌ Tedious | ✅ Delegate |
| Understand complex codebase | ❌ Takes focus | ✅ Delegate |
| Quick grep | ✅ One tool call | ❌ Overhead |
| Need multiple searches | 🤔 Borderline | ✅ Better |

**Current Ambiguity:**
The docs say "When NOT to use the Task tool:
- If you want to read a specific file path, use the Read tool instead"

But what about:
- Reading 5 specific files?
- Reading files I need to find first?
- Reading files to answer a complex question?

**Recommendation for Architects:**

Provide a decision tree:
```
Do I know exact file paths?
├─ Yes → Do I need to read > 3 files?
│  ├─ No  → Use Read tool directly
│  └─ Yes → Use agent for batch reading + analysis
│
└─ No → Do I know search pattern?
   ├─ Yes → Is it simple (one grep)?
   │  ├─ Yes → Use Grep tool directly
   │  └─ No  → Use Explore agent
   │
   └─ No → Use general-purpose agent to figure it out
```

#### 3. Result Opacity

**The Problem:**
Agent returns final result, but I don't see its reasoning or process.

**Scenario:**
```
Task: "Find all API endpoints"

Agent returns: "Found 12 endpoints in src/api/"

Questions I have:
- Did it check tests?
- Did it look for HTTP route definitions?
- What search patterns did it use?
- Are there more in other directories?
```

I can't see the agent's work, only the result.

**Recommendation for Architects:**

Return structured results with metadata:
```rust
struct AgentResult {
    summary: String,           // "Found 12 endpoints"
    findings: Vec<Finding>,    // Detailed list
    search_strategy: String,   // "Searched src/api/ for 'fn get_', 'fn post_'"
    coverage: Coverage,        // Which dirs were searched
    confidence: Confidence,    // High/Medium/Low
}
```

This helps me verify the agent did what I expected.

#### 4. Prompt Overhead

**The Problem:**
Writing detailed prompts for agents takes time and effort.

**Example of what I write:**
```
Task(
    agent: "Explore",
    task: "Search the vtcode codebase for all tool definitions. Look in:
           - vtcode-core/src/tools/
           - vtcode-tools/
           Search for patterns like 'ToolRegistration::new', 'pub const.*TOOL',
           and Rust function names ending in '_executor'. Return a list of all
           tool names with their file locations. Include both builtin tools and
           MCP tools if found. Use thoroughness level 'medium' - don't need
           exhaustive search but cover main locations."
)
```

This is ~100 words to describe a 30-second task. High cognitive overhead.

**Recommendation for Architects:**

Support templated tasks:
```rust
// Predefined task templates
Task(
    agent: "Explore",
    template: "find_all_functions",
    params: {
        function_pattern: "*_executor",
        search_dirs: ["vtcode-core/src/tools/"],
    }
)

// Or common patterns
Task(
    agent: "Explore",
    quick_task: "list_tools"  // Built-in knowledge of how to find tools
)
```

### Design Patterns That Emerge

#### Pattern 1: Parallel Exploration

When I don't know a codebase:

```rust
// Launch 4 agents in parallel
Task(agent: "Explore", task: "Find all API route definitions")
Task(agent: "Explore", task: "Find all database models")
Task(agent: "Explore", task: "Find all authentication code")
Task(agent: "Explore", task: "Find all test files")

// Synthesize results to build mental model
```

This is vastly more efficient than sequential exploration.

#### Pattern 2: Divide and Conquer

For complex searches:

```rust
// Instead of one huge search
Task(agent: "Explore", task: "Find all uses of deprecated function")

// Split by directory
Task(agent: "Explore", task: "Find deprecated function in src/")
Task(agent: "Explore", task: "Find deprecated function in tests/")
Task(agent: "Explore", task: "Find deprecated function in examples/")

// Faster + more thorough
```

#### Pattern 3: Agent + Validation

Use agent for search, then validate myself:

```rust
// Agent does heavy lifting
let results = Task(agent: "Explore", task: "Find all database queries")

// I validate key results
read_file(results[0].file)  // Check first result makes sense
read_file(results[1].file)  // Check second result

// If validation passes, trust the rest
```

This balances efficiency with accuracy.

## Part 3: Cross-Cutting Design Principles

### Principle 1: Visibility Builds Trust

**Observation:**
Tools that show progress to users are dramatically more effective than opaque tools.

**TodoWrite Example:**
```
User sees:
✓ Read authentication files
✓ Understand current implementation
→ Editing user.rs to fix bug
  Pending: Test authentication flow
  Pending: Commit changes

User thinks: "The agent is making progress and I can see what it's doing"
```

vs.

```
User sees:
[Agent working...]

User thinks: "Is it stuck? Did it crash? What's happening?"
```

**Design Recommendation:**

Every long-running operation should have visibility:
- File operations: Show file being read/edited
- Build commands: Show output as it comes
- Searches: Show patterns being searched
- Agent work: Show which agent is working on what

### Principle 2: State Machines Beat Free-Form

**Observation:**
Rigid state transitions (pending → in_progress → completed) work better than free-form status updates.

**Why:**
- Prevents invalid states (can't be both "pending" and "completed")
- Clear invariants to check (only one in_progress)
- Easy to visualize (progress bar)
- Matches human mental models (not started / doing / done)

**Design Recommendation:**

Use state machines for:
- Task status (pending/in_progress/completed)
- Build status (queued/building/success/failure)
- Test status (not run/running/passed/failed)
- File status (unchanged/modified/saved)

Avoid free-form strings like:
- "kind of done" (what does this mean?)
- "almost working" (not actionable)
- "needs more work" (too vague)

### Principle 3: Forcing Functions Prevent Errors

**Observation:**
Tools that FORCE good behavior work better than tools that SUGGEST good behavior.

**TodoWrite Example:**
- Forces planning by requiring todo list for complex tasks
- Forces focus by allowing only ONE in_progress task
- Forces completion marking by making it required

**Alternatives that don't work as well:**
- "You should probably plan this out" (agents ignore)
- "Try to focus on one thing" (agents multitask anyway)
- "Don't forget to mark tasks complete" (agents forget)

**Design Recommendation:**

Enforce best practices through tool constraints:
```rust
// Force planning
if task.is_complex() && !has_todo_list() {
    return Err("Complex tasks require a todo list");
}

// Force one-at-a-time
if in_progress_count() >= 1 {
    return Err("Complete current task before starting new one");
}

// Force completion
if todo_list.has_pending() && try_finish_conversation() {
    return Err("Cannot finish with pending todos");
}
```

### Principle 4: Granularity Is Context-Dependent

**Observation:**
There's no universal "right" task size. It depends on:
- User expectations (do they want detailed progress?)
- Task complexity (refactoring vs reading a file)
- Agent capability (can it hold the plan in working memory?)
- Time constraints (quick fix vs major feature)

**Design Recommendation:**

Provide granularity guidance as a spectrum:

```
Micro (seconds):
- Read a file
- Run a grep
- Single edit

Fine (1-3 minutes):
- Understand a module
- Fix a small bug
- Add a function

Medium (5-10 minutes):
- Refactor a file
- Implement a feature
- Write tests for a module

Coarse (15-30 minutes):
- Implement a complete feature
- Refactor a subsystem
- Fix a complex bug

Macro (hours):
- Redesign architecture
- Migrate framework
- Implement major feature
```

TodoWrite is appropriate for Medium and Coarse tasks.
Task agents are appropriate for Fine, Medium, and Coarse tasks.
Direct tool use is appropriate for Micro and Fine tasks.

### Principle 5: Error Recovery > Prevention

**Observation:**
LLM agents WILL make mistakes. Design for recovery, not perfection.

**TodoWrite Example:**
I can add new tasks when I discover issues:
```
Original plan:
1. Fix bug
2. Commit

Reality:
1. ✓ Fix bug
2. ✓ Discover bug was in 2 files, not 1
3. ✓ Fix second file
4. ✓ Run tests - they failed!
5. [in progress] Fix test failures
6. [pending] Commit
```

The todo list EVOLVED to match reality.

**Design Recommendation:**

Support error recovery workflows:
- Allow adding tasks mid-execution
- Allow modifying task descriptions
- Allow marking tasks as blocked
- Allow adding notes/context
- Support retry/rollback operations

## Part 4: Recommendations for Architects

### TodoWrite Improvements

#### 1. Add Task Dependencies
```rust
pub struct Todo {
    content: String,
    activeForm: String,
    status: Status,
    depends_on: Vec<String>,  // Can't start until these complete
    blocks: Vec<String>,       // These can't start until I complete
}
```

This prevents me from starting tests before building.

#### 2. Support Task Hierarchy
```rust
pub struct Todo {
    id: String,
    parent: Option<String>,
    children: Vec<String>,
}

// Example:
- Implement authentication
  - Create auth module
  - Add login function
  - Add logout function
- Write tests
  - Test login
  - Test logout
```

Better visibility for complex tasks.

#### 3. Add Estimated Durations
```rust
pub struct Todo {
    content: String,
    estimated_duration: Option<Duration>,  // I estimate this will take 5min
    actual_duration: Option<Duration>,     // It actually took 8min
}
```

Helps users understand timeline.

#### 4. Support Notes/Annotations
```rust
pub struct Todo {
    content: String,
    notes: Vec<String>,  // "Updated: found bug in auth.rs too"
}
```

Provides context for plan changes.

### Task Agent Improvements

#### 1. Support Streaming Results
```rust
// Instead of waiting for final result
let result = agent.execute(task)?;

// Stream intermediate results
for update in agent.execute_streaming(task)? {
    match update {
        AgentUpdate::Progress(msg) => show_user(msg),
        AgentUpdate::Finding(finding) => process_finding(finding),
        AgentUpdate::Complete(result) => return result,
    }
}
```

Better user experience for long-running agents.

#### 2. Support Result Validation
```rust
let result = agent.execute(task)?;

// I can ask follow-up questions
if !result.seems_complete() {
    let more_results = agent.refine("also check tests/")?;
}
```

Allows iteration instead of one-shot.

#### 3. Provide Execution Trace
```rust
pub struct AgentResult {
    findings: Vec<Finding>,
    trace: ExecutionTrace,  // What the agent actually did
}

pub struct ExecutionTrace {
    tools_used: Vec<ToolCall>,
    search_patterns: Vec<String>,
    files_examined: Vec<PathBuf>,
    reasoning: Vec<String>,
}
```

I can verify the agent's approach.

#### 4. Template Library
```rust
// Pre-defined common tasks
let tools = agent.execute_template(
    "find_all_tools",
    params! { crate: "vtcode-core" }
)?;

let tests = agent.execute_template(
    "find_tests_for_module",
    params! { module: "auth" }
)?;
```

Reduces prompt writing overhead.

### General Tool Design Principles

#### 1. Make Common Operations One Call

**Bad:**
```rust
// Finding all Rust files requires multiple steps
let all_files = list_files("src/")?;
let rust_files = all_files.filter(|f| f.ends_with(".rs"));
```

**Good:**
```rust
// One call
let rust_files = list_files("src/", pattern: "*.rs")?;
```

#### 2. Provide Both Simple and Advanced Modes

**Simple (common case):**
```rust
let results = grep("function_name", "src/")?;
```

**Advanced (power users):**
```rust
let results = grep(GrepOptions {
    pattern: "function_name",
    path: "src/",
    file_pattern: "*.rs",
    context_lines: 3,
    max_results: 100,
})?;
```

#### 3. Return Structured Data, Not Strings

**Bad:**
```rust
let output = run_command("cargo build")?;
// Returns: "Compiling... error[E0425]: cannot find value `x`..."
// I have to parse this string
```

**Good:**
```rust
let result = build()?;
// Returns:
BuildResult {
    success: false,
    errors: vec![
        CompileError {
            code: "E0425",
            message: "cannot find value `x`",
            file: "src/main.rs",
            line: 42,
        }
    ],
    warnings: vec![...],
}
```

#### 4. Include Metadata in Results

Every result should include context:
```rust
pub struct SearchResult {
    // The data
    matches: Vec<Match>,

    // Metadata
    query: String,           // What was searched
    search_path: PathBuf,    // Where was searched
    total_files: usize,      // How many files examined
    search_duration: Duration, // How long it took
    truncated: bool,         // Were results limited?
}
```

This helps me understand and trust the results.

## Conclusion

Building tools for agentic LLMs requires thinking about:
1. **Visibility** - Users need to see what's happening
2. **State** - Clear state machines beat free-form
3. **Forcing** - Constraints prevent errors better than suggestions
4. **Granularity** - Context-dependent, provide guidance
5. **Recovery** - Agents will err, design for adaptation

TodoWrite succeeds by forcing planning, showing progress, and constraining state.
Task agents succeed by enabling parallelism, specialization, and delegation.

Both could improve with:
- Better granularity guidance
- Support for plan evolution
- Richer result metadata
- Clearer delegation boundaries

The best tools feel like extensions of the agent's capabilities, not obstacles to work around.
