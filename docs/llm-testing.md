# LLM Tool Testing Guide

This guide explains how to use the `llm-tool-test` harness to validate LLM coding tools (e.g., OpenCode) against Qipu.

Status: Draft  
Last updated: 2026-01-31

## Philosophy

The LLM tool tests are **not binary pass/fail tests**. The expectation is that most scenarios will eventually succeed—the questions are:

1. **How efficiently?** Did the LLM get it right on the first try, or did it fumble with `--help` and retry commands multiple times?
2. **How quickly?** Wall-clock time and command count matter.
3. **How well?** Quality of the resulting notes, links, and knowledge graph structure.

This enables comparison across tools and models to understand which combinations are best-tuned for qipu usage.

## Evaluation Dimensions

### Efficiency (Automated)
- **Command count**: Total qipu commands executed
- **Error rate**: Commands that failed before succeeding
- **Retry count**: Same command attempted multiple times
- **Help invocations**: How often did it need `--help`?
- **First-try success rate**: % of commands correct on first attempt

### Quality (Automated + Judge)
- **Note structure**: Titles, tags, types, body length
- **Graph connectivity**: Links per note, orphan notes, MOC coverage
- **Semantic quality**: Relevance, coherence, granularity (LLM-judged)

### Cost/Speed
- **Duration**: Wall-clock time to complete
- **Token usage**: API tokens consumed (when available)
- **Commands per minute**: Throughput indicator

## Overview

The LLM tool test harness runs automated scenarios that:
1. Set up a fresh Qipu store from a fixture
2. Execute an LLM tool with a task prompt
3. Analyze the transcript for efficiency metrics
4. Analyze the resulting store for quality metrics
5. Optionally run LLM-as-judge evaluation for semantic scoring
6. Optionally pause for human review

This enables regression testing and cross-tool/model comparison.

## Quick Start

```bash
# Build the harness
cargo build -p llm-tool-test

# List available scenarios (no env var needed)
llm-tool-test scenarios

# Run all scenarios (requires LLM_TOOL_TEST_ENABLED=1)
export LLM_TOOL_TEST_ENABLED=1
llm-tool-test run --all --tool opencode --model mock

# Run a specific scenario with dry run
llm-tool-test run \
  --scenario llm-test-fixtures/capture_basic.yaml \
  --tool opencode \
  --dry-run
```

## Prerequisites

### Tool Availability

The harness checks that the specified tool is available before running:

- **amp**: Must be in PATH and respond to `amp --version`
- **opencode**: Must be in PATH and respond to `opencode --version`

 ### API Keys (for LLM-as-judge)

 If your scenario uses judge evaluation, set:

 ```bash
 export OPENAI_API_KEY="sk-..."
 # or
 export LLM_TOOL_TEST_API_KEY="sk-..."
 ```

### Configuration

Create a `llm-tool-test-config.toml` in your workspace root:

```bash
cp crates/llm-tool-test/llm-tool-test-config.example.toml llm-tool-test-config.toml
```

**Configuration options:**

```toml
# Path to test fixtures (default: "llm-test-fixtures")
fixtures_path = "llm-test-fixtures"

# Path for test results (default: "llm-tool-test-results")
results_path = "llm-tool-test-results"

[models.claude-3-5-sonnet]
input_cost_per_1k_tokens = 3.0
output_cost_per_1k_tokens = 15.0
```

## Commands

### `scenarios` - List available scenarios

```bash
llm-tool-test scenarios [OPTIONS]

Options:
      --tags <TAGS>    Filter scenarios by tags
      --tier <TIER>    Filter scenarios by tier [default: 0]
```

### `run` - Execute a scenario

```bash
llm-tool-test run [OPTIONS]

Options:
  -s, --scenario <SCENARIO>   Path to scenario YAML file or name
      --all                   Run all scenarios
      --tags <TAGS>           Filter scenarios by tags
      --tier <TIER>           Filter scenarios by tier [default: 0]
      --tool <TOOL>           Tool to test [default: opencode]
      --model <MODEL>         Model to use
      --tools <TOOLS>         Multiple tools for matrix run (comma-separated)
      --models <MODELS>       Multiple models for matrix run (comma-separated)
      --dry-run               Parse and validate without executing
      --no-cache              Disable caching
      --judge-model <MODEL>   Judge model for evaluation
      --no-judge              Disable LLM-as-judge evaluation
      --timeout-secs <SECS>   Maximum execution time [default: 300]
      --max-usd <USD>         Maximum budget in USD for this session
```

### `show` - Display scenario details

```bash
llm-tool-test show <NAME>
```

### `clean` - Clear cache

```bash
llm-tool-test clean [--older-than <DURATION>]
```

## Scenarios

Scenarios are YAML files that define a test case:

```yaml
name: capture_basic
description: "Basic note capture scenario"
fixture: qipu
task:
  prompt: "Create a note about quantum entanglement with some basic facts."
evaluation:
  gates:
    - type: min_notes
      count: 1
    - type: search_hit
      query: "entanglement"
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
| `fixture` | Yes | Name of fixture directory to use |
| `task.prompt` | Yes | The prompt given to the LLM tool |
| `evaluation.gates` | Yes | List of pass/fail gates |
| `evaluation.judge` | No | LLM-as-judge configuration |

### Fixtures Location

Test fixtures and scenarios are in `llm-test-fixtures/` at your workspace root:

- **templates/qipu/**: Test environment templates (AGENTS.md, README.md)
- **rubrics/**: Evaluation rubrics for LLM-as-judge scoring
- **\*.yaml**: Scenario definitions

### Gate Types

| Type | Parameters | Description |
|------|------------|-------------|
| `min_notes` | `count: N` | At least N notes exist |
| `min_links` | `count: N` | At least N links exist |
| `search_hit` | `query: "..."` | Search returns results |

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

Results are stored in `llm-tool-test-results/` (configurable via `results_path` in config):

```
llm-tool-test-results/
├── cache/                    # Test result cache files
├── results.jsonl            # Append-only results database
└── <timestamp>-<tool>-<model>-<scenario>/
    ├── artifacts/
    │   ├── events.jsonl      # Event log
    │   ├── run.json          # Run metadata
    │   ├── store_snapshot/   # Store export after test
    │   └── transcript.raw.txt # Full command transcript
    ├── fixture/              # Template files used for test
    ├── evaluation.md         # Human-readable evaluation
    ├── metrics.json          # Detailed metrics
    └── report.md             # Test run report
```

**File locations:**
- **Raw data** (`transcript.raw.txt`, `events.jsonl`, `run.json`) stays in `artifacts/`
- **Analysis** (`evaluation.md`, `report.md`, `metrics.json`) is at the scenario root for easy access
- **Cache** and **database** are at the `llm-tool-test-results/` root

### Caching

Results are cached by a composite key:
- Scenario YAML hash
- Prompt hash
- Tool name
- Qipu git commit

Use `--no-cache` to force re-execution.

## Writing New Scenarios

1. Create a scenario YAML in `llm-test-fixtures/`:

```yaml
name: my_scenario
description: "Test description"
template_folder: qipu
task:
  prompt: |
    Create two notes about related topics and link them together.
    Use the 'related' link type.
evaluation:
  gates:
    - type: min_notes
      count: 2
    - type: min_links
      count: 1
```

2. Optionally create a rubric in `llm-test-fixtures/rubrics/`:

```yaml
criteria:
  - id: accuracy
    weight: 1.0
    description: "Task completed correctly"
output:
  format: json
  require_fields: [scores, weighted_score, confidence, issues, highlights]
```

3. Test with dry-run first:

```bash
llm-tool-test run \
  --scenario llm-test-fixtures/my_scenario.yaml \
  --dry-run
```

## Interpreting Results

### Gate Results

Gates are pass/fail checks. All gates must pass for the scenario to pass:

```
Gate MinNotes passed: Expected >= 1, found 2
Gate SearchHit passed: Query 'entanglement' found: true
```

### Judge Scores

Judge evaluation returns a weighted score (0.0 to 1.0):

```
Judge score: 0.85 (confidence: 0.92)
Issues: None identified
Highlights: Good use of tags, clear note structure
```

### Regression Detection

When comparing to a baseline:

```
--- Regression Report ---
Current: run-20260119-143022
Baseline: run-20260118-091544
Score change: +5.2%
Cost change: -12.3%
```

Warnings trigger when:
- Cost increases >50%
- Judge score decreases >15%
- Previously passing gates now fail

## Troubleshooting

### "Tool unavailable" error

Ensure the tool is in your PATH:

```bash
which amp
amp --version
```

### "Fixture not found" error

Check the fixtures directory exists at the configured path (default: `llm-test-fixtures/`):

```bash
ls llm-test-fixtures/
```

### Judge evaluation fails

1. Check API key is set
2. Verify rubric weights sum to 1.0
3. Check judge model availability

### Long-running scenarios

The harness currently has no timeout. For long scenarios, monitor manually or use system timeout:

```bash
timeout 300 cargo run -p llm-tool-test -- run --scenario ...
```

## Known Limitations

- Amp adapter CLI syntax is speculative and may need adjustment
- No multi-turn interaction support (single prompt only)
- Cost tracking not implemented
- No parallel scenario execution
- Limited gate types (tracked in beads; use `bd search "gate types"`)
