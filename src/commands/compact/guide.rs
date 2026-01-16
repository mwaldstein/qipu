use crate::cli::Cli;
use crate::lib::error::Result;

/// Execute `qipu compact guide`
pub fn execute(_cli: &Cli) -> Result<()> {
    println!(
        r#"# Qipu Compaction Guide

Compaction allows you to create digest notes that summarize and replace sets of notes
in day-to-day retrieval, while keeping the original notes intact.

## Workflow

1. **Find candidates**: Use `qipu compact suggest` to find groups of notes that might
   benefit from compaction (dense, self-contained clusters).

2. **Review summaries**: Use `qipu context --format records` to review candidate notes
   in summary form before authoring a digest.

3. **Author a digest**: Create a new note that summarizes the candidate notes.
   
   Guidelines for digests:
   - Include a one-paragraph Summary
   - List key claims or insights
   - Add a section explaining when to expand into source notes
   - Keep it concise (shorter than the expanded sources)
   - Include source note IDs for traceability

   Example prompt for LLM:
   "Create a digest note that replaces these notes in day-to-day retrieval.
   Include a one-paragraph Summary, key claims, and a small section explaining
   when to expand into sources. Keep it short; include IDs for traceability."

4. **Register compaction**: Use `qipu compact apply <digest-id> --note <id>...`
   to register the compaction relationship.

5. **Validate**: Use `qipu compact report <digest-id>` to check compaction quality.
   Also run `qipu doctor` to validate invariants.

6. **Sanity check traversal**: Run a resolved traversal/search (e.g.,
   `qipu link tree <digest-id> --max-hops 2` or `qipu search <keyword>`) to
   confirm the digest surfaces as expected without `--no-resolve-compaction`.

## Commands

- `qipu compact apply <digest> --note <id>...` - Register compaction
- `qipu compact show <digest>` - Show what a digest compacts
- `qipu compact status <id>` - Show compaction relationships for a note
- `qipu compact report <digest>` - Quality metrics (coming soon)
- `qipu compact suggest` - Suggest compaction candidates (coming soon)
- `qipu compact guide` - Print this guide

## Invariants

Compaction must satisfy these invariants:

- At most one compactor per note
- No cycles in compaction chains
- No self-compaction
- All referenced IDs must exist

Use `qipu doctor` to validate compaction invariants.
"#
    );

    Ok(())
}
