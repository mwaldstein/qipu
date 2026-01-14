# LLM User Validation Test

Status: Draft
Created: 2026-01-14

## Purpose

Validate that qipu achieves its core goal: being usable by an LLM as the primary user.

This test confirms that the documentation provided to an LLM (`qipu prime` + CLI help) enables the LLM to successfully use qipu for knowledge storage and retrieval tasks.

## Core Goal

From README.md: "qipu is designed to be used by LLMs as their long-term memory system."

This spec defines a test to verify that goal.

## Test Concept

Given a fresh qipu store and an LLM that has received qipu's documentation (via `qipu prime` and CLI commands), when the LLM is given a knowledge capture task, it should successfully use qipu commands to complete the task.

## Success Criteria

The test is successful when:
1. The LLM invokes valid qipu commands
2. The resulting store state is valid and meaningful:
   - Notes are created with reasonable content
   - Links between notes reflect the knowledge structure
   - The task's knowledge is captured and retrievable

## Test Input

A knowledge capture task requiring:
- Creating multiple notes
- Establishing relationships between them
- Retrieving information via the graph

Example: "Capture the key ideas from [article/url] as structured notes with links"

## Non-Determinism

LLM behavior is non-deterministic. This test does not verify exact command sequences. It verifies:
- The LLM can discover and use qipu commands
- The outcome is a valid, useful qipu store

## Test Execution

This is an end-to-end test requiring:
- A local LLM CLI tool
- Capture of LLM command invocations
- Validation of resulting store state

### Tool Support

The test framework should support multiple LLM CLI tools:

- **Primary**: OpenCode (current development environment)
- **Planned support**: Claude CLI, Codex CLI, and other agentic LLM tools

Each tool may require different command capture mechanisms. The test framework abstracts tool-specific details behind a common interface.

### Test Modes

- **Single-tool test**: Run against one configured tool
- **Multi-tool test**: Run against multiple tools to verify consistent behavior
- **Comparison mode**: (Future) Compare LLM behavior across tools/models

The test framework should be extensible for new tools without modifying core test logic.

### Transcript Recording

All LLM sessions should be recorded for analysis:

- **Transcript capture**: Full session log (LLM output + shell commands invoked)
- **Storage location**: `tests/transcripts/<tool>/<timestamp>/`
- **Human review**: Transcripts available for qualitative assessment
- **Version control**: Consider `.gitignore` for transcript directories (volatile artifacts)

### LLM-Based Evaluation

For additional quality assessment:

- **Meta-evaluation**: Use an LLM to review the transcript and score:
  - Command appropriateness
  - Graph structure quality
  - Knowledge capture completeness
- **Scoring criteria**: (Future) Define rubrics for evaluation
- **Confidence reporting**: LLM evaluator provides confidence scores alongside pass/fail

This meta-evaluation adds a qualitative layer beyond store state validation.

## Not in This Spec (deferred to implementation)

- How to capture LLM commands (shell logging, transcript recording)
- How to verify store state programmatically
- Specific task prompts or test fixtures
- CI integration or feature flags
