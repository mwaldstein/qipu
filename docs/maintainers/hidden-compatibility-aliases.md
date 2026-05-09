# Hidden Compatibility Aliases

This is the maintainer-facing catalog for hidden compatibility behavior governed
by ADR 0006. These command shapes intentionally work but are not intended API.
Do not add them to command help, README examples, quickstarts, generated
`AGENTS.md` content, or public integration guidance unless a later decision
promotes them.

When adding an alias, record the intended API, hidden accepted shape, evidence
pattern, implementation location, and regression coverage here.

## Note Creation Title Alias

- Intended inline-body API: `qipu create "Title" --body "Body text"`
- Intended stdin API: `echo "Body text" | qipu capture --title "Title"`
- Hidden accepted shape: `qipu create --title "Title" --body "Body text"`
- Evidence pattern: agents attempted `create --title` by analogy with
  `capture --title`.
- Implementation: `src/cli/args.rs`, `src/commands/dispatch/notes.rs`
- Regression coverage: `tests/cli/create.rs::test_create_hidden_title_alias_warns`

The successful hidden path emits guidance toward the intended positional-title
API and must not document the hidden shape as a recommended form.

## Context Positional Note Alias

- Intended API: `qipu context --note <id>`
- Hidden accepted shape: `qipu context <id>`
- Evidence pattern: LLM usage transcripts repeatedly attempted the positional
  note form, then recovered through `qipu context --help`.
- Implementation: `src/cli/commands/data.rs`, `src/commands/context/mod.rs`
- Regression coverage:
  `tests/cli/context/hidden_positional.rs::test_context_hidden_positional_note_alias_selects_note`

Keep this alias narrow: one positional note id normalizes into the existing
`--note` selector. Do not broaden this into positional query behavior without a
new decision.

## Link Add Flat Alias

- Intended API: `qipu link add <from> <to> --type <type>`
- Hidden accepted shape: `qipu link <from> <to> --type <type>`
- Evidence pattern: LLM usage transcripts repeatedly attempted the flat link
  form, then recovered through `qipu link --help`.
- Implementation: `src/cli/link.rs`, `src/commands/dispatch/link.rs`
- Regression coverage:
  `tests/cli/link/add/basic.rs::test_link_hidden_add_shorthand_adds_typed_link`

Keep this alias narrow: require exactly two existing/custom-compatible note ids
or generated `qp-` ids plus an explicit type flag. Near-miss subcommand typos
such as `qipu link ad ...` must remain errors and point users to the intended
`qipu link add <from> <to> --type <type>` API.
