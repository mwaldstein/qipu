/**
 * qipu context - Build an LLM-friendly context bundle.
 *
 * Based on specs/llm-context.md.
 *
 * Why: LLMs need relevant context to answer questions effectively. The context
 * command assembles a bundle of notes based on selection criteria (note IDs,
 * tags, MOCs, or search queries) with budget controls to fit within context
 * windows.
 *
 * Selection methods (composable):
 * - --note <id>: explicit note selection
 * - --tag <tag>: all notes with tag
 * - --moc <id>: notes linked from a MOC
 * - --query <text>: search-based selection
 *
 * Output profiles:
 * - default: markdown bundle
 * - --json: structured JSON
 * - --token: token-optimized format
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, Note } from "../lib/models.js";
import { resolveStore, findNote, listNotes } from "../lib/storage.js";
import { getIndex, getOutgoingLinks, NoteMetadata } from "../lib/indexing.js";
import {
  formatTokenContext,
  getSummary,
  estimateTokens,
} from "../lib/token-output.js";

/**
 * Format context bundle in markdown.
 */
function formatMarkdownContext(
  notes: Note[],
  storePath: string,
  truncated: boolean,
): string {
  const lines: string[] = [];

  lines.push("# Qipu Context Bundle");
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push(`Store: ${storePath}`);
  lines.push(`Notes: ${notes.length}`);
  if (truncated) {
    lines.push("Status: TRUNCATED (budget exceeded)");
  }
  lines.push("");

  for (const note of notes) {
    const fm = note.frontmatter;
    lines.push(`## Note: ${fm.title} (${fm.id})`);

    if (note.path) {
      lines.push(`Path: ${note.path}`);
    }
    lines.push(`Type: ${fm.type || "fleeting"}`);

    if (fm.tags && fm.tags.length > 0) {
      lines.push(`Tags: ${fm.tags.join(", ")}`);
    }

    if (fm.sources && fm.sources.length > 0) {
      lines.push("Sources:");
      for (const source of fm.sources) {
        if (source.url) {
          lines.push(
            `- ${source.url}${source.title ? ` (${source.title})` : ""}`,
          );
        }
      }
    }

    lines.push("");
    lines.push("---");
    lines.push(note.body);
    lines.push("---");
    lines.push("");
  }

  return lines.join("\n");
}

/**
 * Format context bundle in JSON.
 */
function formatJsonContext(
  notes: Note[],
  storePath: string,
  truncated: boolean,
): object {
  return {
    generated_at: new Date().toISOString(),
    store: storePath,
    truncated,
    notes: notes.map((note) => ({
      id: note.frontmatter.id,
      title: note.frontmatter.title,
      type: note.frontmatter.type || "fleeting",
      tags: note.frontmatter.tags || [],
      path: note.path,
      sources: note.frontmatter.sources || [],
      summary: getSummary(note),
      content: note.body,
    })),
  };
}

/**
 * Apply budget constraints to notes.
 * Returns truncated list of notes that fit within budget.
 */
function applyBudget(
  notes: Note[],
  options: { maxChars?: number; maxTokens?: number },
): { notes: Note[]; truncated: boolean } {
  if (!options.maxChars && !options.maxTokens) {
    return { notes, truncated: false };
  }

  const result: Note[] = [];
  let charCount = 0;

  for (const note of notes) {
    // Estimate size of this note in the output
    const noteContent =
      `## Note: ${note.frontmatter.title} (${note.frontmatter.id})\n` +
      `Type: ${note.frontmatter.type || "fleeting"}\n` +
      note.body;

    const noteChars = noteContent.length;
    const noteTokens = estimateTokens(noteContent);

    // Check if adding this note would exceed budget
    if (options.maxChars && charCount + noteChars > options.maxChars) {
      return { notes: result, truncated: true };
    }

    if (options.maxTokens) {
      const currentTokens = estimateTokens(
        result.map((n) => n.body).join("\n"),
      );
      if (currentTokens + noteTokens > options.maxTokens) {
        return { notes: result, truncated: true };
      }
    }

    result.push(note);
    charCount += noteChars;
  }

  return { notes: result, truncated: false };
}

export const contextCommand = new Command("context")
  .description("Build an LLM-friendly context bundle")
  .option(
    "--note <id>",
    "include note by ID (repeatable)",
    (val, prev: string[]) => prev.concat([val]),
    [],
  )
  .option("--tag <tag>", "include notes with tag")
  .option("--moc <id>", "include notes linked from MOC")
  .option("--query <text>", "include notes matching search query")
  .option("--max-chars <n>", "maximum characters in output")
  .option("--max-tokens <n>", "maximum tokens (approximate)")
  .option("--with-body", "include full note bodies in token output", false)
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
      const selectedIds = new Set<string>();

      // Selection by explicit note IDs
      const noteIds = options.note as string[];
      for (const id of noteIds) {
        const note = findNote(store.storePath, id);
        if (note) {
          selectedIds.add(note.frontmatter.id);
        }
      }

      // Selection by tag
      if (options.tag) {
        const tagNotes = listNotes(store.storePath, {
          tag: options.tag as string,
        });
        for (const note of tagNotes) {
          selectedIds.add(note.frontmatter.id);
        }
      }

      // Selection by MOC
      if (options.moc) {
        const mocNote = findNote(store.storePath, options.moc as string);
        if (mocNote) {
          selectedIds.add(mocNote.frontmatter.id);

          // Get notes linked from MOC
          const links = getOutgoingLinks(index, mocNote.frontmatter.id);
          for (const link of links) {
            selectedIds.add(link.to);
          }
        }
      }

      // Selection by query
      if (options.query) {
        const query = (options.query as string).toLowerCase();
        const allNotes = listNotes(store.storePath);

        for (const note of allNotes) {
          const title = note.frontmatter.title.toLowerCase();
          const body = note.body.toLowerCase();
          const tags = (note.frontmatter.tags || []).map((t) =>
            t.toLowerCase(),
          );

          if (
            title.includes(query) ||
            body.includes(query) ||
            tags.some((t) => t.includes(query))
          ) {
            selectedIds.add(note.frontmatter.id);
          }
        }
      }

      // If no selection criteria, show usage
      if (
        selectedIds.size === 0 &&
        !(options.note as string[])?.length &&
        !options.tag &&
        !options.moc &&
        !options.query
      ) {
        console.error("Error: No selection criteria provided.");
        console.error(
          "Use --note <id>, --tag <tag>, --moc <id>, or --query <text>",
        );
        process.exit(ExitCodes.USAGE_ERROR);
      }

      // Load full notes
      const notes: Note[] = [];
      for (const id of selectedIds) {
        const note = findNote(store.storePath, id);
        if (note) {
          notes.push(note);
        }
      }

      // Sort deterministically by created date, then ID
      notes.sort((a, b) => {
        const dateA = a.frontmatter.created || "";
        const dateB = b.frontmatter.created || "";
        if (dateA !== dateB) return dateA.localeCompare(dateB);
        return a.frontmatter.id.localeCompare(b.frontmatter.id);
      });

      // Apply budget
      const maxChars = options.maxChars
        ? parseInt(options.maxChars as string, 10)
        : undefined;
      const maxTokens = options.maxTokens
        ? parseInt(options.maxTokens as string, 10)
        : undefined;

      const { notes: budgetedNotes, truncated } = applyBudget(notes, {
        maxChars,
        maxTokens,
      });

      const relativePath =
        path.relative(process.cwd(), store.storePath) || ".qipu";

      if (globalOpts.token) {
        console.log(
          formatTokenContext(budgetedNotes, {
            store: relativePath,
            withBody: options.withBody as boolean,
            maxChars,
            maxTokens,
            query: options.query as string | undefined,
          }),
        );
      } else if (globalOpts.json) {
        console.log(
          JSON.stringify(
            formatJsonContext(budgetedNotes, relativePath, truncated),
            null,
            2,
          ),
        );
      } else {
        console.log(
          formatMarkdownContext(budgetedNotes, relativePath, truncated),
        );
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
