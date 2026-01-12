# Qipu Agent Operations Guide

## Build & Run

```bash
npm install          # Install dependencies
npm run build        # Compile TypeScript to dist/
npm test             # Run all tests
npm run test:watch   # Run tests in watch mode
```

## CLI Usage (after build)

```bash
node dist/cli.js --help
node dist/cli.js --version
```

## Project Structure

- `src/` - TypeScript source code
- `src/lib/` - Shared utilities (storage, models, parsing)
- `src/commands/` - CLI command implementations
- `dist/` - Compiled JavaScript output
- `specs/` - Application specifications (read-only reference)

## Key Commands

- `qipu init` - Create a new store
- `qipu create` - Create a new note
- `qipu list` - List notes
- `qipu show <id>` - Display a note

## Testing

Tests use Vitest. Run `npm test` to execute all tests.
