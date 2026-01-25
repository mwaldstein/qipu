# Quality Gates for Knowledge

**Source**: https://x.com/arscontexta/status/2015437189115486354  
**Author**: @arscontexta (Heinrich)

## Core Observation

Vibe coding worked once "ralph" introduced:
- Testing after each step
- Linting
- Clean context per loop

That's what made it scale. Vibe note-taking has the same problem.

## The Parallel

Knowledge bases and codebases are both "folders of text files with relationships between them"—a structure that compounds when it's good, decays when it's not.

Ralph gave coding quality gates. Notes don't have that yet.

## Open Question

**What's the unit test for knowledge?**

## Implications for Qipu

This suggests a potential feature direction:

- Validation rules for notes (completeness, link health, orphan detection)
- "Lint" passes that flag issues (circular references, dead links, unclear titles)
- Quality metrics (link density, MOC coverage, note freshness)
- Consistency checks similar to what we already have for DB/filesystem sync

The question of "what makes a good note" is harder than "what makes good code"—but maybe that's where typed links, required fields, and graph analysis come in.
