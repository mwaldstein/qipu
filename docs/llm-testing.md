# LLM Agent Evaluation Guide

This guide explains how to use the [ax-eval](https://github.com/mwaldstein/ax-eval) harness to validate LLM coding agents (e.g., OpenCode, Claude Code, Codex) against Qipu.

Status: Draft
Last updated: 2026-06-16

## Philosophy

ax-eval does **not** treat scenarios as binary pass/fail tests. Most scenarios eventually succeed — the interesting questions are:

1. **How efficiently?** Did the agent get it right on the first try, or fumble with `--help` and retry commands multiple times?
2. **How quickly?** Wall-clock time and command count matter.
3. **How well?** Quality of the resulting notes, links, and knowledge graph structure.

This enables comparison across agent tools and models to understand which combinations are best-tuned for qipu usage.

## Evaluation Dimensions

### Interaction Quality (Automated)
- **Command count**: Total qipu commands executed
- **Error count**: Commands that failed
- **Retry count**: Same command attempted multiple times
- **Help invocations**: How often the agent needed `--help`
- **First-try success rate**: % of commands correct on first attempt
- **Iteration ratio**: Follow-up commands relative to unique commands

### Quality (Automated + Judge)
- **Note structure**: Titles, tags, types, body length
- **Graph connectivity**: Links per note, orphan notes, MOC coverage
- **Semantic quality**: Relevance, coherence, granularity (LLM-judged via rubric)

### Efficiency
- **Duration**: Wall-clock time to complete
- **Token usage**: API tokens consumed (when the adapter reports them)
- **Cost**: Reported by the adapter when available (e.g., opencode)

## Overview

ax-eval runs reproducible scenarios that:
1. Set up a fresh Qipu store from a fixture
2. Execute an LLM agent with a task prompt
3. Capture every command, error, and token as structured interaction evidence
4. Optionally run LLM-as-judge evaluation for rubric scoring
5. Write a dimensional evaluation profile (metrics, transcript, report)

This enables regression testing and cross-tool/model comparison.

## Installation

Install the latest ax-eval release on macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/mwaldstein/ax-eval/master/scripts/install.sh | sh
```

On Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/mwaldstein/ax-eval/master/scripts/install.ps1 | iex
```

Or build from source:

```bash
git clone https://github.com/mwaldstein/ax-eval
cd ax-eval
cargo build --release   # binary at target/release/ax-eval
```

ax-eval is pre-1.0 and not yet published to crates.io.

## Safety Flag

`AX_EVAL_ENABLED=1` is **real-run consent**. ax-eval will not launch an agent
adapter unless this variable is set, because real runs may spend LLM API
credits and execute agent-driven CLI commands.

```bash
export AX_EVAL_ENABLED=1
```

Use `--dry-run` to validate scenario selection, fixture setup, cache keys, and
run planning **without** the safety flag or any LLM spend.

## Quick Start

```bash
# List available scenarios (no env var needed)
ax-eval scenarios

# Validate a scenario's YAML without running
ax-eval validate --scenario ax-eval-fixtures/capture_basic.yaml

# Dry run: parse, set up fixtures, compute cache keys — no LLM
ax-eval run --scenario capture_basic --dry-run

# Real run (requires AX_EVAL_ENABLED=1)
AX_EVAL_ENABLED=1 ax-eval run --scenario capture_basic --tool opencode

# Run all scenarios with Claude Code
AX_EVAL_ENABLED=1 ax-eval run --all --tool claude-code
```

> The agent must be able to find the qipu binary. For local development, put the
> build on PATH before running:
> `PATH="$PWD/target/debug:$PATH" AX_EVAL_ENABLED=1 ax-eval run --scenario capture_basic --tool opencode`

## Prerequisites

### Agent Tool Availability

The harness checks that the selected adapter is available before running. Supported runtime adapters:

- `opencode`
- `claude-code`
- `codex`

Each must be installed and authenticated.

### API Keys (for LLM-as-judge)

If your scenario uses judge evaluation, the judge tool must be authenticated.
The judge defaults to the configured judge tool/model (see `--judge-tool` /
`--judge-model`).

## Configuration

An optional `ax-eval-config.toml` in the workspace root customizes fixture/result
paths, tool/model validation, and matrix profiles:

```toml
fixtures_path = "ax-eval-fixtures"
results_path = "ax-eval-results"

[tools.opencode]
name = "opencode"
command = "opencode"
models = ["gpt-4o", "claude-sonnet"]

[tools.claude-code]
name = "claude-code"
command = "claude"
models = ["claude-sonnet", "claude-haiku"]

# Profiles build a Cartesian product of tools x models
[profiles.quick]
name = "quick"
tools = ["opencode"]
models = ["gpt-4o"]
```

Copy `ax-eval-config.example.toml` from the ax-eval repository as a starting
point, or print one with `ax-eval template config > ax-eval-config.toml`.

## Commands

### `scenarios` - List available scenarios

```bash
ax-eval scenarios [--tags <TAGS>] [--tier <TIER>]
# tier: 0=smoke, 1=quick, 2=standard, 3=comprehensive (default: 0)
```

### `run` - Execute scenarios

```bash
ax-eval run [OPTIONS]

Options:
  -s, --scenario <SCENARIO>   Path to scenario file or name
      --all                   Run all scenarios in the fixtures directory
      --tags <TAGS>           Filter scenarios by tags
      --tier <TIER>           Filter by tier [default: 0]
      --tool <TOOL>           Agent tool (opencode, claude-code, codex)
      --model <MODEL>         Model to use with the tool
      --profile <PROFILE>     Configured matrix profile (from config)
      --dry-run               Validate without invoking an LLM
      --no-cache              Disable caching
      --judge-model <MODEL>   Judge model for LLM-as-judge evaluation
      --judge-tool <TOOL>     Judge tool (defaults to judge config or opencode)
      --no-judge              Disable LLM-as-judge evaluation
      --timeout-secs <SECS>   Maximum execution time per command [default: 300]
```

### `validate` - Check scenario YAML

```bash
ax-eval validate --scenario ax-eval-fixtures/capture_basic.yaml
ax-eval validate --all
```

Catches missing fields, unknown gate types, invalid regexes, and misconfigured
judge/composite weights — no fixture setup, no LLM spend.

### `discover` - Evaluate a target CLI before writing scenarios

```bash
AX_EVAL_ENABLED=1 ax-eval discover qipu --tool opencode
```

Asks an agent to inspect qipu, author five goal-oriented scenarios, run them,
judge usage quality, and summarize the results. Use it to bootstrap a new
scenario set or to gauge how self-describing qipu is.

### `show` - Display a saved run

```bash
ax-eval show <run-id>
```

### `clean` - Clear cache and legacy artifacts

```bash
ax-eval clean [--older-than <DURATION>]   # e.g. "30d", "7d", "1h"
```

### `template` - Print copyable schemas

```bash
ax-eval template scenario > ax-eval-fixtures/my_scenario.yaml
ax-eval template config > ax-eval-config.toml
ax-eval template script-gate
ax-eval template evaluator
```

## Scenarios

Scenarios are YAML files defining the agent task, the environment, and post-run
evaluation. Example from `ax-eval-fixtures/capture_basic.yaml`:

```yaml
name: capture_basic
description: "Basic note capture scenario"
template_folder: qipu
tier: 1
tags: [capture, smoke]
target:
  binary: qipu
  env:
    QIPU_STORE: "${AX_EVAL_FIXTURE_DIR}/.qipu"
task:
  prompt: "Initialize qipu if needed. Capture one literature note about quantum entanglement..."
interaction:
  target_commands: required
evaluation:
  gates:
    - type: command_output_contains
      command: qipu search entanglement --format records
      substring: "Quantum Entanglement"
    - type: script
      description: "The captured note has expected tags and searchable content"
      command: |
        set -e
        qipu list --tag physics --format records | grep -q "Quantum Entanglement"
        qipu context --query entanglement --format records --max-chars 8000 | grep -q entanglement
  judge:
    enabled: true
    rubric: rubrics/capture_v1.yaml
    pass_threshold: 0.7
```

### Scenario Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier |
| `description` | Yes | Human-readable description |
| `template_folder` | Yes | Name of fixture directory under `ax-eval-fixtures/templates/` |
| `target.binary` | Yes | Binary under test, usually `qipu` |
| `target.env` | No | Env vars; supports `${AX_EVAL_FIXTURE_DIR}` / `${AX_EVAL_RESULTS_DIR}` placeholders |
| `task.prompt` | Yes | The prompt given to the agent |
| `interaction.target_commands` | No | `required` (default), `optional`, or `forbidden` |
| `evaluation.gates` | No | Pass/fail guardrail checks |
| `evaluation.judge` | No | LLM-as-judge configuration |

### Fixtures Location

Test fixtures and scenarios live in `ax-eval-fixtures/` at the workspace root:

- **templates/qipu/**: Test environment templates (AGENTS.md, README.md)
- **rubrics/**: Evaluation rubrics for LLM-as-judge scoring
- **\*.yaml**: Scenario definitions

### Gate Types

Gates are binary assertions evaluated after the agent completes. Use them as
guardrails for catastrophic failures — the richer signal is in the metrics and
judge score.

| Type | Parameters | Description |
|------|------------|-------------|
| `command_succeeds` | `command: "qipu ..."` | Command exits successfully |
| `command_output_contains` | `command`, `substring` | Command stdout contains expected text |
| `command_output_matches` | `command`, regex field | Command stdout matches a pattern |
| `command_json_path` | `command`, JSON path fields | JSON command output has expected data |
| `file_exists` | file path field | Expected file exists |
| `file_contains` | file path, substring | Expected file contains text |
| `file_matches` | file path, regex | Expected file matches a pattern |
| `no_transcript_errors` | none | Fails when target-tool command errors appear |
| `script` | `description`, `command` | Shell script exits successfully; best for multi-step qipu assertions |

Prefer outcome gates (`file_exists`, `command_json_path`, `script`) over
`no_transcript_errors`. Treat command errors as interaction-quality metrics.

## Rubrics

Rubrics define criteria for LLM-as-judge evaluation:

```yaml
criteria:
  - id: command_correctness
    weight: 0.25
    description: "Uses valid qipu commands with correct syntax"

  - id: structure_quality
    weight: 0.30
    description: "Notes are well-organized with meaningful links"

  - id: coverage
    weight: 0.30
    description: "Captures key concepts without major omissions"

  - id: retrieval_success
    weight: 0.15
    description: "Can retrieve captured knowledge via search/show"

output:
  format: json
  require_fields:
    - scores
    - weighted_score
    - confidence
    - issues
    - highlights
```

Weights must sum to 1.0.

## Results

Results are stored in `ax-eval-results/` (configurable via `results_path`):

```
ax-eval-results/
├── results.jsonl            # Append-only run records
└── <timestamp>-<tool>-<model>-<scenario>/
    ├── artifacts/
    │   ├── events.jsonl          # Structured event log
    │   ├── transcript.raw.txt    # Full agent transcript
    │   ├── tool-output.raw.txt   # Raw adapter output when available
    │   └── command-events.json   # Normalized command events when available
    ├── evaluation.md             # Human-readable evaluation profile
    ├── metrics.json              # Machine-readable metrics
    └── report.md                 # Execution details + gate results
```

**`metrics.json` example:**

```json
{
  "gates_passed": 3,
  "gates_total": 3,
  "efficiency": {
    "total_commands": 6,
    "unique_commands": 5,
    "error_count": 0,
    "retry_count": 1,
    "help_invocations": 1,
    "first_try_success_rate": 1.0,
    "iteration_ratio": 0.83,
    "completed": true
  },
  "interaction_evidence_source": "structured_tool_calls"
}
```

Judge, composite, and custom evaluator fields appear only when configured.

### Evidence Source

The `interaction_evidence_source` field in `metrics.json` shows how command
metrics were built:
- `structured_tool_calls` — the adapter provided canonical command events.
- `transcript_regex_fallback` — metrics came from transcript regex analysis.

Structured-capable adapters must not use the fallback. If you see fallback
evidence from opencode/claude-code/codex, the adapter's raw-output parser likely
no longer matches the CLI output schema.

### Caching

Results are cached by a composite key (scenario YAML hash, prompt hash, tool
name, and target git commit). Use `--no-cache` to force re-execution.

## Writing New Scenarios

1. Print a template to start from:

```bash
ax-eval template scenario > ax-eval-fixtures/my_scenario.yaml
```

2. Edit it (example with two linked notes):

```yaml
name: my_scenario
description: "Test description"
template_folder: qipu
target:
  binary: qipu
task:
  prompt: |
    Initialize qipu if needed.
    Create two notes about related topics and link them together.
    Use the 'related' link type.
evaluation:
  gates:
    - type: script
      description: "Two notes exist and a related link connects them"
      command: |
        set -e
        count=$(qipu list --format records | grep -c '^N ')
        test "$count" -eq 2
        qipu link list "$(qipu list --format records | awk '/^N / {print $2; exit}')" --format records | grep -q related
```

3. Validate, then dry-run, then run for real:

```bash
ax-eval validate --scenario ax-eval-fixtures/my_scenario.yaml
ax-eval run --scenario my_scenario --dry-run
AX_EVAL_ENABLED=1 ax-eval run --scenario my_scenario --tool opencode
```

## Interpreting Results

### Gates

Gates are pass/fail guardrails. All gates must pass for the scenario to pass,
but pass/fail is a coarse signal — inspect the metrics and transcript for the
real evaluation profile.

### Judge Scores

Judge evaluation returns a weighted score (0.0 to 1.0) with rationale,
confidence, issues, and highlights. Use it as a repeatable qualitative signal
across runs.

## Troubleshooting

### "Real LLM tool runs require AX_EVAL_ENABLED=1"

Set the safety flag to consent to real runs, or use `--dry-run` to validate
without an LLM.

### Tool unavailable / not supported

Use one of the runtime adapters: `opencode`, `claude-code`, or `codex`. Ensure
it is installed and authenticated:

```bash
which opencode && opencode --version
```

### Scenario not found

Check the fixtures directory and list discovered scenarios:

```bash
ls ax-eval-fixtures/
ax-eval scenarios
```

### Long-running scenarios

Increase the per-command timeout:

```bash
AX_EVAL_ENABLED=1 ax-eval run --scenario capture_basic --timeout-secs 600
```

### Cache issues

Disable caching with `--no-cache` or run `ax-eval clean`.

## Known Limitations

- No multi-turn interaction support (single prompt per scenario)
- No parallel scenario execution
- Cost is reported only when the adapter self-reports it

For the complete scenario schema and evaluator scripts, see the
[ax-eval docs](https://github.com/mwaldstein/ax-eval/tree/master/docs).
