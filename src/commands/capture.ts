/**
 * qipu capture - Create a new note from stdin.
 *
 * Based on specs/cli-interface.md.
 *
 * Examples:
 *   pbpaste | qipu capture --type fleeting --tag docs
 *   qipu capture --title "Thoughts on indexing" < notes.txt
 *
 * Why: The capture command enables frictionless knowledge capture from any
 * text source (clipboard, pipes, files). This supports the Zettelkasten
 * workflow where quick capture is essential - ideas should flow into the
 * system with minimal ceremony.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, NoteType } from "../lib/models.js";
import { resolveStore, createNote } from "../lib/storage.js";

/**
 * Read all data from stdin.
 * Returns empty string if stdin is a TTY (no pipe).
 */
async function readStdin(): Promise<string> {
  // Check if stdin is a TTY (no pipe)
  if (process.stdin.isTTY) {
    return "";
  }

  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];

    process.stdin.on("data", (chunk: Buffer) => {
      chunks.push(chunk);
    });

    process.stdin.on("end", () => {
      resolve(Buffer.concat(chunks).toString("utf-8"));
    });

    process.stdin.on("error", reject);
  });
}

/**
 * Derive a title from content if none provided.
 * Uses first non-empty line, truncated to reasonable length.
 */
function deriveTitle(content: string): string {
  const firstLine = content
    .split("\n")
    .map((line) => line.trim())
    .find((line) => line.length > 0);

  if (!firstLine) {
    return `Capture ${new Date().toISOString().slice(0, 16)}`;
  }

  // Remove markdown headers
  const cleaned = firstLine.replace(/^#+\s*/, "");

  // Truncate to reasonable length
  if (cleaned.length <= 60) {
    return cleaned;
  }

  return cleaned.slice(0, 57) + "...";
}

export const captureCommand = new Command("capture")
  .description("Create a new note from stdin")
  .option("--title <title>", "note title (derived from content if omitted)")
  .option(
    "-t, --type <type>",
    "note type (fleeting, literature, permanent, moc)",
    "fleeting",
  )
  .option(
    "--tag <tag>",
    "add tag (repeatable)",
    (value, previous: string[]) => {
      return previous.concat([value]);
    },
    [],
  )
  .action(async (options: Record<string, unknown>, command: Command) => {
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
      // Read content from stdin
      const content = await readStdin();

      if (!content.trim()) {
        if (globalOpts.json) {
          console.log(
            JSON.stringify({
              status: "error",
              error: "No content provided on stdin",
            }),
          );
        } else {
          console.error("Error: No content provided on stdin");
        }
        process.exit(ExitCodes.USAGE_ERROR);
      }

      // Determine title
      const title = (options.title as string) || deriveTitle(content);

      // Parse tags
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
        type: options.type as NoteType,
        tags: tags.length > 0 ? tags : undefined,
        body: content.trim(),
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
