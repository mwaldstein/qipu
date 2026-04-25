# Pre-Release Conceptual Review

Status: Recommendations Ready  
Date: 2026-02-10
Updated: 2026-02-11

## Executive Summary

Qipu's core value proposition — making research and domain knowledge accumulate rather than evaporate — is strong and differentiated. The implementation is solid.

**Positioning shift**: qipu is now positioned as a "Knowledge graph CLI designed for scripts and agents" — emphasizing programmatic consumers over human direct use.

This document examines remaining positioning gaps, feature surface issues, and communication clarity.

## The Zettelkasten Question

### What Zettelkasten brings

The Zettelkasten model provides:
- A vocabulary for note types (fleeting, literature, permanent, MOC)
- A lifecycle narrative (raw capture → refined insight → curated collection)
- Semantic link types that go beyond "related"

### What it costs

**It front-loads concepts that most users won't use.** The four note types require understanding a taxonomy before first use. In practice:
- Agents will use `capture` and never specify a type (defaulting to fleeting)
- Scripts setting up agent integration will use `capture` and `context`
- Almost nobody will manually promote fleeting → literature → permanent
- MOCs are powerful but are a v2 workflow for most users

**It attracts the wrong comparisons.** "Zettelkasten CLI" invites comparison with Obsidian, Logseq, Zettlr — tools with richer note-taking UIs, visual graphs, and plugin ecosystems. Qipu loses that comparison on note-taking experience but wins on a dimension those tools don't compete on (agent-optimized retrieval). The Zettelkasten frame obscures the winning dimension.

### The real problem: Progressive Disclosure

The issue isn't "Zettelkasten" per se — it's **any methodology that front-loads vocabulary**. Replace "Zettelkasten" with "Graph-based knowledge system with typed edges" and you'd have the same problem: new users must learn terminology before they can be productive.

The system needs two paths:
1. **"Just works" path**: capture, search, context — no terminology required
2. **"Power user" path**: types, links, MOCs, ontology — available for those who want them

The Zettelkasten vocabulary is fine for the power user path. The problem is making it the on-ramp.

### Current state

Zettelkasten is now in README "Inspirations" section, not the tagline. This is the right call. The model (note types, typed links, MOCs) remains as internal implementation.

## Audience Clarity

### What qipu actually serves

Qipu is for **knowledge work** — the kind of knowledge that doesn't live in code and can't be found by grepping a codebase. Research findings, API behaviors discovered through trial and error, design rationale, domain context, things you learned from reading docs or debugging a third-party service. This knowledge currently lives in:

- Slack threads (unsearchable after a week)
- Meeting notes (never read again)
- Someone's head (unavailable when they're not around)
- Markdown files in `docs/` (flat, unlinked, stale)
- LLM conversation history (lost between sessions)

Qipu's value is making this knowledge **accumulate and remain retrievable** — whether the retriever is a person, an agent, or a tool chain.

### Where markdown knowledge bases fail

The most common "knowledge base" in a developer project is a `docs/` folder full of markdown. This works at small scale but breaks down predictably:

**Discovery becomes impossible.** With 50+ markdown files, you can't answer "what do I know about authentication?" without reading most of them. Grep finds keywords but not concepts. There's no ranking, no relevance, no way to surface the best content.

**Connections are invisible.** A design decision in `architecture.md` relates to a tradeoff documented in `auth-research.md` which contradicts a finding in `performance-notes.md`. These relationships exist only in the author's head. When someone new (or an agent) reads the docs, they see isolated documents, not a connected graph.

**Staleness is silent.** Which of your 80 markdown files are still accurate? Which supersede others? There's no mechanism to deprecate, score, or surface quality. Everything looks equally authoritative.

**Context assembly is manual.** When you need to give an LLM relevant background, you're copying and pasting from multiple files, guessing which ones matter, and hoping you stay within the context window. There's no budgeting, no automated selection, no "give me everything relevant to X in under 8000 characters."

**Scale makes it worse, not better.** A markdown knowledge base that works at 20 files becomes write-only at 200. The more you document, the harder it is to find what you documented. The incentive flips: people stop writing things down because nobody will find them anyway.

Qipu solves this by adding structure (typed notes, semantic links), retrieval (ranked search, graph traversal), quality signals (value scoring), and budgeted assembly (`context --max-chars`) on top of what are still just markdown files in a git repo.

### The target user

**1. LLM Agents**
- Session bootstrap via `qipu prime`
- Context assembly via `qipu context --max-chars`
- Deterministic output via `--format json/records`
- May use note types/links, or just capture/search

**2. Automation scripts**
- CI/CD pipelines capturing build findings
- Hooks that log discoveries
- Report generators that query the graph
- Integration bridges to other tools

**3. Humans (direct use)**
- Setup and configuration
- Ad-hoc queries
- MOC curation
- Quality maintenance (value scoring, verification)

**4. The team with siloed knowledge**
- Meeting notes, Slack threads, individual expertise
- "Alice's research dies when Alice leaves"
- Need shared, searchable, versioned knowledge

### The agent angle

Agent integration is a *capability* of qipu, not the whole story. `--format json`, `context --max-chars`, and `prime` make qipu agent-accessible, but the knowledge itself is valuable to humans too. The agent is one consumer of the graph, not the reason the graph exists.

**Critical architectural point**: Qipu is **agent-compatible, not agent-powered**. Per the specs:

> "Qipu must not require calling an LLM API" — `compaction.md`, `llm-context.md`

This is a deterministic tool that *prepares context* for external consumption. It doesn't use LLMs internally. This is a key differentiator from tools that embed AI features.

### Positioning recommendation

Position around the knowledge problem, not the agent solution. A user should understand qipu's value before they ever connect it to an agent. The pitch is: "your research compounds instead of disappearing."

**Important nuance**: Retrieval is the *differentiator*, but capture is the *prerequisite*. The user loop is fundamentally bidirectional:

```
capture → link → search/context → [agent uses it] → capture more
```

Leading with retrieval in quickstart is correct (immediate value demonstration), but don't undersell capture — without it, there's nothing to retrieve.

## Feature Surface Analysis

### Concepts a new user must learn (current)

Before productive use, a user currently encounters: note types (4), link types (10 + inverses), value scoring (0-100 with bands), MOCs, compaction, workspaces, custom ontology modes, inline vs typed links, semantic inversion, context selectors (walk, moc, tag, query, note, related, backlinks, min-value, custom-filter), and three output formats.

That's competitive with a database ORM, not a CLI tool.

### Concepts a new user should need (proposed)

1. **Notes** — text with a title and optional tags
2. **Links** — connections between notes (default type: `related`)
3. **Context** — a budgeted bundle of relevant notes for your agent

Everything else (note types, link type taxonomy, value scoring, MOCs, compaction, workspaces, ontology) should be discoverable through docs and `--help` but not required for the first hour of use.

### Feature-by-feature assessment

#### Clear value, keep prominent
- `capture` / `create` — core capture loop
- `context` — the killer feature, well-designed
- `search` — essential retrieval
- `link add` / `link tree` / `link path` — graph is the differentiator
- `prime` — great agent onboarding
- `doctor` — expected maintenance tool
- `--format json/records/human` — clean multi-audience output
- `init --stealth` — pragmatic for personal use
- `show` / `list` / `inbox` — basic CRUD

#### Clear value, but should be quieter at launch
- **Note types** — make `fleeting` the silent default, don't require users to choose
- **Link types** — default to `related`, mention others in docs
- **Value scoring** — useful for large stores, but not needed in first session
- **MOCs** — powerful organizational tool, but an intermediate concept
- **Custom ontology** — excellent extension point, belongs in "Building on Qipu" not first-run
- **Export** — solid feature, document but don't highlight
- **Provenance fields** (`--generated-by`, `--prompt-hash`, `--verified`) — important for trust, but agent-facing not user-facing

#### Adds complexity without clear v1 value
- **Compaction** — solves a real problem (large stores) that v1 users won't have. Currently leaks into every command's `--help` via global flags. Should be hidden from default help or feature-gated.
- **Workspaces** — this is primarily an **agent feature** for sub-agent isolation (see `specs/workspaces.md`: "Agent runs `qipu workspace new research-x --from-tag X --temp`, uses it, then merges back"). Not organizational complexity for humans, but should be documented as agent-specific.
- **Telemetry** — implemented with `enable`/`disable`/`status`/`show` commands and explicit opt-out. Not "undocumented telemetry" — but missing from README. Add a brief mention.
- **10 link types at launch** — CLI help only shows 5 (`related`, `derived-from`, `supports`, `contradicts`, `part-of`). The other 5 exist but aren't exposed in help. Either expose all 10 or reduce to 3-4 common ones with others via custom ontology.

### Missing features users would expect
- **`delete`** — no way to remove a note from the CLI
- **`promote`** — if the Zettelkasten lifecycle is kept, there should be a first-class command for type progression
- **Bulk operations** — tag/link/value multiple notes at once
- **Status dashboard** — "you have N notes, M orphans, your most-connected topic is X" would make the graph tangible

## The Quick Start Test

A tool's quick start reveals its mental model. Current:

```bash
qipu init
echo "TIL: ..." | qipu capture --title "Rust question mark"
qipu link add <new-id> <existing-id> --type derived-from
qipu search "rust error handling"
qipu prime
```

This requires: init, capture, understand IDs, choose a link type, search, and know what prime does. That's five concepts and a vocabulary decision (what does `derived-from` mean?) in the first minute.

Proposed:

```bash
qipu init
echo "TIL: Rust's ? operator works with Option too" | qipu capture --title "Rust question mark"
qipu context --query "rust"
```

Three commands. The value is immediately visible: you put knowledge in, you get relevant knowledge out. Linking, types, and scoring can come later as the store grows.

## Competitive Positioning

### What qipu competes with

| Competitor | Stars | Qipu's advantage | Qipu's disadvantage |
|------------|-------|------------------|---------------------|
| **No tool** (docs/, Slack, memory) | — | Structured, retrievable, versioned | Requires setup, learning curve |
| **Notion/Confluence** | — | Local-first, git-backed, CLI-native | No rich UI, no collaboration features |
| **Obsidian/Logseq** | — | Agent-optimized output, budgeted context, LLM-free | No visual graph, no plugin ecosystem |
| **Mem0** | 47k | LLM-free, local-first, deterministic | No managed service, no auto-extraction |
| **Letta/MemGPT** | 21k | Simple CLI vs full agent platform, no LLM dependency | No autonomous agents, no self-improvement |
| **LlamaIndex** | 47k | Ready-to-use CLI vs framework, no code required | Less flexible, fewer integrations |
| **Memory files** (CLAUDE.md, AGENTS.md) | — | Searchable, typed links, traversal, budgeted | More overhead than single file |

### The real competition

For most potential users, the alternative isn't another tool — it's **nothing**. They capture knowledge in:
- Chat messages that scroll away
- Notes that never get read again
- Mental models that leave when people leave

The pitch should meet them where they are: "Your research is already evaporating. Here's how to make it stick."

### Detailed competitive analysis

See "Competitive Landscape Analysis" section below for detailed breakdown of Mem0, Letta, LlamaIndex, and Obsidian.

## Competitive Landscape Analysis

### LLM Memory Solutions

**Mem0** (47k GitHub stars)
- **What it is**: "Universal memory layer for AI Agents" — managed service + SDK
- **Model**: Cloud-first, requires LLM API to function, auto-extracts memories from conversations
- **Pricing**: Freemium hosted service
- **Key claim**: "+26% accuracy vs OpenAI Memory, 91% faster, 90% fewer tokens"
- **Qipu differentiation**: Mem0 is LLM-powered (requires calling an LLM). Qipu is deterministic, local-first, no external dependencies. Mem0 optimizes for chatbot personalization; qipu optimizes for developer knowledge accumulation.

**Letta/MemGPT** (21k GitHub stars)
- **What it is**: "Stateful agents with advanced memory that can learn and self-improve"
- **Model**: Cloud-first, managed agents with self-editing memory, requires LLM API
- **Key claim**: Agents that write to their own memory, continual learning
- **Qipu differentiation**: Letta is about *autonomous agents managing their own memory*. Qipu is about *tools/scripts/agents having access to a shared knowledge graph*. Different problem spaces. Letta = agent brains; qipu = project knowledge.

**LlamaIndex** (47k GitHub stars)
- **What it is**: "Framework for building LLM-powered agents over your data"
- **Model**: RAG framework, data connectors, vector stores, query engines
- **Key claim**: Connect any data source to any LLM
- **Qipu differentiation**: LlamaIndex is a framework for *building* RAG systems. Qipu is a *ready-to-use* knowledge CLI. LlamaIndex requires code; qipu requires a shell. They're complementary — LlamaIndex could use qipu as a data source.

### Knowledge Management Tools

**Obsidian**
- **What it is**: Personal knowledge management with visual graph, plugin ecosystem
- **Model**: GUI-first, local markdown files, human-centric
- **Key claim**: "Sharpen your thinking" — personal PKM, journaling, notes
- **Qipu differentiation**: Obsidian is human-focused with rich UI. Qipu is agent/script-focused with deterministic output. Obsidian has plugins, visual graph, mobile apps. Qipu has `--format json`, budgeted context, CLI automation.

**Logseq**
- **What it is**: Privacy-first, open-source knowledge base
- **Model**: GUI-first, outliner-based, local files
- **Qipu differentiation**: Same as Obsidian — human-centric UI vs agent-centric CLI.

### The Key Differentiation Matrix

| Feature | Qipu | Mem0/Letta | Obsidian/Logseq | LlamaIndex |
|---------|------|------------|-----------------|------------|
| LLM-free operation | ✅ | ❌ | ✅ | N/A (framework) |
| CLI-first | ✅ | ❌ (SDK) | ❌ (GUI) | ❌ (Python lib) |
| Deterministic output | ✅ | ❌ | ❌ | ✅ |
| Budgeted context | ✅ | ❌ | ❌ | ✅ (requires code) |
| Git-native | ✅ | ❌ | ✅ | ❌ |
| Typed semantic links | ✅ | ❌ | ❌ (untyped) | ✅ (via code) |
| Agent-optimized output | ✅ | ✅ | ❌ | ✅ |

### Qipu's Unique Position

**The only tool that is:**
1. LLM-free (deterministic, no API keys, works offline)
2. CLI-native (scriptable, automatable, CI/CD-friendly)
3. Agent-optimized (JSON/records output, budgeted context)
4. Git-backed (versioned, mergeable, portable)
5. Structured (typed notes, semantic links, value scoring)

**Closest competitors by use case:**
- "I want my LLM to remember things" → Mem0, Letta
- "I want to organize my personal notes" → Obsidian, Logseq  
- "I want to build a RAG system" → LlamaIndex
- "I want my project knowledge to accumulate and be retrievable by scripts/agents" → **Qipu**

## Recommendations

### 1. Positioning: Lead with the Problem, Not the Solution

**Current tagline**: "Knowledge graph CLI designed for scripts and agents"

**Problem**: This describes *what* qipu is, not *why* someone needs it.

**Recommended alternatives**:

1. **"Your research compounds instead of disappearing"** — emotional, problem-focused
2. **"Persistent knowledge for projects that outlast sessions"** — developer-focused
3. **"The knowledge base that scripts and agents can actually use"** — differentiator-focused

**Recommendation**: Use #1 for tagline, expand with #3 in description.

### 2. Quick Start: Demonstrate Value in 3 Commands

**Current**: 5+ concepts in quickstart (init, capture, link, search, prime)

**Recommended**:
```bash
qipu init
echo "TIL: Rust's ? operator works with Option too" | qipu capture --title "Rust question mark"
qipu context --query "rust"
```

This shows: put knowledge in → get relevant knowledge out. Immediate value.

### 3. Hide Advanced Features from First-Run Experience

**Hide from default `--help` and quickstart:**
- Compaction (global `--compact-*` flags)
- Workspaces (document in "Building on Qipu")
- Custom ontology (document in "Building on Qipu")
- Telemetry commands (keep but don't advertise)
- Provenance fields (document but don't highlight)

**Keep prominent:**
- capture, context, search, link (core loop)
- prime (agent onboarding)
- doctor (maintenance expectation)
- `--format json/records` (programmatic use)

### 4. Competitive Messaging

**Against "no tool":** 
> "Your research is already evaporating. Meeting notes nobody reads, Slack threads that disappear, knowledge that leaves when people leave. Qipu makes knowledge accumulate."

**Against Obsidian/Logseq:**
> "Great for humans, but your LLM agent can't use them. Qipu is built for programmatic access: `--format json`, budgeted context, deterministic output."

**Against Mem0/Letta:**
> "They require calling an LLM. Qipu is deterministic — no API keys, no cloud dependency, works offline. Your knowledge graph isn't dependent on an external AI service."

**Against memory files (CLAUDE.md, AGENTS.md):**
> "One file doesn't scale. Qipu gives you search, ranking, graph traversal, and context budgeting — without giving up the simplicity of markdown files in git."

### 5. Release Checklist

**Documentation updates:**
- [ ] Simplify README quickstart to 3 commands
- [ ] Add "Why Not Just..." section (markdown files, Obsidian, Mem0)
- [ ] Document telemetry in README (transparency)
- [ ] Create "Getting Started with Agents" guide

**CLI adjustments:**
- [ ] Hide compaction flags from default `--help`
- [ ] Consider `qipu delete` for v1 (basic expectation)
- [ ] Add `qipu status` or enhance `prime` with store health summary

**Marketing materials:**
- [ ] Landing page with problem/solution framing
- [ ] Comparison page with Mem0, Obsidian, markdown folders
- [ ] Example agent integration (AGENTS.md snippet)

### 6. Success Metric for 1-Week Users

After one week of use, a user should be able to:
1. Run `qipu context --query <topic>` and get useful results
2. See that their `qipu capture` notes are being retrieved
3. Have their LLM agent use qipu knowledge without manual copy-paste

This suggests the quickstart should emphasize:
- Capture something immediately
- Retrieve it immediately
- Show the agent integration path

## Open Questions

### From original analysis
1. Is the link type taxonomy worth maintaining at 10 types, or should the default set be reduced to 3-4 with others available via custom ontology? (CLI help currently shows 5, not 10.) **Recommendation: Keep 5 in help, document all 10 in docs.**
2. Should compaction and workspaces ship in v1 but be hidden from `--help`, or be feature-gated? **Recommendation: Ship but hide from default help.**
3. Does `inbox` serve the agent use case at all, or is it purely a human Zettelkasten workflow feature? **Answer: Human-focused, agents use search/context.**
4. ~~Is `prime` the right name?~~ **Decision: Keep `prime` — follows beads pattern for LLM familiarity.**

### Decisions
5. **Is "scriptable" the right differentiator?** → The real differentiator is "LLM-free + agent-optimized". Scriptable is a feature, not the positioning.
6. **Is the beads alignment helping or hurting?** → **Confirmed: Keep beads patterns (`prime`, `setup`, hooks).** Hypothesis: following established patterns makes it easier for LLMs to learn and use the tool.
7. **What's the success metric?** → User can `qipu context --query X` and get useful results after 1 week.
8. **Should `delete` be v1?** → Yes, basic expectation. Users will ask for it.
9. **Link type help should list all 10 or keep showing 5?** → Keep showing 5 in help, document all 10 in semantic-graph docs.
10. **Keep `prime` naming?** → **Confirmed: Yes.** Follows beads pattern, creates consistency for LLMs familiar with `bd prime`.

## Summary

Qipu is well-positioned as the **only LLM-free, CLI-native, git-backed knowledge graph** with agent-optimized output. The main competition is "nothing" — users who don't capture knowledge at all, or capture it in ways that don't accumulate.

**Key release actions:**
1. Simplify quickstart to 3 commands demonstrating immediate value
2. Lead with the problem ("research evaporates") not the solution ("knowledge graph CLI")
3. Hide advanced features (compaction, workspaces, ontology) from first-run experience
4. Add explicit competitive messaging against Mem0, Obsidian, and markdown folders
5. Ship `delete` as a basic v1 expectation
