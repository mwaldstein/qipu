/**
 * qipu inbox - List unprocessed notes (knowledge processing queue).
 *
 * Based on specs/cli-interface.md.
 *
 * Why: The inbox command surfaces notes that need attention - fleeting captures
 * and literature notes that haven't been processed into permanent notes. This
 * is the Zettelkasten "processing queue" that helps prevent knowledge from
 * getting lost in the pile of quick captures.
 *
 * Default behavior: Show notes with type in {fleeting, literature} that are
 * not yet linked into MOCs. These are candidates for refinement.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, Note } from "../lib/models.js";
import { resolveStore, listNotes } from "../lib/storage.js";
import { getIndex, getBacklinks } from "../lib/indexing.js";

/**
 * Check if a note is linked from any MOC.
 */
function isLinkedFromMoc(
  note: Note,
  index: ReturnType<typeof getIndex>,
): boolean {
  const backlinks = getBacklinks(index, note.frontmatter.id);

  for (const linkerId of backlinks) {
    const linkerMeta = index.metadata[linkerId];
    if (linkerMeta && linkerMeta.type === "moc") {
      return true;
    }
  }

  return false;
}

/**
 * Format a note for human-readable output.
 */
function formatNote(note: Note, cwd: string): string {
  const id = note.frontmatter.id;
  const title = note.frontmatter.title;
  const type = note.frontmatter.type || "fleeting";
  const tags = note.frontmatter.tags || [];

  let line = `${id}\t${title}`;

  if (type !== "fleeting") {
    line += ` [${type}]`;
  }

  if (tags.length > 0) {
    line += ` {${tags.join(", ")}}`;
  }

  return line;
}

export const inboxCommand = new Command("inbox")
  .description("List unprocessed notes (fleeting/literature not in MOCs)")
  .option("--no-moc", "include notes already linked from MOCs")
  .option("--type <type>", "filter by type (fleeting, literature)")
  .action((options: Record<string, unknown>, command: Command) => {
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
      // Get all notes
      const allNotes = listNotes(store.storePath);

      // Get index for backlink checking
      const index = getIndex(store.storePath);

      // Filter to inbox-worthy notes
      const inboxNotes = allNotes.filter((note) => {
        const type = note.frontmatter.type || "fleeting";

        // Only include fleeting and literature by default
        if (type !== "fleeting" && type !== "literature") {
          return false;
        }

        // Filter by specific type if requested
        if (options.type && type !== options.type) {
          return false;
        }

        // Exclude notes already linked from MOCs unless --no-moc is specified
        if (options.moc !== false && isLinkedFromMoc(note, index)) {
          return false;
        }

        return true;
      });

      // Sort by created date (oldest first - process the backlog)
      inboxNotes.sort((a, b) => {
        const dateA = a.frontmatter.created || "";
        const dateB = b.frontmatter.created || "";
        return dateA.localeCompare(dateB);
      });

      if (globalOpts.json) {
        const items = inboxNotes.map((note) => ({
          id: note.frontmatter.id,
          title: note.frontmatter.title,
          type: note.frontmatter.type || "fleeting",
          tags: note.frontmatter.tags || [],
          path: note.path ? path.relative(process.cwd(), note.path) : undefined,
          created: note.frontmatter.created,
          updated: note.frontmatter.updated,
        }));

        console.log(JSON.stringify(items, null, 2));
      } else {
        if (inboxNotes.length === 0) {
          if (!globalOpts.quiet) {
            console.log("Inbox empty - no unprocessed notes!");
          }
        } else {
          for (const note of inboxNotes) {
            console.log(formatNote(note, process.cwd()));
          }

          if (!globalOpts.quiet) {
            console.error(`\n${inboxNotes.length} note(s) to process`);
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
