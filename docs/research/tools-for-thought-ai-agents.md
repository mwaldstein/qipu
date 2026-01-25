# Tools for Thought for AI Agents

**Source**: https://x.com/arscontexta/status/2015201046469943660  
**Author**: @arscontexta (Heinrich)

## Core Problem

Vibe note-taking has the same problem as vibe coding before testing/linting—a few ideas work fine but hundreds become unmanageable slop. What's the "unit test" for knowledge work? What catches drift before it compounds?

## Proposal

Build tools for thought *native to how agents work*, using what agents already have:

- Markdown files + wiki links for structure
- YAML frontmatter + embeddings for discovery
- Hooks and subagents for automation and separation
- Bash, grep, git, MCP servers for tooling
- Self-written code to extend the system

## The Meta Layer

The system researches tools for thought to build itself a tool for thought. Feed Claude methodologies on how humans build knowledge systems; Claude figures out what applies to agents and adjusts its own instructions.

Every rule starts as a hypothesis. Observations get logged to learning files that persist across sessions—always something to reflect on.

## Foundation: Graph from Markdown

Build a graph database out of markdown files:

- **Files** are nodes
- **Wiki links** are edges
- **YAML frontmatter** is queryable metadata

It's a knowledge graph an LLM can move through naturally.

## Context Curation

Filenames as claims: when you write "since [[quality is the hard part]] the question becomes..." the title IS the argument. Before opening anything, there's already a sense of what each note argues.

Every note has a YAML header with a one-sentence description. Before loading any file, grab the description and decide if it's worth the context. Most decisions can be made at the description level without loading full files.

## Historical Context

Humans have built tools for thought for centuries:

- Llull's rotating wheels for combinatorial truth generation
- Bruno's memory palaces with millions of image combinations
- Zettelkasten's network of connected ideas
- Evergreen notes forcing complete thoughts
- MOCs organizing clusters of related thinking

All had one thing in common: a human was the operator. What's different now is *something else* is using this architecture—and it can build its own.

## Self-Engineering Loop

1. Dump deep research articles about tools for thought into inbox
2. Claude reads and extracts claims ("this method argues X" → note titled "X")
3. Apply claims to how agents work
4. Log observations to persistent files
5. System reflects on learnings, considers what to change

## Adaptations

Claude found the Cornell Notes 5 Rs framework while researching and adapted it for agents, adding a 6th phase for self-improvement.

## Relevance to Qipu

This validates several Qipu design decisions:

- Graph structure from markdown + links
- Typed relationships for semantic edges
- MOCs as organizational layer
- The need for "quality gates" in knowledge work (see related note)
