/**
 * qipu prime - Emit a bounded session primer for agent startup.
 *
 * Based on specs/llm-context.md.
 *
 * Why: Agents need quick context at session start without consuming too much
 * of their context window. The primer provides:
 * - What qipu is (knowledge memory, not tasks)
 * - Quick command reference
 * - Store location
 * - Key MOCs and recently updated notes
 *
 * Target size: ~1-2k tokens for efficient session initialization.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes } from "../lib/models.js";
import { resolveStore, listNotes } from "../lib/storage.js";
import { getIndex, NoteMetadata } from "../lib/indexing.js";
import { formatTokenPrimer } from "../lib/token-output.js";

/** Core commands for quick reference */
const COMMAND_REFERENCE = [
  "init",
  "create",
  "capture",
  "list",
  "show",
  "search",
  "inbox",
  "index",
  "link add/remove/list/tree/path",
  "context",
  "prime",
];

/**
 * Format primer for human-readable output.
 */
function formatHumanPrimer(
  storePath: string,
  mocs: NoteMetadata[],
  recent: NoteMetadata[],
  noteCount: number,
): string {
  const lines: string[] = [];

  lines.push("# Qipu Knowledge Store");
  lines.push("");
  lines.push(
    "Qipu is a Zettelkasten-inspired knowledge management system for LLM workflows.",
  );
  lines.push(
    "Use it to store research, capture insights, and build knowledge graphs.",
  );
  lines.push("");
  lines.push(`Store: ${storePath}`);
  lines.push(`Notes: ${noteCount}`);
  lines.push("");
  lines.push("## Quick Commands");
  lines.push("");
  lines.push("- `qipu create <title>` - Create a new note");
  lines.push("- `qipu capture` - Create note from stdin");
  lines.push("- `qipu list [--tag <tag>] [--type <type>]` - List notes");
  lines.push("- `qipu show <id>` - Display a note");
  lines.push("- `qipu search <query>` - Search notes");
  lines.push("- `qipu inbox` - Show unprocessed notes");
  lines.push("- `qipu link tree <id>` - Show knowledge graph");
  lines.push("- `qipu context --note <id>` - Build context bundle");

  if (mocs.length > 0) {
    lines.push("");
    lines.push("## Maps of Content (MOCs)");
    lines.push("");
    for (const moc of mocs) {
      const tags = moc.tags.length > 0 ? ` [${moc.tags.join(", ")}]` : "";
      lines.push(`- ${moc.id}: ${moc.title}${tags}`);
    }
  }

  if (recent.length > 0) {
    lines.push("");
    lines.push("## Recently Updated");
    lines.push("");
    for (const note of recent) {
      const type = note.type !== "fleeting" ? ` (${note.type})` : "";
      lines.push(`- ${note.id}: ${note.title}${type}`);
    }
  }

  return lines.join("\n");
}

/**
 * Format primer for JSON output.
 */
function formatJsonPrimer(
  storePath: string,
  mocs: NoteMetadata[],
  recent: NoteMetadata[],
  noteCount: number,
): object {
  return {
    store: storePath,
    note_count: noteCount,
    commands: COMMAND_REFERENCE,
    mocs: mocs.map((m) => ({
      id: m.id,
      title: m.title,
      tags: m.tags,
    })),
    recent: recent.map((n) => ({
      id: n.id,
      title: n.title,
      type: n.type,
      updated: n.updated,
    })),
  };
}

export const primeCommand = new Command("prime")
  .description("Emit a bounded session primer for agent startup")
  .option("--max-mocs <n>", "maximum MOCs to include", "5")
  .option("--max-recent <n>", "maximum recent notes to include", "10")
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
      const index = getIndex(store.storePath);
      const allMetadata = Object.values(index.metadata);

      // Get MOCs
      const mocs = allMetadata
        .filter((m) => m.type === "moc")
        .sort((a, b) => (b.updated || "").localeCompare(a.updated || ""))
        .slice(0, parseInt(options.maxMocs as string, 10) || 5);

      // Get recent non-MOC notes
      const maxRecent = parseInt(options.maxRecent as string, 10) || 10;
      const recent = allMetadata
        .filter((m) => m.type !== "moc")
        .sort((a, b) => (b.updated || "").localeCompare(a.updated || ""))
        .slice(0, maxRecent);

      const relativePath =
        path.relative(process.cwd(), store.storePath) || ".qipu";
      const noteCount = allMetadata.length;

      if (globalOpts.token) {
        console.log(
          formatTokenPrimer(relativePath, mocs, recent, COMMAND_REFERENCE),
        );
      } else if (globalOpts.json) {
        console.log(
          JSON.stringify(
            formatJsonPrimer(relativePath, mocs, recent, noteCount),
            null,
            2,
          ),
        );
      } else {
        console.log(formatHumanPrimer(relativePath, mocs, recent, noteCount));
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
