# LLM Tool Test

Framework for evaluating LLM coding agents against structured test scenarios.

## Safety

**Required**: Set `LLM_TOOL_TEST_ENABLED=1` before running tests.

```bash
export LLM_TOOL_TEST_ENABLED=1
```

This prevents accidental expensive LLM API calls.

## Basic Commands

### Run Scenarios

```bash
# Run single scenario
llm-tool-test run --scenario capture_basic

# Run all scenarios
llm-tool-test run --all

# Filter by tags or tier
llm-tool-test run --all --tags smoke
llm-tool-test run --all --tier 1

# Dry run (no LLM calls)
llm-tool-test run --scenario capture_basic --dry-run
```

### List Scenarios

```bash
# List all
llm-tool-test scenarios

# Filter
llm-tool-test scenarios --tags capture
llm-tool-test scenarios --tier 0
```

### Show Scenario Details

```bash
llm-tool-test show capture_basic
```

### Clean Artifacts

```bash
# Clean old results (older than 7 days)
llm-tool-test clean --older-than "7d"

# Clean all
llm-tool-test clean
```

## Matrix Runs

Test multiple tools/models in one run:

```bash
llm-tool-test run --all --tools opencode,claude-code --models gpt-4o,claude-sonnet
```

## Interpreting Results

Each run generates an `evaluation.md` with:

**Summary**: Scenario name, tool, model, outcome (Pass/Fail)

**Metrics**:
- Gates Passed: X/N - Test criteria satisfied
- Notes Created: Count
- Links Created: Count
- Duration: Time taken
- Cost: Estimated API cost
- Composite Score: Overall performance (0.0-1.0)

**Human Review**: Manual scoring section (you fill in)

**Links**: Transcript, metrics, events, store snapshot

### Gate Types

Tests pass when all gates succeed:
- `min_notes`: Minimum notes created
- `min_links`: Minimum links created
- `search_hit`: Query returns results
- `note_exists`: Specific note ID exists
- `link_exists`: Specific link exists
- `tag_exists`: Tag found in store
- `content_contains`: Note contains substring
- `command_succeeds`: Shell command exits successfully
- `doctor_passes`: Qipu health check passes
- `no_transcript_errors`: No parsing errors

## Typical Workflow

```bash
# 1. Enable safety flag
export LLM_TOOL_TEST_ENABLED=1

# 2. List available scenarios
llm-tool-test scenarios

# 3. Run specific scenario
llm-tool-test run --scenario capture_basic --tool opencode

# 4. Check results
cat llm-tool-test-results/<timestamp>*/evaluation.md

# 5. Review transcript for debugging
cat llm-tool-test-results/<timestamp>*/transcript.raw.txt
```

## Configuration

Optional `llm-tool-test-config.toml` for cost tracking:

```toml
[models.gpt-4o]
input_cost_per_1k_tokens = 2.5
output_cost_per_1k_tokens = 10.0
```

Copy `llm-tool-test-config.example.toml` as template.

## Troubleshooting

**"LLM testing is disabled"**: Set `LLM_TOOL_TEST_ENABLED=1`

**Scenario not found**: Check it's in `fixtures/` directory, use `llm-tool-test scenarios` to list

**Gate failures**: Check metrics.json and transcript.raw.txt for details

**Timeout errors**: Increase timeout with `--timeout-secs 600`

**Cache issues**: Disable caching with `--no-cache` or clean old results

**Composite score low**: Review which gates failed in evaluation.md

**Tool not supported**: Available tools: opencode, claude-code. (Note: amp is experimental/de-prioritized)

## Results Location

All test artifacts stored in `llm-tool-test-results/<timestamp>-<tool>-<model>-<scenario>/`
