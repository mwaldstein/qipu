# Telemetry (DRAFT)

> **WARNING: DRAFT STATUS**
> This specification is currently in DRAFT status. DO NOT IMPLEMENT until finalized.
> The design below is for discussion purposes only.

## Overview

This specification outlines the requirements for a minimal, privacy-focused telemetry system to collect basic usage statistics. The primary goal is to understand tool usage patterns to improve the software while strictly respecting user privacy and providing clear, easy-to-use opt-out mechanisms.

## Privacy & Principles

- **Minimal Collection:** Only collect what is necessary to answer specific product questions.
- **Privacy First:** No PII (Personally Identifiable Information) or content (note text, filenames) shall be collected.
- **Opt-Out:** Users must be able to opt-out easily via CLI command or configuration.
- **Transparency:** The system should be able to display exactly what is being sent.

## Requirements

### Data Collection

The following minimal events are proposed for collection:

1.  **Command Execution:**
    - Command name (e.g., `capture`, `log`, `sync`)
    - Success/Failure status
    - Execution duration (bucketed)
    - *Excludes*: Arguments, flags, filenames, or user content.

2.  **Environment & State:**
    - OS Platform (Linux, macOS, Windows) - Generic, no version numbers if possible.
    - App Version
    - **Usage Stats (Bucketed):**
        - Number of Workspaces (e.g., 1, 2-5, 5+).
        - Total Note Count (e.g., <10, 10-100, 100-1000, 1000+).
    - *Rationale*: To understand scale of usage and performance needs without tracking content.

3.  **Errors:**
    - Error types/codes (e.g., `IOError`, `ConfigError`)
    - *Excludes*: Stack traces containing user paths or data.

### User Control (Opt-Out)

Telemtry should likely be opt-in or strictly opt-out. If opt-out:

- **CLI Command:** `qipu telemetry disable` (and `enable`)
- **Config:** A simple boolean in the global config file (e.g., `telemetry_enabled = false`).
- **Environment Variable:** `QIPU_NO_TELEMETRY=1` to disable.

### Implementation Constraints

- **Non-blocking:** Telemetry submission must not block the main execution path. It should happen in a background thread or process, or "fire and forget".
- **Offline Capable:** If the user is offline, events should either be dropped (preferred for simplicity) or queued (with strict limits).
- **Endpoint:** Data should be sent to a secure, first-party managed endpoint (or a privacy-respecting proxy).

## Success Criteria

- [ ] Privacy review completed and approved.
- [ ] Opt-out mechanisms (CLI, Config, Env Var) fully specified and verified.
- [ ] List of collected events finalized and minimized.
- [ ] "Dry run" mode specified (to show users what *would* be sent).
