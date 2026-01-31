# Qipu Skills Directory

This directory contains skill definitions for integrating Qipu with various AI coding agents and tools.

## Skill Definitions

| File | Tool | Format |
|------|------|--------|
| `mcp-server.json` | Claude Desktop, Cursor | Model Context Protocol (MCP) |
| `github-copilot.json` | GitHub Copilot Extensions | OpenAI Function Calling |
| `opencode.json` | OpenCode | JSON Skill Definition |

## Quick Links

- **[Installation Guide](INSTALL.md)** - How to install Qipu skills for each tool
- **[Qipu README](../README.md)** - Project overview and CLI documentation
- **[Agent Guide](../AGENTS.md)** - Guide for AI agents working with Qipu

## Overview

Qipu provides integrations with major AI coding tools through standardized skill definitions. These skills allow agents to:

- **Capture knowledge** - Create notes directly from agent context
- **Search** - Full-text search across your knowledge graph
- **Build connections** - Create typed semantic links between notes
- **Generate context** - Agent-optimized output for session priming
- **Maintain health** - Check graph integrity and fix issues

## Skill Formats

### Model Context Protocol (MCP)

The universal standard for AI agent tools. Used by:
- Claude Desktop (Anthropic)
- Cursor IDE
- Other Anthropic-compatible clients

**Schema**: JSON Schema based tool definitions with `name`, `description`, and `inputSchema`.

### GitHub Copilot Extensions

Uses OpenAI's function calling format. Tools are defined with:
- `type: "function"`
- `function.name` - Tool identifier
- `function.description` - Natural language description for semantic routing
- `function.parameters` - JSON Schema for arguments

### OpenCode

JSON-based skill definition format compatible with function calling standards.

## Available Tools

All skill formats expose the same toolset:

| Tool | Purpose |
|------|---------|
| `qipu_capture` | Quick capture from stdin or args |
| `qipu_search` | Full-text search with ranking |
| `qipu_show` | Display a single note |
| `qipu_link_add` | Create typed semantic link |
| `qipu_link_tree` | Visualize link connections |
| `qipu_list` | List notes with filters |
| `qipu_prime` | Context primer for sessions |
| `qipu_context` | Context bundle generation |
| `qipu_value_set` | Set note value score |
| `qipu_doctor` | Store health check |

## Installation

See [INSTALL.md](INSTALL.md) for detailed installation instructions for each tool.

## Adding Support for New Tools

To add a new skill format:

1. Research the tool's skill/function definition format
2. Create a new JSON file in this directory
3. Define the same 10 tools with appropriate schema
4. Update `INSTALL.md` with installation instructions
5. Update this README.md with the new entry

## Questions?

- **Installation issues**: See [INSTALL.md](INSTALL.md) troubleshooting section
- **Feature requests**: Open an issue on GitHub
- **Contributions**: PRs welcome!

---

**Note**: These skill definitions require Qipu CLI to be installed and accessible in the system PATH.
