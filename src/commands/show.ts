/**
 * qipu show - Display a note.
 *
 * Based on specs/cli-interface.md.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes } from "../lib/models.js";
import { resolveStore, findNote } from "../lib/storage.js";
import { extractInlineLinks } from "../lib/parsing.js";

export const showCommand = new Command("show")
  .description("Display a note")
  .argument("<id-or-path>", "note ID or file path")
  .option("--links", "show links for the note")
  .action((idOrPath: string, options, command) => {
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
      const note = findNote(store.storePath, idOrPath);

      if (!note) {
        if (globalOpts.json) {
          console.log(
            JSON.stringify({
              status: "error",
              error: `Note not found: ${idOrPath}`,
            }),
          );
        } else {
          console.error(`Error: Note not found: ${idOrPath}`);
        }
        process.exit(ExitCodes.DATA_ERROR);
      }

      if (globalOpts.json) {
        const output: Record<string, unknown> = {
          id: note.frontmatter.id,
          title: note.frontmatter.title,
          type: note.frontmatter.type || "fleeting",
          created: note.frontmatter.created || null,
          updated: note.frontmatter.updated || null,
          tags: note.frontmatter.tags || [],
          path: note.path ? path.relative(store.rootPath, note.path) : null,
          content: note.body,
        };

        if (note.frontmatter.sources) {
          output.sources = note.frontmatter.sources;
        }

        if (options.links) {
          // Include both typed links and inline links
          const inlineLinks = extractInlineLinks(note.body);
          output.links = {
            typed: note.frontmatter.links || [],
            inline: inlineLinks.map((l) => ({
              target: l.target,
              label: l.label,
            })),
          };
        }

        console.log(JSON.stringify(output, null, 2));
      } else {
        // Human-readable output
        console.log(`# ${note.frontmatter.title}`);
        console.log();
        console.log(`ID: ${note.frontmatter.id}`);
        console.log(`Type: ${note.frontmatter.type || "fleeting"}`);

        if (note.frontmatter.tags?.length) {
          console.log(`Tags: ${note.frontmatter.tags.join(", ")}`);
        }

        if (note.frontmatter.created) {
          console.log(`Created: ${note.frontmatter.created}`);
        }

        if (options.links) {
          console.log();
          console.log("## Links");

          const typedLinks = note.frontmatter.links || [];
          const inlineLinks = extractInlineLinks(note.body);

          if (typedLinks.length > 0) {
            console.log("\nTyped links:");
            for (const link of typedLinks) {
              console.log(`  - [${link.type}] ${link.id}`);
            }
          }

          if (inlineLinks.length > 0) {
            console.log("\nInline links:");
            for (const link of inlineLinks) {
              console.log(`  - ${link.target}`);
            }
          }

          if (typedLinks.length === 0 && inlineLinks.length === 0) {
            console.log("(no links)");
          }
        }

        if (note.body) {
          console.log();
          console.log("---");
          console.log();
          console.log(note.body);
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
