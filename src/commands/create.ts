/**
 * qipu create / qipu new - Create a new note.
 *
 * Based on specs/cli-interface.md.
 */

import { Command } from "commander";
import * as path from "node:path";
import { spawnSync } from "node:child_process";
import { ExitCodes, NoteType } from "../lib/models.js";
import { resolveStore, createNote } from "../lib/storage.js";

function makeCreateAction(_commandName: string) {
  return (
    title: string,
    options: Record<string, unknown>,
    command: Command,
  ) => {
    const globalOpts = command.parent?.opts() || {};

    // Resolve store
    const store = resolveStore({
      store: globalOpts.store as string | undefined,
      root: globalOpts.root as string | undefined,
    });

    if (!store) {
      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "error",
            error: 'No store found. Run "qipu init" first.',
          }),
        );
      } else {
        console.error('Error: No store found. Run "qipu init" first.');
      }
      process.exit(ExitCodes.DATA_ERROR);
    }

    try {
      // Parse tags (can be specified multiple times)
      const tags: string[] = [];
      if (options.tag) {
        const tagOpts = Array.isArray(options.tag)
          ? options.tag
          : [options.tag];
        for (const t of tagOpts) {
          tags.push(...(t as string).split(",").map((s: string) => s.trim()));
        }
      }

      const note = createNote(store.storePath, {
        title,
        type: options.type as NoteType | undefined,
        tags: tags.length > 0 ? tags : undefined,
      });

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "created",
            id: note.frontmatter.id,
            title: note.frontmatter.title,
            type: note.frontmatter.type,
            path: note.path,
            tags: note.frontmatter.tags || [],
          }),
        );
      } else {
        console.log(note.frontmatter.id);
        if (!globalOpts.quiet && note.path) {
          console.log(path.relative(process.cwd(), note.path));
        }
      }

      // Handle --open flag
      if (options.open) {
        const editor = process.env.EDITOR || "vi";
        spawnSync(editor, [note.path!], { stdio: "inherit" });
      }

      process.exit(ExitCodes.SUCCESS);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "error",
            error: message,
          }),
        );
      } else {
        console.error(`Error: ${message}`);
      }

      process.exit(ExitCodes.FAILURE);
    }
  };
}

export const createCommand = new Command("create")
  .description("Create a new note")
  .argument("<title>", "note title")
  .option(
    "-t, --type <type>",
    "note type (fleeting, literature, permanent, moc)",
  )
  .option(
    "--tag <tag>",
    "add tag (repeatable)",
    (value, previous: string[]) => {
      return previous.concat([value]);
    },
    [],
  )
  .option("--open", "open in $EDITOR after creation")
  .option("--template <name>", "use template from templates/")
  .action(makeCreateAction("create"));

// Alias: qipu new
export const newCommand = new Command("new")
  .description("Create a new note (alias for create)")
  .argument("<title>", "note title")
  .option(
    "-t, --type <type>",
    "note type (fleeting, literature, permanent, moc)",
  )
  .option(
    "--tag <tag>",
    "add tag (repeatable)",
    (value, previous: string[]) => {
      return previous.concat([value]);
    },
    [],
  )
  .option("--open", "open in $EDITOR after creation")
  .option("--template <name>", "use template from templates/")
  .action(makeCreateAction("new"));
