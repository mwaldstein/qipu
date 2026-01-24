# LLM User Validation Testing

## Purpose

Validate that qipu achieves its core goal: being usable by an LLM as the primary user.

This spec defines a **separate testing harness** (`llm-tool-test`) that invokes real LLM CLI tools, captures complete transcripts, and evaluates both structural outcomes and qualitative interaction quality.

## Core Goal

From README.md: "qipu is designed to be used by LLMs as their long-term memory system."

This test confirms that goal by having actual LLMs attempt to use qipu given only its documentation.

---

## Architecture Overview

The test harness is a **separate binary** (`llm-tool-test`) that tests qipu as a black box. This keeps test infrastructure out of the distributed qipu binary and allows the harness to be reused for testing other LLM-facing CLI tools.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Scenarios     │────▶│  llm-tool-test  │────▶│  Tool Adapters  │
│   (YAML)        │     │  (separate bin) │     │  amp, opencode  │
└─────────────────┘     └────────┬────────┘     └────────┬────────┘
                                 │                       │
                                 ▼                       ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │   Evaluator     │◀────│  Transcript     │
                        │  (gates+judge)  │     │  Capture        │
                        └────────┬────────┘     └─────────────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Results DB     │
                        │  (JSONL/SQLite) │
                        └─────────────────┘
```

### Key Architectural Decisions

1. **Separate binary**: `llm-tool-test` is NOT part of qipu. It lives in a separate crate within the workspace (or separate repo).

2. **Black-box testing**: The harness treats qipu as an external CLI tool. It doesn't link against qipu's library code.

3. **Test fixtures**: Each scenario includes its own AGENTS.md, README, and any seed data the LLM needs to understand the tool.

4. **Reusability**: The harness design is tool-agnostic. While built for qipu, it could test other CLI tools with appropriate scenario definitions.

---

## Test Environment Setup

Each test run creates an isolated environment that mimics what a real LLM user would see.

### Project Fixture Structure

```
tests/fixtures/qipu_basic/
├── AGENTS.md              # Instructions for LLM (qipu commands, patterns)
├── README.md              # Project context
├── .qipu/                 # Pre-initialized store (optional)
│   └── ...
└── seed_notes/            # Notes to import before test (optional)
    └── ...
```

### AGENTS.md for Tests

The test AGENTS.md should contain:
- Available qipu commands with examples
- Common workflows
- Output format guidance
- Error handling patterns

This is the primary "documentation" the LLM receives, so it must be representative of real usage.

```markdown
# AGENTS.md (test fixture)

## Qipu - Knowledge Store

### Quick Start
qipu init                    # Initialize store
qipu create "Title"          # Create a note
qipu list                    # List all notes
qipu show <id>               # Display a note

### Creating Linked Notes
qipu create "Concept A" --type permanent
qipu create "Concept B" --type permanent  
qipu link add <id-a> <id-b> --type related

### Searching
qipu search "query"          # Full-text search
qipu context --query "topic" # Get context for a topic

### Output Formats
All commands support: --format human|json|records
```

---

## Scenarios

Scenarios define test cases in declarative YAML format.

### Location

```
tests/llm_scenarios/
├── capture_article.yaml
├── link_navigation.yaml
├── context_retrieval.yaml
└── ...
```

### Schema

```yaml
id: capture_article_basic              # Unique identifier
description: "Capture article ideas as linked notes"
tags: [capture, links, retrieval]

# Documentation provided to LLM
docs:
  prime: true                          # Include `qipu prime` output
  help_commands:                       # Include --help for these commands
    - create
    - link
    - list
    - search
    - show

# Store setup
setup:
  store: fresh                         # fresh | seeded
  seed_notes: []                       # Optional fixture notes
  seed_from_pack: null                 # Optional: load from pack file

# The task
task:
  prompt: |
    Capture the key ideas from this article about distributed systems:
    [article content or URL]
    
    Create structured notes with meaningful links between concepts.
    When done, verify you can retrieve the main concepts via search.

# Tool/model matrix (run scenario against multiple tools)
tool_matrix:
  - tool: amp
    model: default
  - tool: opencode
    model: default

# Execution constraints
run:
  timeout_secs: 600
  max_turns: 40                        # If tool supports limiting

# Evaluation criteria
evaluation:
  # Stage 1: Structural gates (cheap, deterministic)
  gates:
    min_notes: 3
    min_links: 1
    retrieval_queries:                 # Must return results
      - "distributed"
      - "systems"
  
  # Stage 2: LLM-as-judge (expensive, qualitative)
  judge:
    enabled: true
    rubric: rubrics/capture_v1.yaml
    pass_threshold: 0.70

# Cost controls
cost:
  max_usd: 0.75                        # Per-run budget
  cache: true                          # Cache identical runs
```

### Rubrics

Rubrics define qualitative evaluation criteria for the LLM judge.

```yaml
# tests/llm_scenarios/rubrics/capture_v1.yaml
criteria:
  command_correctness:
    weight: 0.25
    description: "Uses valid qipu commands with correct syntax"
  
  structure_quality:
    weight: 0.30
    description: "Notes are well-organized with meaningful links"
  
  coverage:
    weight: 0.30
    description: "Captures key concepts without major omissions"
  
  retrieval_success:
    weight: 0.15
    description: "Can retrieve captured knowledge via search/show"

output:
  format: json
  require_fields:
    - scores          # Per-criterion scores (0.0-1.0)
    - weighted_score  # Overall weighted score
    - confidence      # Judge confidence (0.0-1.0)
    - issues          # List of problems found
    - highlights      # List of good practices observed
```

---

## Tool Adapters

Tool adapters handle the specifics of invoking each LLM CLI tool.

### Trait Definition

```rust
pub trait ToolAdapter: Send + Sync {
    /// Tool identifier
    fn name(&self) -> &str;
    
    /// Check if tool is installed and authenticated
    fn is_available(&self) -> Result<ToolStatus, AdapterError>;
    
    /// Execute a task and capture transcript
    fn execute_task(
        &self,
        context: &TaskContext,
        work_dir: &Path,
        transcript_dir: &Path,
    ) -> Result<ExecutionResult, AdapterError>;
    
    /// Estimate cost for a prompt (if possible)
    fn estimate_cost(&self, prompt_tokens: usize) -> Option<CostEstimate>;
}

pub struct TaskContext {
    pub system_prompt: String,    // qipu prime + help output
    pub task_prompt: String,      // The actual task
    pub timeout: Duration,
}

pub struct ExecutionResult {
    pub exit_code: i32,
    pub duration: Duration,
    pub token_usage: Option<TokenUsage>,
    pub cost_estimate: Option<f64>,
}

pub struct ToolStatus {
    pub available: bool,
    pub version: Option<String>,
    pub authenticated: bool,
    pub budget_remaining: Option<f64>,
}
```

### Supported Tools

| Tool | Invocation | Transcript Capture |
|------|------------|-------------------|
| amp | `amp --prompt-file <file>` | PTY capture |
| opencode | `opencode <prompt>` | PTY capture |
| claude | `claude --prompt <text>` | PTY capture |

### Token Usage and Cost Collection

Token counts and costs should be obtained from actual LLM tool responses when available:

- Tools (amp, opencode, claude) may report actual API token usage and/or cost in their output
- Adapters parse token counts and/or cost from tool output when available
- If token counts are not available in tool output, return `None` for `token_usage`
- If actual cost is available in tool output, use it instead of estimating from tokens
- Do not use `len() / 4` or other character-based approximations for token counts
- Do not estimate cost from character counts

### PTY Session Capture

Use pseudo-terminal capture to get complete interaction including:
- ANSI colors and formatting
- Interactive prompts
- Real-time streaming output
- Tool invocations and results

Fallback to piped stdout/stderr if PTY unavailable.

---

## Transcript Artifacts

Each run produces a bundle of artifacts for review and analysis.

### Directory Structure

```
tests/transcripts/<scenario_id>/<tool>/<timestamp>/
├── transcript.raw.txt      # Complete PTY output
├── events.jsonl            # Structured event log
├── run.json                # Run metadata
├── store_snapshot/         # Copy of .qipu/ after run
│   └── export.json         # qipu dump --format json
└── report.md               # Human-readable summary
```

### Event Log Format

```jsonl
{"ts": 1705500000.123, "event": "spawn", "command": "amp", "args": ["--prompt-file", "/tmp/prompt.txt"]}
{"ts": 1705500001.456, "event": "output", "text": "I'll create some notes...\n"}
{"ts": 1705500005.789, "event": "tool_call", "tool": "bash", "command": "qipu create \"Main Concept\""}
{"ts": 1705500006.012, "event": "tool_result", "output": "qp-abc123\n", "exit_code": 0}
{"ts": 1705500030.000, "event": "complete", "exit_code": 0, "duration_secs": 30.0}
```

### Run Metadata

```json
{
  "scenario_id": "capture_article_basic",
  "scenario_hash": "abc123def456",
  "tool": "amp",
  "model": "claude-3-5-sonnet",
  "qipu_version": "0.1.0",
  "qipu_commit": "abc123",
  "timestamp": "2025-01-17T12:00:00Z",
  "duration_secs": 45.3,
  "cost_estimate_usd": 0.023,
  "token_usage": {
    "input": 1500,
    "output": 800
  }
}
```

---

## Evaluation

### Two-Stage Evaluation

**Stage 1: Structural Gates (cheap, deterministic)**

Run locally without LLM calls:
- Note count meets minimum
- Link count meets minimum
- Retrieval queries return results
- Store passes `qipu doctor`
- No command errors in transcript

Gates produce binary pass/fail plus metrics.

**Stage 2: LLM-as-Judge (optional, qualitative)**

If enabled and gates pass:
- Prepare judge prompt with transcript summary + store export + rubric
- Call a low-cost judge model (e.g., gpt-4o-mini, claude-haiku)
- Parse structured JSON response
- Compute weighted score from rubric criteria

### Evaluation Result

```rust
pub struct EvaluationResult {
    // Stage 1
    pub gates_passed: bool,
    pub metrics: EvaluationMetrics,
    
    // Stage 2 (optional)
    pub judge_score: Option<f64>,
    pub judge_confidence: Option<f64>,
    pub judge_feedback: Option<JudgeFeedback>,
    
    // Final determination
    pub outcome: Outcome,  // Pass | Fail | ReviewRequired
}

pub struct EvaluationMetrics {
    pub note_count: usize,
    pub link_count: usize,
    pub retrieval_hits: usize,
    pub retrieval_misses: usize,
    pub command_errors: usize,
    pub doctor_passed: bool,
}

pub enum Outcome {
    Pass,
    Fail { reason: String },
    ReviewRequired { reason: String },
}
```

---

## Results Tracking

### Results Database

Store results in append-only format for trend analysis:

```
tests/llm_results/
├── results.jsonl           # Append-only run results
└── results.db              # Optional SQLite for queries
```

### Result Record

```json
{
  "id": "run-2025-01-17-001",
  "scenario_id": "capture_article_basic",
  "scenario_hash": "abc123",
  "tool": "amp",
  "model": "claude-3-5-sonnet",
  "qipu_commit": "def456",
  "timestamp": "2025-01-17T12:00:00Z",
  "duration_secs": 45.3,
  "cost_usd": 0.023,
  "gates_passed": true,
  "metrics": { ... },
  "judge_score": 0.82,
  "outcome": "Pass",
  "transcript_path": "tests/transcripts/capture_article_basic/amp/1705500000"
}
```

### Regression Detection

Compare against baseline runs:
- Score degradation > 15% triggers warning
- Gate failures that previously passed trigger alert
- Cost increases > 50% trigger warning

---

## CLI Interface

The harness is invoked via `llm-tool-test`, a separate binary.

### Commands

```bash
# Run scenarios
llm-tool-test run                           # Run all scenarios
llm-tool-test run --scenario capture_basic  # Run specific scenario
llm-tool-test run --tags capture,links      # Run by tags
llm-tool-test run --tool amp                # Run with specific tool
llm-tool-test run --max-usd 1.00            # Budget limit

# Dry run (no LLM calls)
llm-tool-test run --dry-run                 # Show what would run + cost estimate

# Results
llm-tool-test list                          # List recent runs
llm-tool-test show <run-id>                 # Show run details
llm-tool-test compare <run-id> <run-id>     # Compare two runs
llm-tool-test report                        # Generate summary report

# Maintenance
llm-tool-test clean --older-than 30d        # Clean old transcripts
```

### Environment Variables

```bash
LLM_TOOL_TEST_ENABLED=1       # Must be set to run tests (safety)
LLM_TOOL_TEST_BUDGET_USD=5.00 # Session budget limit
LLM_TOOL_TEST_TOOL=amp        # Default LLM tool
LLM_TOOL_TEST_JUDGE=gpt-4o-mini  # Judge model
```

---

## Cost Management

### Budget Enforcement

1. **Per-run limit**: From scenario `cost.max_usd`
2. **Session limit**: From `--max-usd` or `QIPU_LLM_BUDGET_USD`
3. **Estimate before run**: Warn if estimated cost exceeds limit
4. **Track actual cost**: Log to results for trend analysis

### Caching

Cache key components:
- Scenario YAML hash
- Prompt file hash
- qipu prime output hash
- Tool + model identifier
- qipu version/commit

If cache hit, reuse transcript and evaluation results.

### Dry Run Mode

`--dry-run` shows:
- Scenarios that would run
- Estimated prompt sizes
- Estimated costs
- Cache status (hit/miss)

---

## Security

### Transcript Redaction

Before writing `report.md`, redact:
- API keys and tokens
- Passwords and secrets
- Email addresses (optional)
- File paths with usernames

Raw transcript preserved but marked sensitive.

### Gitignore

```gitignore
# LLM test artifacts (volatile, potentially sensitive)
tests/transcripts/
tests/llm_results/
```

---

## Not In Scope

- CI integration (too expensive, run manually)
- Multi-model statistical benchmarking (future)
- Real-time cost tracking via provider APIs
- Interactive test authoring UI
