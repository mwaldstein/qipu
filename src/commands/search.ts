/**
 * qipu search - Full-text search within qipu notes.
 *
 * Based on specs/cli-interface.md and specs/indexing-search.md.
 *
 * Why: Search enables discovery of relevant knowledge when you don't know
 * the exact note ID or when exploring connections. This is not a replacement
 * for repo-wide grep - it's specifically for the knowledge store.
 *
 * Ranking heuristics (per spec):
 * - Title matches rank above body matches
 * - Exact tag matches rank above plain text
 * - Recently updated notes get a small boost
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, Note, NoteType } from "../lib/models.js";
import { resolveStore, listNotes } from "../lib/storage.js";

/**
 * Search result with score for ranking.
 */
interface SearchResult {
  note: Note;
  score: number;
  titleMatch: boolean;
  tagMatch: boolean;
  bodyMatch: boolean;
}

/**
 * Calculate search score for a note against a query.
 */
function scoreNote(note: Note, query: string): SearchResult | null {
  const queryLower = query.toLowerCase();
  const title = note.frontmatter.title.toLowerCase();
  const body = note.body.toLowerCase();
  const tags = (note.frontmatter.tags || []).map((t) => t.toLowerCase());

  let score = 0;
  let titleMatch = false;
  let tagMatch = false;
  let bodyMatch = false;

  // Title matches (highest weight)
  if (title.includes(queryLower)) {
    titleMatch = true;
    score += 100;

    // Exact title match gets bonus
    if (title === queryLower) {
      score += 50;
    }
  }

  // Tag matches (high weight)
  if (tags.some((tag) => tag === queryLower)) {
    tagMatch = true;
    score += 80;
  } else if (tags.some((tag) => tag.includes(queryLower))) {
    tagMatch = true;
    score += 40;
  }

  // Body matches (base weight)
  if (body.includes(queryLower)) {
    bodyMatch = true;
    score += 20;

    // Multiple occurrences boost (up to 3x)
    const occurrences = (body.match(new RegExp(queryLower, "gi")) || []).length;
    score += Math.min(occurrences - 1, 10) * 2;
  }

  // No matches
  if (score === 0) {
    return null;
  }

  // Recency boost (up to 10 points for notes updated in last 7 days)
  const updated = note.frontmatter.updated;
  if (updated) {
    const updateDate = new Date(updated);
    const now = new Date();
    const daysSinceUpdate =
      (now.getTime() - updateDate.getTime()) / (1000 * 60 * 60 * 24);

    if (daysSinceUpdate < 7) {
      score += Math.round(10 * (1 - daysSinceUpdate / 7));
    }
  }

  return { note, score, titleMatch, tagMatch, bodyMatch };
}

/**
 * Format a search result for human-readable output.
 */
function formatResult(result: SearchResult): string {
  const note = result.note;
  const id = note.frontmatter.id;
  const title = note.frontmatter.title;
  const type = note.frontmatter.type || "fleeting";
  const tags = note.frontmatter.tags || [];

  let line = `${id}\t${title}`;

  // Show match indicators
  const indicators: string[] = [];
  if (result.titleMatch) indicators.push("title");
  if (result.tagMatch) indicators.push("tag");
  if (result.bodyMatch) indicators.push("body");

  if (indicators.length > 0) {
    line += ` (${indicators.join(", ")})`;
  }

  if (tags.length > 0) {
    line += ` [${tags.join(", ")}]`;
  }

  return line;
}

export const searchCommand = new Command("search")
  .description("Search within qipu notes (titles + bodies)")
  .argument("<query>", "search query")
  .option("--tag <tag>", "filter by tag")
  .option(
    "--type <type>",
    "filter by type (fleeting, literature, permanent, moc)",
  )
  .option("--moc", "include only MOCs")
  .option("--no-moc", "exclude MOCs")
  .option("-n, --limit <n>", "limit number of results", "20")
  .action(
    (query: string, options: Record<string, unknown>, command: Command) => {
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
        let notes = listNotes(store.storePath);

        // Apply pre-filters
        if (options.tag) {
          notes = notes.filter((n) =>
            n.frontmatter.tags?.includes(options.tag as string),
          );
        }

        if (options.type) {
          notes = notes.filter(
            (n) => n.frontmatter.type === (options.type as NoteType),
          );
        }

        // MOC filter (--moc includes only MOCs, --no-moc excludes MOCs)
        if (options.moc === true) {
          notes = notes.filter((n) => n.frontmatter.type === "moc");
        } else if (options.moc === false) {
          notes = notes.filter((n) => n.frontmatter.type !== "moc");
        }

        // Score all notes
        const results: SearchResult[] = [];
        for (const note of notes) {
          const result = scoreNote(note, query);
          if (result) {
            results.push(result);
          }
        }

        // Sort by score (descending)
        results.sort((a, b) => b.score - a.score);

        // Apply limit
        const limit = parseInt(options.limit as string, 10) || 20;
        const limitedResults = results.slice(0, limit);

        if (globalOpts.json) {
          const items = limitedResults.map((result) => ({
            id: result.note.frontmatter.id,
            title: result.note.frontmatter.title,
            type: result.note.frontmatter.type || "fleeting",
            tags: result.note.frontmatter.tags || [],
            path: result.note.path
              ? path.relative(process.cwd(), result.note.path)
              : undefined,
            score: result.score,
            matches: {
              title: result.titleMatch,
              tag: result.tagMatch,
              body: result.bodyMatch,
            },
          }));

          console.log(
            JSON.stringify({
              query,
              total: results.length,
              results: items,
            }),
          );
        } else {
          if (limitedResults.length === 0) {
            console.log(`No results for "${query}"`);
          } else {
            for (const result of limitedResults) {
              console.log(formatResult(result));
            }

            if (!globalOpts.quiet && results.length > limit) {
              console.error(`\nShowing ${limit} of ${results.length} results`);
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
    },
  );
