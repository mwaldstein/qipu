# CLI Tool (Executable + Command Runtime)

## Job to be done

Provide a single `qipu` executable that can be run locally (offline) to create, read, and navigate a qipu knowledge store.

This spec focuses on the CLI as a product surface (parsing, dispatch, output profiles, determinism, and error conventions). Individual command semantics and data formats are specified elsewhere.

## In scope

- A single `qipu` executable with stable command/flag parsing.
- Global flags, output profiles (`--format human|json|records`), and verbosity controls.
- Store discovery and path resolution rules.
- Error handling and exit codes.
- Determinism requirements shared across commands.
- Testing expectations for the CLI runtime (goldens + integration).

## Out of scope

- Command-specific behavior (see `specs/cli-interface.md`).
- On-disk store formats (see `specs/storage-format.md`).
- Graph/query algorithms (see their respective specs).

## Requirements

### Executable

- The tool is invoked as `qipu`.
- The primary distribution artifact is a single, self-contained native executable (a "real binary"), not a script.
- `qipu --help` prints stable help output and exits `0`.
- `qipu --version` prints a single line with version information and exits `0`.
- The CLI must not require network access for normal operation.

### Distribution + dependencies

- Installation must be possible by downloading a release artifact and placing it on `$PATH` (no `node`, `python`, `java`, or other runtime required).
- Optional installer wrappers (Homebrew, system package managers, etc.) may exist, but they must install/launch the same native binary.
- The executable should be portable across common developer platforms (macOS, Linux, Windows) with minimal external dynamic dependencies.

### Performance budgets

The CLI is intended to feel "instant" in interactive use.

Targets (approximate, p95 on a typical developer laptop):

- `qipu --help` and `qipu --version`: < 100ms wall time.
- `qipu list` with ~1k notes: < 200ms.
- `qipu search` over ~10k notes: < 1s (with indexes built where applicable).

Implementation implications:

- The reference implementation language is Rust.
- Use Cargo as the build system.
- The build must produce a single `qipu` native executable suitable for release artifacts.
- Avoid designs that require a heavyweight runtime, JIT warmup, or spawning background daemons for baseline commands.

### Global flags

Global flags are defined in `specs/cli-interface.md` and must apply consistently across commands:

- `--root <path>`: base directory for resolving the store (default: current working directory).
- `--store <path>`: explicit store root path.
- `--format <human|json|records>`: output format (default: `human`).
  - `json`: machine-readable output.
  - `records`: line-oriented record output.
- `--quiet` / `--verbose`.

Constraints:

- `--format` may be specified at most once.
- Unknown `--format` values must produce a usage error (exit code `2`).
- Unknown flags/args must produce a usage error (exit code `2`).

### Store discovery and resolution

Qipu commands operate against a "store root" directory.

A "project root" is the first directory encountered while walking upward that contains either a `.git` directory or `Cargo.toml` file. If neither marker is found, the filesystem root serves as the project root.

Resolution order:

1. If `--store` is provided:
   - If it is relative, resolve it relative to `--root` (or cwd if `--root` is omitted).
   - Use the resulting path as the store root.
2. Otherwise, discover a store by walking up from `--root` (or cwd):
   - At each directory, if `.qipu/` exists, treat it as the store root and stop.
   - If `.qipu/` is absent but `qipu/` exists, treat `qipu/` as the store root and stop.
   - If the project root or filesystem root (whichever comes first) is reached with no store found, the store is "missing".

Missing-store behavior:

- Commands that require an existing store must fail with exit code `3`.
- Commands that create a store (notably `qipu init`) may create it at the default location.

### Output determinism

For the same inputs and store state:

- Output ordering must be stable.
- Output formatting must be stable.
- When truncation/budgeting occurs, it must be explicit and deterministic.

Determinism applies to all output formats (`--format human|json|records`).

### Exit codes

Exit codes must follow `specs/cli-interface.md`:

- `0`: success
- `1`: generic failure
- `2`: usage error
- `3`: data/store error

### Error messages

- Human output may include friendly context.
- With `--format json`, commands must include structured error details (shape defined per command) while still using the correct exit code.

### Filesystem hygiene

When writing files:

- Avoid rewriting files unnecessarily (do not touch notes unless the operation requires it).
- Preserve newline style where practical.
- Avoid writing derived caches unless a command explicitly calls for it (e.g. `qipu index`).

### Observability

- Provide a `--verbose` mode that can report timing for major phases (parse args, discover store, load indexes, execute command).
- Timing output must be deterministic in shape (keys/labels stable), even if values vary.

## Validation

This spec is considered implemented when:

- `qipu` runs and dispatches commands per `specs/cli-interface.md`.
- Store discovery behaves as described above.
- Exit codes and determinism rules are consistently applied.

## Testing expectations

- Integration tests: run CLI commands against a temporary directory/store.
- Golden tests: lock down deterministic outputs for commands like `qipu prime`, `qipu context`, and traversal outputs.
