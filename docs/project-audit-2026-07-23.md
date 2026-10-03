# Qipu Project Audit — 2026-07-23

**Scope:** engineering health, product/feature shape, and downstream-consumer fit (blibio).
**Method:** multi-agent audit — codex (architecture, product shape), opencode (robustness review + hands-on first-use trial as a target-user agent), lower-powered subagents (docs drift, testing/CI, dependencies, feature weight, competitive landscape, blibio integration map) — with all load-bearing claims verified directly against source before inclusion. Full hands-on trial transcript: `QIPU_AGENT_EVAL.md` (session scratchpad).

---

## Executive summary

Qipu is an unusually well-run project with a genuinely differentiated core: markdown-in-git as literal source of truth, a single dependency-free binary, and a curation discipline no competitor has. The engineering discipline is real (clean fmt/clippy, 3-OS CI, 500-line file cap, golden tests, signed releases, ADRs, an LLM-eval harness, a triaged issue backlog).

The audit initially read qipu as "three products sharing one CLI" — agent memory, a general knowledge-graph engine, and a publishing/exchange toolkit — with the latter two crowding out the first. **The blibio evaluation materially revises that verdict:** the knowledge-graph-engine mission has a real production consumer (blibio, the author's own consumption-graph application), and several features flagged as speculative (`custom` metadata, `value`, `verified`, provenance, `--custom-filter`) are load-bearing for it. Qipu is deliberately dual-mission: **an agent memory CLI and an embeddable knowledge-store engine.** The product problem is not that the second mission exists — it's that both missions share one undifferentiated surface, and each mission's features read as noise from the other's perspective.

Top-priority actions across all passes:

1. Atomic note-file writes (data integrity for the source of truth).
2. Machine-readable error contract for the JSON surface (blibio string-matches errors today).
3. Fix the five agent-facing conceptual footguns found in the hands-on trial (MOC direction, `--related` default, link-direction templates, init idempotency, jargon).
4. Native upsert (`capture --upsert` or equivalent) — blibio hand-rolls exists-then-branch.
5. Resolve the semantic-search expectation mismatch with blibio (qipu is deliberately lexical; blibio's docs assume embeddings).
6. `cargo audit` in CI, MSRV, version-sync preflight across the 7+ release manifests.

---

## Part 1 — Engineering audit

### Strengths

- fmt and clippy fully clean; `too_many_lines = "deny"` enforced; no file over ~500 lines.
- CI: fmt + clippy `-D warnings` + tests on ubuntu/macos/windows; release workflow builds 7 targets, signs with cosign, publishes to crates.io, packages deb/rpm.
- 258 test files (~40k lines), golden/snapshot tests, bench tests, and the ax-eval LLM harness with judge + composite scoring.
- Triaged beads backlog (31 open at audit time, incl. 5 P1 indexing/pack bugs).

### P1 — Data integrity (verified in source)

1. **Non-atomic note writes.** Note markdown files are the source of truth (ADR-0001) yet are written with plain `fs::write` — `crates/qipu-core/src/store/lifecycle.rs:49,93,152`, plus config writes in `store/io.rs`, `store/config.rs`. A crash mid-write truncates user data. Fix: temp file + `fs::rename` in the same directory. Highest-value small fix in the codebase. **Raised in priority by blibio:** its job queue drives concurrent qipu subprocesses with no client-side locking, so crash/contention windows are exercised in production, not hypothetically.
2. **DB corruption auto-rebuild** (`crates/qipu-core/src/db/mod.rs:52-93`). Lock/busy errors are correctly excluded, but corruption is detected by substring-matching stringified error text (`"corrupt"`, `"malformed"`) — forced by `QipuError::Other(String)` erasing rusqlite's structured codes — and `remove_file` races against other live processes. Mitigation: qipu.db is a derived, rebuildable index, so worst case is a rebuild, not data loss.

### P2 — Supply chain & release

3. No `cargo audit`/`cargo-deny` in CI; neither installed locally. No MSRV (`rust-version`) declared or tested.
4. Version manually synced across 7+ files; drift already present (flake.nix says 0.3.19 vs Cargo.toml 0.3.32; CHANGELOG missing 0.3.27–0.3.32, tracked as qipu-pu34). Add a version-sync preflight script to CI and the release gate. Pin flake's `rust-bin.stable.latest`.
5. `scripts/install.sh` / `install.ps1` (platform detection, musl fallback, checksum verification) are never executed in CI.
6. Minor: CI `target/` cache keyed only on Cargo.lock (consider `Swatinem/rust-cache`); RPM builds x86_64 only; `serde_yml 0.0.12` is a pre-1.0 fork of unmaintained serde-yaml with community quality concerns — it parses user frontmatter, worth re-evaluating.

### P3 — Architecture (codex findings, spot-verified)

Common root cause: the qipu-core/CLI boundary has eroded in both directions.

7. **Make `Store` the exclusive persistence seam.** `Database` is public; pack-load writes markdown and inserts rows itself (`src/commands/load/loader.rs`). Every parallel write path is a chance to violate the markdown↔index invariant — and it's where fix #1 must live once, not N times.
8. **Replace the monolithic error type.** Exactly 164 `QipuError::Other(String)` constructions. Typed per-module errors with preserved `source` fix corruption detection (#2), improve ADR-0007 failure guidance, and unblock the machine-readable error contract blibio needs (Part 3).
9. **Separate command execution from rendering.** ~59 command files print directly; 22 branch on output format independently. Centralize typed results + renderers — the JSON contract blibio parses can currently drift in any one file.
10. **Honor ADR-0004 in code:** pack loading and workspace merge implement conflict policy independently; add a typed `MergePolicy` in core with thin adapters.
11. Smaller: move CLI slice-selection (380 lines) into core; consolidate the four BFS/Dijkstra traversal variants; move `println!`/telemetry/git workflows out of qipu-core (directly serves the embeddable-engine mission); delete `allow(dead_code)` scaffolding (`QueryTimer`, unused error-chain API).

### Docs & hygiene (minor)

Docs are accurate overall. Refresh specs audit date (2026-02-09); dedupe the Project Structure section (README vs AGENTS.md); document `setup`/`onboard`/`quickstart`/`status` in the README reference.

---

## Part 2 — Product & feature audit

### Evidence base

- Full CLI surface dump: 33 top-level commands, 1,074 lines of help.
- Codex product-shape review; feature-weight/coverage table; hands-on first-use trial by an OpenCode agent (the literal target user); mid-2026 competitive research brief.

### The hands-on trial (strongest evidence)

The trial agent completed capture→link→search→context in minutes, praised short IDs, stdin capture, recovery-recipe errors, and `prime` — and said it would use qipu again. Every failure was **conceptual, not mechanical**:

1. **MOC membership direction is the #1 trap.** Agent linked `note --part-of--> MOC` (natural English). `link list` displayed the virtual `has-part` inverse — but `context --moc` and `inbox --exclude-linked` ignored it, silently requiring typed outbound `has-part` from the MOC. The tool's own display contradicted its selection semantics. Fix: make virtual inverses count for membership (preferred), or state the rule at `create --type moc` time and in `prime`.
2. **`context --related 0.3` default breaks closed-world expectations.** Requesting a specific MOC pulled an unrelated orphan via shared tags, labeled `Compaction: via=shared-tags` — colliding with the unrelated `compact` feature. Fix: default `--related 0` for explicit selectors; rename the provenance label (`Included-via:`).
3. **Asymmetric link verbs lack direction templates.** Agent stored `hypothesis supports paper` meaning the reverse. Add per-type English templates to `link add --help` and prime (`<evidence> supports <claim>`, `<moc> has-part <member>`).
4. **The hidden-surface strategy backfires on its target user.** `--with-body` is real, hidden (`src/cli/commands/data.rs:86`), documented in `specs/records-output.md`, and taught by `quickstart` — the agent cross-checked help, found nothing, and concluded the docs were stale. Hidden compat aliases (ADR-0006) + hidden flags + unexplained jargon (MOC, value, virtual, compaction, workspaces) make the agent's model diverge from actual behavior, and agents notice and lose trust.
5. Smaller: re-`init` on an existing store prints "Initialized" (agent feared a wipe); `prime` prescribes a git-commit ritual without detecting absence of git; `doctor --check ontology` reports nothing distinct; `-t` = title on `capture` but tag on `create`; `[P]/[F]/[L]/[M]` badges never defined.

### Surface shape

- Five commands answer "give me knowledge" (`show`/`search`/`context`/`prime`/`export`); four onboard (`prime`/`quickstart`/`onboard`/`setup`); three conflict-resolution systems (pack load / workspace merge / note merge); metadata mutation scattered across six verbs.
- Hollow top-levels: `store` (only `stats`), `ontology` (only `show`), `tags` (only `list`), `value` (set/show).
- `context` carries ~20 options (11 `--walk-*` tuning flags) — "a query language encoded as flags."
- Heaviest features by code weight: `doctor`, `link`, `context`, `export`, `compact` — while `capture` is ~263 lines and capture-time dedup doesn't exist.
- **21 of 33 commands have zero ax-eval coverage**; the harness tests the happy path, not the surfaces where the trial stumbled.

### Market position (mid-2026)

- **Differentiators nobody else combines:** git-native markdown source of truth (diffable/PR-reviewable/blame-able; even Beads layers a Dolt cache; mem0/cognee/zep need vector DBs or servers), single dependency-free binary, and a curation lifecycle when competitors are auto-capture firehoses.
- **Table-stakes gaps:** no MCP server (every named competitor has one; `prime --mcp` anticipates it); no semantic search (defensible trade-off — say so explicitly in docs); no auto-capture (a feature, but loses "it just remembers" demos).
- **Existential threat:** Claude Code's native Auto-Memory commoditizes basic cross-session recall. Qipu's answer must be *quality of curated knowledge*.
- **Positioning:** "Beads for knowledge instead of tasks; basic-memory without the vector index. Your project's zettelkasten, versioned like code, primeable into any agent, zero infrastructure."

---

## Part 3 — Downstream consumer: blibio (revises Part 2)

Blibio (`~/dev/blibio`, Go) is a personal knowledge graph built from consumption: browser extension/PWA → Go server (job queue, fetch, LLM extraction via OpenCode) → **qipu as the storage engine**. Its `internal/qipu/` package (2,669 lines, zero TODO/FIXME/HACK) shells out to the CLI with `--store <path> --format json` and parses JSON. Its architecture evaluation (`docs/architecture-decisions/qipu-backend-evaluation.md`) concluded "fully viable, production ready" — and qipu shipped features specifically to unblock it (`custom` command, `--custom-filter` range/date operators, stdin body replace on `update`, negative-value parsing, stdout/stderr separation).

### What blibio actually uses

capture (with `--id` as deterministic key), show `--custom`, update (+stdin body), delete, verify, inbox, list `--custom`/`--tag`, context (`--custom`, `--custom-filter`, `--min-value`, `--related`, `--limit`, `--query`), link add/remove/list/tree/path (7 typed link types), index, prime. Heavy use of `custom.*` (alignment, blibio_submission, dates, chunk provenance), `value`, `verified`, `generated_by`/`prompt_hash`.

**Not used:** workspaces, doctor, sync, compact, export/dump/pack, records format.

### How this revises the product verdict

| Prior verdict | Revised |
| --- | --- |
| `custom` metadata "speculative escape hatch" | **Load-bearing.** Core to blibio's data model (alignment, submission cross-refs, date filtering). Its hiddenness is a discoverability risk blibio itself flagged. |
| Manual 0–100 `value` "cut or rethink" | **Keep.** Maps directly to blibio's user impact signal; humans/apps set it, not agents. Retrieval semantics still need definition (what does value *do* in ranking?). |
| `verified`, provenance fields "adjacent" | **Core platform surface.** Blibio sets them on every chunk. |
| `context --custom-filter` complexity | **Justified** — it's blibio's query API. The unjustified complexity is specifically the 11 `--walk-*` flags, which blibio never touches. |
| "Three products in one CLI — cut the engine" | **Two deliberate missions, one confused surface.** Don't cut the engine; *separate the surfaces* (see recommendations). |
| Compaction, workspaces, packs, PDF export "speculative" | **Still speculative** — the real downstream consumer uses none of them. Verdict unchanged: freeze, don't expand. |
| MOC semantics footgun (trial finding) | **Confirmed independently:** blibio composes MOCs manually from capture + link add — there is no native collection API; every consumer re-derives the direction rule. |

### New gaps surfaced by blibio's integration

1. **No machine-readable error contract.** `isNotFoundError` (`blibio/internal/qipu/notes.go:68-73`) substring-matches "not found"/"does not exist" across stdout+stderr. Any wording change silently misclassifies errors. Fix: stable error codes in the JSON envelope + documented exit codes (pairs with engineering fix #8).
2. **No native upsert.** Blibio implements Exists→Show→branch(Update|Capture) with deterministic IDs — race-prone across concurrent workers. A `capture --upsert` (or `put`) is the single most valuable API addition for embedders.
3. **No CLI attachment surface.** `blibio/internal/qipu/attachments.go` manipulates `.qipu/attachments/` directly with `os.WriteFile` — hand-rolled hashing, sanitization, size caps — which qipu's own docs call an anti-pattern. Either add `qipu attach add/get/list/rm` or document the directory layout as a stable contract.
4. **Indexing contract unclear.** Blibio calls `qipu index` after every job, best-effort, because auto-index-on-write behavior isn't a documented guarantee. Document it (or make it one).
5. **JSON surface inconsistency** (blibio feedback items still open): `--format` not honored uniformly across subcommands; `list`/`show` can't opt into `value`/`verified`/`custom`/`path` fields uniformly.
6. **Semantic-search expectation mismatch (cross-project).** Blibio's `docs/specs/storage-schema.md` states "Qipu handles embeddings and semantic search internally… No embedding code in blibio." Qipu's `specs/similarity-ranking.md` explicitly avoids embeddings (TF-IDF/BM25 only). Blibio's retrieval quality ceiling is qipu's lexical matching, and its docs don't know it. Either align blibio's docs/expectations, or treat this as the first concrete demand signal for an optional embedding/hybrid layer in qipu.
7. **`--id` is a production API documented as "for testing and advanced use cases."** Help text undersells a flag a real application depends on — fold into the platform-surface docs.
8. **Concurrent-embedder reality.** Blibio's worker queue means multiple simultaneous qipu processes against one store is the *normal* production mode. Engineering fixes #1 and #2 move from "correctness hygiene" to "known production risk."

---

## Consolidated recommendations

### Now (days)

1. Atomic note/config writes (temp + rename).
2. Structured error codes in JSON output + documented exit codes; stop embedders string-matching.
3. Fix the five trial footguns: MOC membership (prefer honoring virtual inverses), `--related 0` default for explicit selectors + rename `Compaction:` provenance label, link-direction templates in help/prime, `init` idempotency message, one-line jargon definitions in prime (+ git detection).
4. `cargo audit` in CI; declare + test MSRV; version-sync preflight; fix flake.nix pin/version.

### Next (weeks)

5. Native upsert for capture; CLI attachment commands (or a documented stable attachment contract).
6. Turn trial failures and blibio's integration patterns into ax-eval scenarios (21/33 commands currently uncovered; the harness should encode the known failure modes and the embedder contract).
7. **Split the surface by mission** instead of cutting the engine: a small agent-facing default surface (~recall, capture, update, connect, curate + prime), and a documented *platform surface* (`custom`, `--id`, records format, attachment contract, JSON error envelope) promoted in `building-on-qipu.md` rather than hidden. Consolidate within the agent surface: merge capture/create (unify `-t`!), fold value/verify/tags/custom-set into `update` or `note get/set`, collapse quickstart/onboard/setup, unify export/dump.
8. Test install scripts in CI; document the indexing contract.

### Later (strategic)

9. One `recall "<task>" --budget N` command with excellent defaults, subsuming the `--walk-*` zoo for common use; retrieval-quality evaluation.
10. Capture-time duplicate detection (blibio dedups pre-insert in Go — second demand signal; `doctor --duplicates` is too late).
11. Freshness/supersession lifecycle (`last_confirmed`, `superseded-by`, review queue) — answers zep/graphiti's temporal pitch nearly free via git history.
12. Thin MCP shim over the core verbs, CLI remains ground truth.
13. Decide the semantic-search stance explicitly: either document "lexical by design" (and correct blibio's assumption), or spec an optional local-embedding hybrid as a platform feature with blibio as first customer.
14. Freeze compaction/workspaces/packs/PDF until the core loop and platform contract are polished; reconsider retrieval-feedback signals (used/corrected/ignored) as a complement to manual value.

### Meta-observation

Qipu's development loop (specs → ADRs → beads triage → ax-eval) is strong enough that the highest-leverage change is simply *feeding these findings into it*: the trial footguns become ax-eval scenarios, the blibio contract becomes a documented platform spec, and the surface split becomes an ADR.
