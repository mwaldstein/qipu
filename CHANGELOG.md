# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Removed

- **BREAKING**: Removed interactive picker (`--interactive` / `-i` flag) from `list`, `search`, and `inbox` commands. This feature was undocumented, untested, and violated the spec-first principle. Users relying on fzf-style selection should use external tools like `fzf` piped with the JSON output format instead:
  ```bash
  # Example: interactive selection using fzf
  qipu list --format json | jq -r '.[] | "\(.id) \(.title)"' | fzf
  ```

- Removed dependencies: `inquire` and `atty` (no longer needed without interactive picker)

## [0.3.27] - 2026-02-09

### Added

- Initial release with core Zettelkasten-inspired knowledge management features
- Note creation, linking, search, and graph traversal capabilities
- Support for multiple note types: fleeting, literature, permanent, MOC
- Compaction and indexing system
- Records format for LLM tool integration
