/**
 * qipu list - List notes in the store.
 *
 * Based on specs/cli-interface.md.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, NoteType } from "../lib/models.js";
import { resolveStore, listNotes } from "../lib/storage.js";

export const listCommand = new Command("list")
  .description("List notes")
  .option("-t, --type <type>", "filter by note type")
  .option("--tag <tag>", "filter by tag")
  .option("--since <date>", "filter by creation date (ISO 8601)")
  .action((options, command) => {
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
      let notes = listNotes(store.storePath, {
        type: options.type as NoteType | undefined,
        tag: options.tag,
      });

      // Filter by --since if provided
      if (options.since) {
        const sinceDate = new Date(options.since);
        notes = notes.filter((n) => {
          const created = n.frontmatter.created;
          return created && new Date(created) >= sinceDate;
        });
      }

      if (globalOpts.json) {
        const output = notes.map((n) => ({
          id: n.frontmatter.id,
          title: n.frontmatter.title,
          type: n.frontmatter.type || "fleeting",
          tags: n.frontmatter.tags || [],
          path: n.path ? path.relative(store.rootPath, n.path) : null,
          created: n.frontmatter.created || null,
          updated: n.frontmatter.updated || null,
        }));
        console.log(JSON.stringify(output, null, 2));
      } else {
        if (notes.length === 0) {
          if (!globalOpts.quiet) {
            console.log("No notes found.");
          }
        } else {
          for (const note of notes) {
            const tags = note.frontmatter.tags?.length
              ? ` [${note.frontmatter.tags.join(", ")}]`
              : "";
            console.log(
              `${note.frontmatter.id}\t${note.frontmatter.title}${tags}`,
            );
          }
        }
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
  });
