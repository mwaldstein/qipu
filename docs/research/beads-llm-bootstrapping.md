# Beads LLM Bootstrapping Research

Research on how beads (bd) bootstraps LLMs into effective tool usage.

## Key Commands

### `bd init`

**Purpose**: Initialize beads in a project directory.

**Key features**:
- `--quiet` flag for non-interactive agent use
- `--stealth` mode: adds .beads to .gitignore, configures global gitattributes
- `--contributor` / `--team` wizards for different workflows
- Auto-imports from git history if JSONL exists
- Auto-installs git hooks (pre-commit, post-merge, pre-push, post-checkout)
- Auto-installs merge driver for conflict resolution
- Auto-detects prefix from directory name or existing issues
- Runs `bd doctor` diagnostics at end to catch setup issues
- Adds "landing the plane" instructions to AGENTS.md

**After init outputs**:
```
bd initialized successfully!
  Backend: sqlite
  Database: .beads/beads.db
  Issue prefix: myproject
  Issues will be named: myproject-<hash> (e.g., myproject-a3f2dd)

Run `bd quickstart` to get started.
```

### `bd setup <recipe>`

**Purpose**: Install integration files for specific editors/agents.

**Recipes supported**:
- `cursor` - .cursor/rules/beads.mdc
- `claude` - Claude Code hooks (SessionStart, PreCompact)
- `gemini` - Gemini-specific integration
- `codex` - Updates AGENTS.md
- `aider` - .aider.conf.yml
- `windsurf`, `cody`, `kilocode`, `junie`, `factory`

**Key insight**: Uses a recipe system with TOML config for extensibility:
```bash
bd setup --list                    # List all recipes
bd setup --add myeditor path.md    # Add custom recipe
bd setup cursor --check            # Verify installation
bd setup cursor --remove           # Uninstall
```

### `bd onboard`

**Purpose**: Display minimal snippet for AGENTS.md (informational only, does not modify files).

**Philosophy**: Keep AGENTS.md lean, point to `bd prime` for dynamic context.

**Output** (~10 lines):
```markdown
## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`
```

### AGENTS.md Modification (in `bd init`)

**Key distinction**: `bd init` automatically modifies AGENTS.md, while `bd onboard` only displays content.

**What `bd init` writes to AGENTS.md** (via `addLandingThePlaneInstructions`):

If AGENTS.md **doesn't exist**, creates it with:
```markdown
# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)
[... full landing the plane section ...]
```

If AGENTS.md **exists but lacks "Landing the Plane"**, appends the section.

**The "Landing the Plane" section** (~30 lines):
```markdown
## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
```

**Why two approaches?**

| Command | Modifies Files? | When Used |
|---------|----------------|-----------|
| `bd init` | Yes - creates/updates AGENTS.md | Project setup (runs once) |
| `bd onboard` | No - display only | Manual reference, understanding |

The init-time modification ensures:
1. Every initialized project has "landing the plane" instructions
2. LLMs see these instructions automatically (AGENTS.md is loaded by most tools)
3. No manual step required to set up agent instructions

The onboard command exists for:
1. Understanding what to add manually (if init didn't run)
2. Showing recommended content for other files (copilot-instructions.md)
3. Reference documentation

### `bd prime`

**Purpose**: Output AI-optimized workflow context (~1-2k tokens).

**Key features**:
1. **Adaptive output based on environment**:
   - Detects MCP mode (checks ~/.claude/settings.json for beads server)
   - MCP mode: ~50 tokens (brief reminders)
   - CLI mode: ~1-2k tokens (full command reference)

2. **Adaptive based on git workflow**:
   - Detects ephemeral branches (no upstream)
   - Detects daemon auto-sync status
   - Detects local-only repos (no remote)
   - Adjusts session close protocol accordingly

3. **Custom override**: `.beads/PRIME.md` replaces default output entirely

4. **Session Close Protocol** (CRITICAL):
   ```
   [ ] 1. git status              (check what changed)
   [ ] 2. git add <files>         (stage code changes)
   [ ] 3. bd sync                 (commit beads changes)
   [ ] 4. git commit -m "..."     (commit code)
   [ ] 5. bd sync                 (commit any new beads changes)
   [ ] 6. git push                (push to remote)
   ```
   
5. **Core Rules emphasized**:
   - Use beads for ALL task tracking (never TodoWrite, TaskCreate, markdown)
   - Create issue BEFORE writing code
   - Mark in_progress when starting
   - "Persistence you don't need beats lost context"

6. **Flags**:
   - `--full` - Force full CLI output (ignore MCP detection)
   - `--mcp` - Force minimal MCP output
   - `--stealth` - No git operations in close protocol
   - `--export` - Output default content (ignores PRIME.md override)

### `bd quickstart`

**Purpose**: Interactive quick start guide showing common workflows.

Shows: init, create, view, dependencies, ready work, updating, closing, database location, agent integration, git workflow.

### `bd hooks install`

**Purpose**: Install git hooks for automatic sync.

Hooks installed:
- **pre-commit**: Flush pending changes before commit
- **post-merge**: Import updated JSONL after pull/merge
- **pre-push**: Export database to JSONL before push
- **post-checkout**: Import JSONL after branch checkout

## Design Patterns

### 1. Minimal AGENTS.md, Dynamic Prime

AGENTS.md contains ~10 lines pointing to `bd prime`. The prime command provides:
- Always-current workflow details
- Context-aware protocol (git state, MCP mode, etc.)
- Avoids stale instructions when bd upgrades

### 2. Session Close Protocol

Critical insight: LLMs need explicit checklists to "land the plane". Beads emphasizes:
- NEVER say "done" without completing the checklist
- NEVER skip git push
- Push is NON-NEGOTIABLE

### 3. Hooks for Auto-injection

Claude Code hooks (SessionStart, PreCompact) auto-call `bd prime`:
- Prevents agents from forgetting workflow after context compaction
- No manual intervention needed

### 4. Adaptive Context

Prime detects environment and adjusts:
- MCP vs CLI mode
- Ephemeral vs tracked branches
- Daemon auto-sync vs manual
- Local-only vs remote repos

### 5. Recipe System for Integrations

Extensible recipe system:
- Built-in recipes for major editors
- User-defined recipes via TOML config
- Common template shared across file-based recipes

## Comparison: Beads vs Current Qipu

| Aspect | Beads | Qipu |
|--------|-------|------|
| Init modifies AGENTS.md | Yes - adds "landing the plane" | No |
| Init message | Shows next command (`bd quickstart`) | Shows nothing specific |
| Setup recipes | Multiple editors (cursor, claude, aider, etc.) | Only agents-md |
| Prime adaptive | Yes (MCP detection, git state, daemon status) | No |
| Session close protocol | Explicit checklist with emphasis | Not present |
| Custom prime override | `.beads/PRIME.md` | Not present |
| Git hooks | Auto-installed on init | Not present |
| Onboard purpose | Display-only (reference) | Creates/checks AGENTS.md |

## Recommendations for Qipu

### High Priority

1. **Add session close protocol to prime output**
   - Explicit checklist for "landing the plane"
   - Emphasize git operations are non-negotiable
   - Adapt based on git state (branch tracking, remote existence)

2. **Make init more helpful**
   - Show next command to run
   - Optionally run diagnostics
   - Auto-add qipu instructions to AGENTS.md

3. **Support custom PRIME.md override**
   - Allow `.qipu/PRIME.md` to replace default output
   - Enables project-specific workflow customization

4. **Trim onboard snippet**
   - Current ~40 lines is too much for AGENTS.md
   - Target ~10 lines pointing to `qipu prime`
   - Full details live in prime output

### Medium Priority

5. **Add more setup recipes**
   - cursor (.cursor/rules/)
   - claude (hooks)
   - aider (.aider.conf.yml)
   - Use recipe system for extensibility

6. **Add quickstart command**
   - Interactive guide showing common workflows
   - Bridges gap between init and productive use

7. **Adaptive prime output**
   - Detect if store is empty vs populated
   - Adjust guidance accordingly

### Lower Priority

8. **Git hooks integration**
   - Auto-sync on commit/merge
   - Would require significant infrastructure

9. **MCP detection**
   - Detect if running via MCP server
   - Adjust output verbosity

## Key Insights

### 1. "Persistence you don't need beats lost context"

LLMs have goldfish memory. The solution isn't just providing tools, but ensuring:
- Tools are used (via explicit instructions)
- Work is saved (via session close protocol)
- Context survives compaction (via hooks/auto-injection)

### 2. The Closed-Loop Bootstrapping Flow

```
init -> setup <editor> -> [hooks auto-inject prime] -> prime guides session -> close protocol saves work
         ^                                                                              |
         └──────────────────────────────────────────────────────────────────────────────┘
```

Each step reinforces the next. The session close protocol feeds back into the next session via git sync.

### 3. Session Close Protocol is Critical

LLMs need explicit checklists. Without them:
- Work gets "done" but not committed
- Changes sit in local state
- Next session starts from stale state
- Multi-agent coordination breaks

The checklist must be:
- Visible in prime output
- Non-negotiable ("NEVER skip git push")
- Adaptive to environment (ephemeral branches, local-only repos)

### 4. Minimal Static, Dynamic Runtime

AGENTS.md should be ~10 lines pointing to `prime`. Why:
- Static instructions get stale when tool upgrades
- Dynamic output can adapt to environment
- Saves tokens in agent context
- Single source of truth for workflow

### 5. Knowledge vs Task Tracking Differ

Beads tracks *tasks* (discrete work items with status, priority, dependencies).
Qipu tracks *knowledge* (notes with types, links, value scores).

Different domains need different session protocols:
- Beads: "close issues, sync, push"
- Qipu: "capture insights, link notes, commit knowledge"

### 6. The "Why" Matters More Than "How"

Beads prime output explains *why* to use the tool:
- "Persistence you don't need beats lost context"
- "Create issue BEFORE writing code"

Commands are secondary to motivation. An LLM that understands *why* will use tools correctly even with incomplete instructions.

---

## Qipu Implementation Recommendations

### High Priority

#### 1. Add Session Protocol to Prime

Current qipu prime focuses on "what" (commands, MOCs, recent notes).
Add "how to end session" protocol:

```markdown
## Session Protocol

**Before ending session:**
1. Capture any new insights: `qipu capture --title "..."`
2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <t>`
3. Commit changes: `git add .qipu && git commit -m "knowledge: ..."`

**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.
```

#### 2. Init Should Modify AGENTS.md Automatically

**Key insight from beads**: `bd init` automatically creates/updates AGENTS.md with "landing the plane" instructions. This ensures every project has proper agent guidance without manual steps.

Qipu should do the same:

```rust
// In init command, after store creation:
fn add_qipu_to_agents_md(verbose: bool) -> Result<()> {
    let agents_md = PathBuf::from("AGENTS.md");
    
    if !agents_md.exists() {
        // Create with full qipu section
        fs::write(&agents_md, QIPU_AGENTS_CONTENT)?;
        if verbose {
            println!("  Created AGENTS.md with qipu instructions");
        }
    } else {
        // Append if "qipu" section not present
        let content = fs::read_to_string(&agents_md)?;
        if !content.contains("## Qipu Knowledge") {
            let mut file = fs::OpenOptions::new().append(true).open(&agents_md)?;
            writeln!(file, "\n{}", QIPU_AGENTS_SECTION)?;
            if verbose {
                println!("  Added qipu section to AGENTS.md");
            }
        }
    }
    Ok(())
}
```

The AGENTS.md section should include:
1. What qipu is (1-2 lines)
2. Session start: `qipu prime`
3. Quick reference (4-5 commands)
4. Session end protocol (commit knowledge)

#### 3. Improve Init Output

Current: Creates store, shows nothing actionable.
Proposed:
```
qipu initialized successfully!
  Store: .qipu/
  Added qipu instructions to AGENTS.md
  
Run `qipu prime` for workflow context.
Run `qipu quickstart` for a guided tour.
```

#### 3. Support .qipu/PRIME.md Override

Allow projects to customize prime output entirely:
```rust
// In prime.rs execute()
let custom_prime = store.root().join("PRIME.md");
if custom_prime.exists() {
    print!("{}", fs::read_to_string(custom_prime)?);
    return Ok(());
}
```

#### 4. Clarify Onboard vs Init Roles

**Beads pattern:**
- `bd init` - Modifies AGENTS.md (creates or appends)
- `bd onboard` - Display-only (shows what SHOULD be in AGENTS.md)

**Current qipu pattern:**
- `qipu init` - Does NOT modify AGENTS.md
- `qipu setup agents-md` - Creates AGENTS.md if missing
- `qipu onboard` - Displays snippet

**Recommended qipu pattern:**
- `qipu init` - Creates/updates AGENTS.md with qipu section (like beads)
- `qipu onboard` - Display-only reference (like beads)
- `qipu setup <recipe>` - For non-AGENTS.md integrations (cursor, claude, etc.)

The onboard output should be trimmed to ~10 lines:

```markdown
## Qipu Knowledge Graph

Run `qipu prime` for workflow context.

**Quick reference:**
- `qipu capture` - Quick capture from stdin
- `qipu link add <from> <to>` - Create typed link
- `qipu search <query>` - Search notes
- `qipu context` - Build LLM context bundle

For full workflow: `qipu prime`
```

### Medium Priority

#### 5. Add Quickstart Command

New command showing common workflows:
```
qipu quickstart

# Qipu Quick Start

## Capturing Knowledge
  qipu capture --title "TIL: ..."     Capture quick insight
  qipu create "Topic X" --type lit    Create literature note

## Building the Graph
  qipu link add <from> <to> --type derived-from
  qipu link tree <id>                 Visualize connections

## Finding Knowledge
  qipu search "error handling"
  qipu context --tag rust --max-chars 8000

## Session Workflow
  1. qipu prime                       Load context at session start
  2. Work, capturing insights
  3. Link new notes to existing
  4. git commit                       Save your knowledge
```

#### 6. Add More Setup Recipes

Extend setup command:
```rust
// recipes: cursor, claude, aider, opencode
match recipe {
    "cursor" => write(".cursor/rules/qipu.mdc", TEMPLATE),
    "claude" => install_claude_hooks(),
    "aider" => write(".aider.conf.yml", AIDER_TEMPLATE),
    "opencode" => write("AGENTS.md", AGENTS_TEMPLATE),
}
```

#### 7. Adaptive Prime Based on Store State

```rust
let notes = store.list_notes()?;
if notes.is_empty() {
    output_empty_store_primer();  // Focus on "how to start"
} else {
    output_populated_primer();    // Focus on "what's here"
}
```

### Lower Priority

#### 8. Git Hooks for Auto-Sync

Would require:
- Hook installation in init
- Pre-commit: rebuild index
- Post-merge: reindex
- Significant complexity for knowledge (vs task) tracking

#### 9. MCP Mode Detection

Detect MCP environment, output minimal primer:
```rust
if is_mcp_active() {
    println!("Use qipu for knowledge. Run `qipu context` for relevant notes.");
    return Ok(());
}
```

---

## Summary

The core insight from beads: bootstrapping LLMs requires a **closed loop** where each step reinforces the next. For qipu:

1. **Init** should modify AGENTS.md automatically (not just create the store)
2. **Prime** should explain why + how to end session
3. **Session protocol** should ensure knowledge is committed
4. **Onboard** should be display-only reference, not the primary setup mechanism

### The Init vs Onboard Distinction

This is crucial for qipu's design:

| Aspect | `init` | `onboard` |
|--------|--------|-----------|
| Runs when | Project setup (once) | Manual reference (anytime) |
| Modifies files | Yes (AGENTS.md) | No (display only) |
| Purpose | Ensure every project has agent instructions | Show what instructions look like |
| User action | Automatic | Informational |

**Why this matters:**

If `init` doesn't modify AGENTS.md, users must:
1. Run `init`
2. Remember to run `setup agents-md`
3. Or manually add qipu instructions

Most won't do steps 2-3. Result: LLMs don't know about qipu.

If `init` DOES modify AGENTS.md:
1. Every initialized project has qipu instructions
2. LLMs automatically learn about qipu
3. Zero manual steps required

Beads chose the second path. Qipu should too.

### The Goal

An LLM that understands *why* to use qipu will build better knowledge graphs than one that just knows the commands. The bootstrapping flow ensures this understanding is present in every session.
