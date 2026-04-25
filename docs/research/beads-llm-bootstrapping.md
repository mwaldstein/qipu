# Beads vs Qipu: LLM Bootstrapping Comparison

Research on how beads (bd) bootstraps LLMs into effective tool usage, and how qipu compares.

> **Last updated**: 2026-02-11

## Scale Comparison

| Metric | Beads | Qipu |
|--------|-------|------|
| Total commands | 93 | 33 |
| Global flags | 13 | 12 |
| Setup recipes | 10 | 2 |
| Help text style | Verbose with examples | Concise |

## Key Commands

### `init`

#### Beads `bd init`

**Purpose**: Initialize beads in a project directory.

**Key features**:
- `--quiet` flag for non-interactive agent use
- `--stealth` mode: adds .beads to .gitignore, configures global gitattributes
- `--contributor` / `--team` wizards for different workflows
- Auto-imports from git history if JSONL exists
- Auto-installs git hooks (pre-commit, post-merge, pre-push, post-checkout)
- Auto-installs merge driver for conflict resolution
- Auto-detects prefix from directory name or existing issues
- Adds "landing the plane" instructions to AGENTS.md
- `--backend dolt` for version-controlled storage
- `--from-jsonl` to import from file instead of git history

**After init outputs**:
```
bd initialized successfully!
  Backend: sqlite
  Database: .beads/beads.db
  Issue prefix: myproject
  Issues will be named: myproject-<hash> (e.g., myproject-a3f2dd)

Run `bd quickstart` to get started.
```

**Flags** (7 command-specific, plus 13 global):
```
--backend, -b, --branch, --contributor, --force, --from-jsonl, -p, --prefix, -q, 
--server, --server-host, --server-port, --server-user, --setup-exclude, --skip-hooks, 
--skip-merge-driver, --stealth, --team
```

#### Qipu `qipu init`

**Purpose**: Initialize a new qipu store.

**Key features**:
- `--visible` flag for non-hidden store (qipu/ instead of .qipu/)
- `--stealth` mode - add store to .gitignore
- `--branch` for protected branch workflow
- `--agents-md` to write qipu section to AGENTS.md
- `--no-index` to skip automatic indexing
- `--index-strategy` override (adaptive, full, incremental, quick)

**Flags** (7 command-specific, plus 12 global):
```
--root, --visible, --stealth, --store, --branch, --format, --no-index, -q, 
--index-strategy, -v, --agents-md, --log-level, --log-json
```

**Gap analysis**: 
- Beads auto-modifies AGENTS.md; qipu requires `--agents-md` flag
- Beads shows next command; qipu doesn't
- Beads auto-installs hooks; qipu has separate `hooks` command

---

### `prime`

#### Beads `bd prime`

**Purpose**: Output AI-optimized workflow context (~80 lines, ~1.5k tokens).

**Output structure**:
```
# Beads Workflow Context
> Context Recovery note
# SESSION CLOSE PROTOCOL (prominent with emojis)
**CRITICAL**: Before saying "done" or "complete", you MUST run this checklist:
[ ] 1. git status
[ ] 2. git add <files>
[ ] 3. bd sync
[ ] 4. git commit
[ ] 5. bd sync
[ ] 6. git push
## Core Rules
- Use beads for ALL task tracking
- Prohibited: TodoWrite, TaskCreate, markdown files
- "Persistence you don't need beats lost context"
## Essential Commands
[... categorized by: Finding Work, Creating & Updating, Dependencies, Sync, Health ...]
## Common Workflows
[... bash code blocks with examples ...]
```

**Key features**:
1. **Adaptive output**: MCP mode (~50 tokens) vs CLI mode (~1.5k tokens)
2. **Environment detection**: ephemeral branches, daemon status, local-only repos
3. **Custom override**: `.beads/PRIME.md` replaces default output
4. **Config option**: `no-git-ops` for stealth mode

**Flags**:
```
--export, --full, -h, --mcp, --stealth
```

#### Qipu `qipu prime`

**Purpose**: Output session-start primer for LLM agents.

**Output structure**:
```
# Qipu Knowledge Store Primer
## About Qipu
## Quick Reference
[8 commands with brief descriptions]
## Session Protocol
**Before ending session:**
1. Capture any new insights
2. Link new notes to existing knowledge
3. Commit changes
**Why this matters:** Knowledge not committed is knowledge lost.
## Ontology
[note types, link types with inverses]
## Key Maps of Content
[recent MOCs with tags]
## Recently Updated Notes
```

**Key features**:
1. **Adaptive output**: `--mcp` for minimal (~50 tokens), `--full` for CLI
2. **Compact mode**: `--compact` omits MOCs and recent notes
3. **Minimal mode**: `--minimal` only ontology and commands
4. **Custom override**: `--use-prime-md` to use `.qipu/PRIME.md`

**Flags**:
```
--compact, --minimal, --full, --mcp, --use-prime-md
```

**Gap analysis**:
- Beads has more emphatic session close protocol (emojis, "CRITICAL", "NEVER")
- Beads includes "Prohibited" section (what NOT to do)
- Beads includes categorized commands and workflow examples
- Qipu includes store-specific content (ontology, MOCs, recent notes)
- Both now have MCP detection and custom override

---

### `setup`

#### Beads `bd setup`

**Purpose**: Install integration files for AI editors and coding assistants.

**Available recipes** (10):
- `aider` - Aider config and instruction files
- `claude` - Claude Code hooks (SessionStart, PreCompact)
- `codex` - Codex CLI AGENTS.md section
- `cody` - Cody AI rules file
- `cursor` - Cursor IDE rules file
- `factory` - Factory Droid AGENTS.md section
- `gemini` - Gemini CLI hooks (SessionStart, PreCompress)
- `junie` - Junie guidelines and MCP configuration
- `kilocode` - Kilo Code rules file
- `windsurf` - Windsurf editor rules file

**Features**:
```
--add <name> <path>   Add custom recipe
--check               Verify installation
--list                List all recipes
--output, -o          Write to custom path
--print               Print to stdout
--project             Project-only install (claude/gemini)
--remove              Uninstall
--stealth             Stealth mode
```

#### Qipu `qipu setup`

**Purpose**: Install qipu integration instructions for agent tools.

**Available integrations** (2):
- `agents-md` - AGENTS.md standard for OpenCode, Cline, Roo-Cline
- `cursor` - Cursor IDE project rules

**Features**:
```
--list, --print, --check, --remove
```

**Gap analysis**:
- Beads has 5x more integrations
- Beads supports custom recipe system (`--add`)
- Beads has hooks-based integrations (claude, gemini)

---

### `onboard`

#### Beads `bd onboard`

**Purpose**: Display minimal AGENTS.md snippet (display only, no file modification).

**Output** (~15 lines):
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

Also includes GitHub Copilot instructions.

#### Qipu `qipu onboard`

**Purpose**: Display minimal AGENTS.md snippet for agent integration.

**Output** (~11 lines):
```markdown
## Qipu Knowledge

This project uses **qipu** for knowledge management.
Run `qipu prime` for workflow context.

**Quick reference:**
- `qipu prime` - Get store overview
- `qipu create` - Create note
- `qipu capture` - Quick capture
- `qipu search` - Search notes
- `qipu context` - Build LLM context

For full workflow: `qipu prime`
```

**Gap analysis**: Both now follow the same pattern - minimal snippet pointing to prime.

---

### `quickstart`

#### Beads `bd quickstart`

**Output sections**:
- GETTING STARTED (init examples)
- CREATING ISSUES (with flags examples)
- VIEWING ISSUES (list, show)
- MANAGING DEPENDENCIES (dep commands)
- DEPENDENCY TYPES (blocks, related, parent-child, discovered-from)
- READY WORK (bd ready)
- UPDATING ISSUES (update examples)
- CLOSING ISSUES (close examples)
- DATABASE LOCATION (discovery hierarchy)
- AGENT INTEGRATION (design philosophy)
- DATABASE EXTENSION (link to docs)
- GIT WORKFLOW (auto-sync explanation)

#### Qipu `qipu quickstart`

**Output sections**:
- Capturing Knowledge (capture, create)
- Building the Graph (link add, tree, path)
- Finding Knowledge (search, list, inbox)
- Building Context (context command)
- Session Workflow (prime, capture, link, commit)
- Next Steps (pointers)

**Gap analysis**: 
- Beads focuses on task tracking workflows
- Qipu focuses on knowledge graph workflows
- Both appropriate to their domains

---

### `hooks`

#### Beads `bd hooks`

**Subcommands**: `install`, `list`, `run`, `uninstall`

**Hooks managed**:
- pre-commit: Flushes pending changes to JSONL before commit
- post-merge: Imports updated JSONL after pull/merge
- pre-push: Prevents pushing stale JSONL
- post-checkout: Imports JSONL after branch checkout
- prepare-commit-msg: Adds agent identity trailers

#### Qipu `qipu hooks`

**Subcommands**: `install`, `run`, `list`, `uninstall`, `status`

**Gap analysis**: Both have hooks support. Beads auto-installs on init; qipu requires explicit `hooks install`.

---

### `doctor`

#### Beads `bd doctor`

**Checks performed**:
- .beads/ directory exists
- Database version and migration status
- Schema compatibility
- CLI version currency
- Claude plugin currency
- Multiple database/JSONL files
- Daemon health
- Database-JSONL sync status
- File permissions
- Circular dependencies
- Git hooks installed
- .beads/.gitignore up to date

**Modes**:
- `--perf` - Performance diagnostics with timing
- `--output` - Save to JSON for historical analysis
- `--check <name>` - Run specific check (pollution, validate)
- `--deep` - Full graph integrity validation

#### Qipu `qipu doctor`

**Checks performed**:
- Store invariants
- Near-duplicate notes (with `--duplicates`)
- Ontology validation (with `--check ontology`)

**Options**:
- `--fix` - Auto-repair
- `--threshold` - Similarity threshold for duplicates

**Gap analysis**: Beads doctor is more comprehensive (version checks, daemon health, git hooks). Qipu focuses on data integrity.

---

### `human` (Beads only)

**Purpose**: Show essential commands for human users (subset of full command list).

**Output**: ~35 commands organized by category (Working With Issues, Finding Work, Dependencies, Setup & Sync, Getting Help) plus quick examples.

**Qipu equivalent**: None. All 33 commands shown in `--help`.

---

## Current State Comparison

| Aspect | Beads | Qipu | Status |
|--------|-------|------|--------|
| Commands | 93 | 33 | Beads is feature-complete; qipu is focused |
| Init modifies AGENTS.md | Yes (automatic) | Via `--agents-md` flag | Qipu improved, not automatic |
| Init shows next command | Yes (`bd quickstart`) | No | Gap remains |
| Setup recipes | 10 | 2 | Gap remains |
| Prime adaptive | Yes (MCP, git state, daemon) | Yes (MCP, compact modes) | Qipu improved |
| Prime has session close protocol | Yes (emphatic with emojis) | Yes (calm version) | Both have it |
| Prime has custom override | `.beads/PRIME.md` | `.qipu/PRIME.md` via flag | Qipu improved |
| Prime has prohibited section | Yes ("never TodoWrite") | No | Gap remains |
| Git hooks | Auto-installed on init | Separate `hooks` command | Different approaches |
| Onboard display-only | Yes | Yes | Same pattern |
| Quickstart command | Yes | Yes | Both have it |
| Human-focused help | `bd human` command | No | Gap remains |
| Doctor comprehensive | Yes (versions, daemon, perf) | Basic (data integrity) | Different focus |

## Recommendations

### Already Implemented (from original research)

- [x] Session close protocol in prime
- [x] Quickstart command
- [x] Custom PRIME.md override
- [x] MCP detection in prime
- [x] Trimmed onboard output
- [x] Git hooks command
- [x] Cursor setup recipe

### Remaining Gaps

1. **Init should auto-modify AGENTS.md** (not require `--agents-md`)
2. **Init should show next command** (point to `qipu quickstart`)
3. **Add "Prohibited" section to prime** (what NOT to do)
4. **Add more setup recipes** (claude hooks, aider, gemini)
5. **Consider `bd human` equivalent** for essential commands only
6. **Make prime session protocol more emphatic** (optional - design choice)

## Key Insights

### 1. Guidance Philosophy

**Beads**: Prescriptive with strong opinions
- "NEVER skip git push"
- "Prohibited: TodoWrite, TaskCreate"
- Emojis for emphasis
- Explicit "do this, don't do that"

**Qipu**: Informative with gentle nudges
- "Why this matters: Knowledge not committed is knowledge lost"
- No prohibited section
- Calm, documentation-style

Both are valid. Beads targets task coordination where mistakes are costly. Qipu targets knowledge accumulation where gradual improvement is fine.

### 2. Help Text Approach

**Beads**: Verbose help with examples inline
```
With --stealth: configures per-repository git settings for invisible beads usage:
  • .git/info/exclude to prevent beads files from being committed
  • Claude Code settings with bd onboard instruction
  Perfect for personal use without affecting repo collaborators.
```

**Qipu**: Concise flags with brief descriptions
```
--stealth    Stealth mode - add store to .gitignore
```

Beads prioritizes discoverability; qipu prioritizes brevity.

### 3. Command Surface

Beads has 93 commands because it solves:
- Task tracking (create, update, close, dependencies)
- Multiple backends (sqlite, dolt)
- Multiple integrations (jira, linear, gitlab)
- Agent coordination (swarm, molecule, gates)
- Federation (peer-to-peer sync)

Qipu has 33 commands because it focuses on:
- Note management (create, capture, list, search)
- Graph traversal (link, context)
- Store health (doctor, sync)

The gap is primarily scope, not completeness within scope.

### 4. Right Action Guidance

**Beads**: Multiple paths to right action
- `bd ready` - Shows unblocked work (primary path)
- `bd prime` - Session context (secondary path)
- `bd quickstart` - Learning path
- `bd human` - Essential commands
- Hooks auto-inject context

**Qipu**: Single path to right action
- `qipu prime` - Session context (primary path)
- `qipu quickstart` - Learning path
- No hooks auto-injection

Beads provides more "nudges" toward correct usage. Qipu assumes users will find `qipu prime`.

---

## Progressive Disclosure Patterns

### Overview

Beads uses several techniques to avoid information dumps and reveal information incrementally as needed. This is critical for LLM agents who have limited context windows.

### Pattern 1: Tiered Help (Human vs Full)

**Beads**:
- `bd human` - ~35 essential commands organized by category (for humans)
- `bd --help` - 93 commands in grouped categories (full reference)

**Qipu**:
- `qipu --help` - 33 commands (single tier)
- No "essential commands only" view

**Insight**: When command count grows, provide a filtered view for common usage. This reduces cognitive load without hiding power-user features.

### Pattern 2: Summary → Detail Drill-Down

**Beads**:
```
bd status          # Overview: counts, health, sync status
bd list            # Summary: one-line per issue
bd list --long     # Detail: multi-line per issue
bd show <id>       # Full: all fields, description, audit trail
bd show --short    # Compact: one-line for scripting
```

**Qipu**:
```
qipu list          # Summary: one-line per note
qipu show <id>     # Full: YAML with body
# No intermediate detail level
# No overview/status command
```

**Insight**: Provide multiple levels of detail. Overview → List → Show creates a clear drill-down path. Qipu lacks the overview level.

### Pattern 3: Adaptive Prime Output

**Beads**:
- `bd prime` - Full CLI output (~83 lines, ~1.5k tokens)
- `bd prime --mcp` - Minimal for MCP mode (~13 lines, ~50 tokens)
- MCP mode auto-detected by checking `~/.claude/settings.json`
- Adaptive to git state (ephemeral branches, daemon status)

**Qipu**:
- `qipu prime` - Full output (~89 lines)
- `qipu prime --mcp` - Minimal (~2 lines)
- `qipu prime --compact` - Omit MOCs and recent notes
- `qipu prime --minimal` - Only ontology and commands

**Insight**: Both now have MCP modes. Beads does more environment detection (daemon, branch state) to adapt content.

### Pattern 4: Grouped Command Help

**Beads** `bd --help` output:
```
Maintenance:
  rename-prefix, repair, resolve-conflicts

Integrations & Advanced:
  [empty group header - visual separator]

Working With Issues:
  children, close, comments, create, edit, ...

Views & Reports:
  activity, count, diff, history, lint, ...

Dependencies & Structure:
  dep, duplicate, epic, graph, ...

Sync & Data:
  branch, daemon, export, import, sync, ...

Setup & Configuration:
  backend, config, hooks, init, ...
```

**Qipu** `qipu --help` output:
```
Commands:
  init, create, new, list, show, inbox, capture, index, search, ...
  [flat list, no grouping]
```

**Insight**: Grouping commands by category helps users discover related functionality without reading all 33+ commands. Beads organizes by workflow stage.

### Pattern 5: Limit and Pagination

**Beads**:
- `bd list` - Default 50 items with `--limit` control
- `bd list --limit 5` - Shows count and hints: "Showing 5 issues (use --limit 0 for all)"
- `bd ready --limit 10` - Default 10, respects user's attention

**Qipu**:
- `qipu list` - Shows all items (no pagination)
- No `--limit` flag

**Insight**: Limits prevent overwhelming output. The hint text teaches users how to get more.

### Pattern 6: Information Density Control

**Beads**:
- `bd list` - One line per issue: `○ qipu-fshe [● P0] [bug] - title`
- `bd list --long` - Multi-line with description preview
- `bd list --pretty` - Tree format with visual hierarchy
- `bd show --short` - Compact for scripting

**Qipu**:
- `qipu list` - One line per note: `qp-29b1 [P] title compacts=2`
- `qipu show` - Full YAML output
- No intermediate detail level

**Insight**: Multiple output formats let users choose density. Scripts need compact; humans scanning need moderate; detail review needs full.

### Pattern 7: Context-Sensitive Defaults

**Beads**:
- `bd ready` - Filters to actionable work (open, no blockers)
- `bd list` - Shows open by default (excludes closed)
- `bd prime` in MCP mode - Minimal because agent already knows context

**Qipu**:
- `qipu list` - Shows all notes (no status filtering)
- `qipu inbox` - Filters to unprocessed (fleeting/literature without links)

**Insight**: Default to the most common use case. "Show me what I can work on" (ready) vs "show me everything" (list all).

### Pattern 8: Explicit Next Steps

**Beads** `bd status` output ends with:
```
For more details, use 'bd list' to see individual issues.
```

**Beads** `bd init` output:
```
Run `bd quickstart` to get started.
```

**Qipu** `qipu init` output:
```
# No next-step guidance
```

**Insight**: Tell users what to do next. Reduces decision paralysis after successful setup.

---

## Recommendations for Qipu

### High Impact

1. **Add status/overview command**
   - `qipu status` showing: note counts by type, recent activity, health checks
   - Creates drill-down entry point: status → list → show

2. **Add --limit to list commands**
   - `qipu list --limit 10`
   - Show hint when truncated: "Showing 10 notes (use --limit 0 for all)"

3. **Group commands in --help**
   - Categories: Knowledge Capture, Navigation, Graph, Store, Setup
   - Reduces cognitive load for new users

4. **Add next-step hints**
   - `qipu init` → "Run `qipu quickstart` to get started"
   - `qipu status` → "Run `qipu list` for details"

### Medium Impact

5. **Add intermediate detail level to list**
   - `qipu list --long` or `qipu list --verbose` for multi-line per note
   - Include first line of body, tags, link count

6. **Create human-focused command subset**
   - `qipu essentials` or document in AGENTS.md
   - 10-15 commands covering 80% of use cases

7. **Improve show output modes**
   - `qipu show --short` for one-line compact output
   - Useful for scripting and quick reference

### Lower Impact

8. **Environment detection in prime**
   - Detect if store is empty vs populated
   - Adjust guidance accordingly
   - Already have `--compact` and `--minimal` modes

9. **Pretty/tree output for graph**
   - `qipu link tree` exists, extend pattern
   - `qipu list --tree` for type hierarchy

---

## Error Handling & Intent Inference

### Overview

How tools handle mistakes reveals their design philosophy. Good error handling:
1. Identifies what went wrong
2. Suggests the correct action
3. Teaches the user for next time
4. Sometimes infers intent and offers alternatives

### Pattern Comparison

#### 1. Command Typos

**Beads**:
```
$ bd creat "Test"
Error: unknown command "creat" for "bd"

Did you mean this?
	create
	create-form
```

**Qipu**:
```
$ qipu creat "Test"
error: unrecognized subcommand 'creat'

  tip: a similar subcommand exists: 'create'
```

**Analysis**: Both use Levenshtein distance to suggest alternatives. Beads shows multiple matches; qipu shows the single best match.

#### 2. Flag Typos

**Beads**:
```
$ bd update qipu-fshe --statu in_progress
Error: unknown flag: --statu
Usage:
  bd update [id...] [flags]

Flags:
      --acceptance string      Acceptance criteria
      ...
```
*Shows full usage + all flags*

**Qipu**:
```
$ qipu create "Test" --typo permanent
error: unexpected argument '--typo' found

  tip: a similar argument exists: '--type'
```
*Suggests correct flag, minimal usage*

**Analysis**: Qipu is more helpful for typos - suggests the correct flag. Beads dumps full help which can overwhelm. Qipu wins here.

#### 3. Missing Required Arguments

**Beads**:
```
$ bd create
Error: title required (or use --file to create from markdown)
```
*Explains what's needed AND offers alternative*

```
$ bd dep add qipu-fshe
Error: requires 2 arg(s), only received 1 (or use --blocked-by/--depends-on flag)
Usage:
  bd dep add [issue-id] [depends-on-id] [flags]
...
```
*Shows missing count AND alternative flags*

**Qipu**:
```
$ qipu create
error: the following required arguments were not provided:
  <TITLE>

Usage: qipu create <TITLE>
```

**Analysis**: Beads offers alternatives (`--file`, `--blocked-by`). This is valuable - it teaches users about related features.

#### 4. Invalid Values

**Beads**:
```
$ bd create "Test" --priority high
Error: invalid priority "high" (expected 0-4 or P0-P4, not words like high/medium/low)
```
*Explains expected format AND what NOT to use*

```
$ bd create "Test" --type invalid_type
Error: operation failed: failed to create issue: validation failed: invalid issue type: invalid_type
```
*Could be improved - doesn't list valid types*

**Qipu**:
```
$ qipu create "Test" --type invalid
error: Invalid note type: 'invalid'. Valid types: fleeting, literature, moc, permanent
```
*Lists all valid values*

**Analysis**: Qipu wins on invalid value errors - it lists valid options. Beads priority error is good (explains format), but type error lacks the list.

#### 5. Domain-Specific Mistakes

**Beads**:
```
$ bd create "Test" --priority 2
⚠ Creating issue with 'Test' prefix in production database.
  For testing, consider using: BEADS_DB=/tmp/test.db ./bd create "Test issue"
✓ Created issue: ...
```
*Proactive warning about test data in production*

```
$ bd dep add qipu-fshe qipu-fshe
Error: operation failed: failed to add dependency: issue cannot depend on itself
```
*Domain rule enforced*

**Qipu**: No equivalent proactive warnings (could warn about empty captures, type demotion)

**Analysis**: Beads is proactive about common mistakes (test data in prod). This is valuable for preventing data pollution.

#### 6. Non-Existent Resources

**Beads**:
```
$ bd show nonexistent-xyz
Error: resolving ID nonexistent-xyz: operation failed: failed to resolve ID: no issue found matching "nonexistent-xyz"
```

**Qipu**:
```
$ qipu show nonexistent
error: note not found: nonexistent
```

**Analysis**: Qipu is more concise. Beads includes more context about the failure path. Both work.

#### 7. Doctor/Health Checks

**Beads** `bd doctor`:
```
bd doctor v0.49.6  ──────────────────────────────────────────  ✓ 73 passed  ⚠ 4 warnings  ✖ 1 errors

  ✖  1. Sync Divergence: 3 sync divergence issue(s) detected
        JSONL file differs from git HEAD: issues.jsonl                                         
        Uncommitted .beads/ changes (1 file(s))                                                
        └─ git add .beads/ && git commit -m 'sync beads' OR bd sync --import-only OR bd sync
  ⚠  2. Claude Integration: Not configured
        Claude can use bd more effectively with the beads plugin
        └─ Set up Claude integration:
            Option 1: Install the beads plugin (recommended)
            ...
```
*Each issue has: problem description + specific fix commands*

**Qipu** `qipu doctor`:
- Checks invariants
- Optional `--duplicates` check
- `--fix` for auto-repair
- No guidance text, just pass/fail

**Analysis**: Beads doctor is prescriptive - it tells you exactly what to run. Qipu is diagnostic - it reports status but doesn't guide fixes.

---

## Recommendations for Qipu Error Handling

### High Impact

1. **List valid values in error messages**
   ```rust
   // Current
   error: Invalid note type: 'invalid'
   
   // Better
   error: Invalid note type: 'invalid'. Valid types: fleeting, literature, moc, permanent
   ```
   *Already doing this for note types - extend to all enum values*

2. **Add proactive warnings**
   ```rust
   // Warn about empty captures
   $ qipu capture --title "Empty" < /dev/null
   ⚠ Capture has empty body. Consider adding content or using 'qipu create'.
   
   // Warn about type demotion
   $ qipu update qp-xxx --type fleeting  # was permanent
   ⚠ Demoting from permanent to fleeting. This may affect knowledge retention.
   ```

3. **Offer alternatives in missing-arg errors**
   ```rust
   // Current
   error: the following required arguments were not provided: <TITLE>
   
   // Better
   error: title required (or use --file to create from markdown, or pipe body to 'qipu capture')
   ```

### Medium Impact

4. **Add fix commands to doctor output**
   ```rust
   // Current
   Found 2 unlinked notes: qp-abc, qp-def
   
   // Better
   ⚠ Unlinked Notes: 2 notes have no incoming links
        qp-abc, qp-def
        └─ Run 'qipu link suggest' to find related notes
        └─ Or ignore if these are starting points
   ```

5. **Improve validation error messages**
   ```rust
   // Current (from update destroying body)
   [silently destroys body]
   
   // Better
   Error: update with --tag/--type/--value requires body via stdin
        To update metadata only: echo "" | qipu update <id> --tag foo
        To update body: cat new_body.md | qipu update <id>
   ```

### Lower Impact

6. **Suggest similar commands for typos** (already good)
7. **Group related commands in usage** (already discussed)

---

## Key Insight: Error Messages as Teaching Moments

Beads uses errors to teach:
- "not words like high/medium/low" - explains why the input was wrong
- "For testing, consider using: BEADS_DB=/tmp/test.db" - teaches about test isolation
- "OR bd sync --import-only OR bd sync" - shows multiple solutions

Qipu is more matter-of-fact:
- "note not found: nonexistent" - what happened, not what to do

For LLM agents, prescriptive errors are valuable because:
1. Agents can't "figure out" the fix through experimentation
2. Each error is a token cost - precise fixes minimize retries
3. Teaching in errors reduces need for documentation lookup
