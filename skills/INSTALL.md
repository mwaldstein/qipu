# Qipu Skills - Installation Guide

This guide explains how to install and configure Qipu skills for different AI coding agents and tools.

## Overview

Qipu provides skill definitions for:
- **Model Context Protocol (MCP)** - Universal standard for Claude Desktop, Cursor, and other Anthropic-compatible clients
- **GitHub Copilot Extensions** - For GitHub Copilot users
- **OpenCode** - Open source AI coding agent

## Quick Start

1. Install Qipu CLI:
   ```bash
   cargo install --path .
   ```

2. Initialize your knowledge store:
   ```bash
   qipu init
   ```

3. Choose your agent below and follow the installation instructions.

---

## Model Context Protocol (MCP)

MCP is the universal standard for connecting AI agents to tools. It's supported by:
- **Claude Desktop** (Anthropic)
- **Cursor IDE**
- Other Anthropic-compatible clients

### Installation (Claude Desktop)

1. Open **Claude Desktop** → **Settings** → **Developer**
2. Click **Edit Config** to open `claude_desktop_config.json`
3. Add the Qipu MCP server configuration:

```json
{
  "mcpServers": {
    "qipu": {
      "command": "qipu",
      "args": ["mcp"],
      "env": {
        "QIPU_STORE_PATH": "/path/to/your/.qipu"
      }
    }
  }
}
```

4. Restart Claude Desktop
5. A hammer icon (🔨) will appear when Qipu tools are available

### Installation (Cursor IDE)

1. Open **Cursor** → **Settings** → **MCP**
2. Add a new MCP server:
   - **Name**: `qipu`
   - **Type**: `command`
   - **Command**: `qipu`
   - **Args**: `["mcp"]`
   - **Environment**: Set `QIPU_STORE_PATH` to your `.qipu` directory

3. Restart Cursor

### MCP Server Implementation

The MCP server definition is in `skills/mcp-server.json`. The server exposes tools with the following schema:

```json
{
  "name": "qipu_capture",
  "description": "Quick capture from stdin or command arguments",
  "inputSchema": {
    "type": "object",
    "properties": {
      "content": {"type": "string"},
      "title": {"type": "string"},
      "type": {"type": "string", "enum": ["fleeting", "literature", "permanent", "moc"]},
      "tags": {"type": "array", "items": {"type": "string"}}
    }
  }
}
```

**Available MCP Tools:**
- `qipu_capture` - Quick capture
- `qipu_search` - Full-text search
- `qipu_show` - Display note
- `qipu_link_add` - Create typed link
- `qipu_link_tree` - Visualize connections
- `qipu_list` - List notes with filters
- `qipu_prime` - Context primer for sessions
- `qipu_context` - Context bundle generation
- `qipu_value_set` - Set note value
- `qipu_doctor` - Store health check

---

## GitHub Copilot Extensions

GitHub Copilot Extensions use a JSON-based function calling format compatible with OpenAI's function calling standard.

### Creating a Copilot Extension

1. Create a **GitHub App** with Copilot extension capabilities
2. Register the extension in your app's configuration
3. Use the skill definition from `skills/github-copilot.json`

### Extension Configuration

The extension exposes tools that Copilot can call based on natural language prompts. Tools include descriptions that help Copilot's semantic routing:

```json
{
  "type": "function",
  "function": {
    "name": "qipu_search",
    "description": "Full-text search across all notes with ranking and value scoring",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {"type": "string", "description": "Search query string"},
        "min_value": {"type": "integer", "minimum": 0, "maximum": 100}
      },
      "required": ["query"]
    }
  }
}
```

### Usage in Copilot

When working in VS Code with Copilot:
- Type prompts like "Search my knowledge graph for Rust error handling"
- Copilot will automatically route to the appropriate Qipu tool
- Results are returned and can be integrated into your code

---

## OpenCode

OpenCode uses a JSON-based skill format similar to function calling. The skill definition is in `skills/opencode.json`.

### Installation

1. Open OpenCode configuration
2. Add the Qipu skill to your agent's tool registry
3. Configure environment variable `QIPU_STORE_PATH` pointing to your `.qipu` directory

### Skill Configuration

```json
{
  "name": "qipu",
  "description": "Zettelkasten-inspired knowledge management CLI",
  "tools": [...]
}
```

### Usage

OpenCode agents can call Qipu tools based on context and user prompts. Use natural language like:
- "Capture this note about Rust ownership"
- "Search my notes for async patterns"
- "Link this research to existing notes"

---

## Environment Variables

All Qipu skills respect these environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `QIPU_STORE_PATH` | Path to `.qipu` store directory | `./.qipu` |
| `QIPU_LOG_LEVEL` | Logging level | `info` |
| `QIPU_FORMAT` | Output format for agents | `json` |

---

## Best Practices

### For AI Agents

1. **Session Initialization**: Run `qipu_prime` at the start of each session to load relevant context
2. **Quality Scoring**: Use `qipu_value_set` to mark high-value notes (81-100) for prioritization
3. **Graph Building**: Use typed links to create semantic connections between notes
4. **Regular Maintenance**: Run `qipu_doctor` periodically to maintain graph integrity

### Value Score Guidelines

| Score Range | Meaning | Usage |
|-------------|---------|-------|
| 0-20 | Deprioritized | Superseded drafts, duplicates, outdated notes |
| 21-80 | Standard | General research, work-in-progress, reference material |
| 81-100 | High-value | Canonical definitions, MOCs, gems, critical knowledge |

### Link Types

Use these semantic link types to build meaningful connections:
- `related` - General relationship
- `derived-from` - Evolution or refinement
- `supports` - Evidence or backing
- `contradicts` - Opposing view
- `part-of` - Hierarchical inclusion
- `answers` - Solution to a question
- `refines` - Improved version
- `same-as` - Equivalent content
- `alias-of` - Alternative naming
- `follows` - Temporal or logical sequence

---

## Troubleshooting

### MCP Server Not Starting

1. Verify `qipu` is in your PATH
2. Check that `QIPU_STORE_PATH` is set correctly
3. Enable verbose logging: Set `QIPU_LOG_LEVEL=debug`

### Copilot Extension Not Responding

1. Verify the GitHub App has Copilot extension permissions
2. Check the extension's endpoint is accessible
3. Review extension logs for tool call failures

### OpenCode Skills Not Available

1. Confirm the skill file is in the correct directory
2. Check that environment variables are set
3. Verify Qipu CLI is installed and accessible

---

## Resources

- [Qipu GitHub Repository](https://github.com/anomalyco/qipu)
- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [GitHub Copilot Extensions Documentation](https://docs.github.com/en/copilot/building-copilot-extensions)
- [OpenCode Documentation](https://opencode.ai)

---

## Contributing

To add support for additional AI tools:

1. Research the tool's skill/function format
2. Create a new skill file in `skills/` directory
3. Update this installation guide
4. Test with the target tool

---

**Note**: Skills definitions are in `skills/` directory:
- `skills/mcp-server.json` - Model Context Protocol
- `skills/github-copilot.json` - GitHub Copilot Extensions
- `skills/opencode.json` - OpenCode agent
