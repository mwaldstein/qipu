/**
 * qipu export - Export notes for use outside the qipu store.
 *
 * Based on specs/export.md.
 *
 * Why: Export makes qipu notes usable outside the store for:
 * - Writing white papers and design docs
 * - Sharing research sets with LLM tools
 * - Archiving a project's knowledge base
 *
 * Three export modes:
 * 1. Bundle: Concatenate selected notes into a single markdown file
 * 2. Outline: Use a MOC as the structural outline, following its ordering
 * 3. Bibliography: Extract sources from notes into a bibliography
 *
 * Selection methods (composable):
 * - --note <id>: explicit note selection
 * - --tag <tag>: all notes with tag
 * - --moc <id>: notes linked from a MOC
 * - --query <text>: search-based selection
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, Note, Source } from "../lib/models.js";
import { resolveStore, findNote, listNotes } from "../lib/storage.js";
import { getIndex, getOutgoingLinks } from "../lib/indexing.js";

/** Link handling modes for export */
type LinkMode = "preserve" | "markdown" | "anchors";

/**
 * Rewrite wiki-style links based on the specified mode.
 */
function rewriteLinks(
  content: string,
  mode: LinkMode,
  noteIdToAnchor: Map<string, string>,
): string {
  if (mode === "preserve") {
    return content;
  }

  // Match wiki-style links: [[id]] or [[id|label]]
  const wikiLinkRegex = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

  return content.replace(wikiLinkRegex, (match, id, label) => {
    const displayLabel = label || id;

    if (mode === "markdown") {
      // Rewrite to markdown link format
      return `[${displayLabel}](${id}.md)`;
    } else if (mode === "anchors") {
      // Rewrite to section anchors
      const anchor = noteIdToAnchor.get(id);
      if (anchor) {
        return `[${displayLabel}](#${anchor})`;
      }
      // Keep as-is if note not in export
      return match;
    }

    return match;
  });
}

/**
 * Generate a URL-safe anchor from a note title.
 */
function titleToAnchor(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/**
 * Format a single note for bundle export.
 */
function formatNoteForBundle(
  note: Note,
  linkMode: LinkMode,
  noteIdToAnchor: Map<string, string>,
): string {
  const fm = note.frontmatter;
  const lines: string[] = [];

  // Create anchor target
  const anchor = titleToAnchor(fm.title);
  lines.push(`<a id="${anchor}"></a>`);
  lines.push("");

  // Note header
  lines.push(`## ${fm.title}`);
  lines.push("");

  // Metadata
  lines.push(`**ID:** ${fm.id}`);
  lines.push(`**Type:** ${fm.type || "fleeting"}`);

  if (fm.tags && fm.tags.length > 0) {
    lines.push(`**Tags:** ${fm.tags.join(", ")}`);
  }

  if (fm.created) {
    lines.push(`**Created:** ${fm.created}`);
  }

  if (fm.sources && fm.sources.length > 0) {
    lines.push(`**Sources:**`);
    for (const source of fm.sources) {
      if (source.url) {
        const title = source.title || source.url;
        lines.push(`- [${title}](${source.url})`);
      }
    }
  }

  lines.push("");
  lines.push("---");
  lines.push("");

  // Body with link rewriting
  const body = rewriteLinks(note.body, linkMode, noteIdToAnchor);
  lines.push(body);

  lines.push("");

  return lines.join("\n");
}

/**
 * Format bundle export (concatenated notes).
 */
function formatBundleExport(
  notes: Note[],
  storePath: string,
  linkMode: LinkMode,
): string {
  const lines: string[] = [];

  // Build anchor map for link rewriting
  const noteIdToAnchor = new Map<string, string>();
  for (const note of notes) {
    noteIdToAnchor.set(
      note.frontmatter.id,
      titleToAnchor(note.frontmatter.title),
    );
  }

  lines.push("# Qipu Export Bundle");
  lines.push("");
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push(`Store: ${storePath}`);
  lines.push(`Notes: ${notes.length}`);
  lines.push("");

  // Table of contents
  lines.push("## Table of Contents");
  lines.push("");
  for (const note of notes) {
    const anchor = titleToAnchor(note.frontmatter.title);
    lines.push(`- [${note.frontmatter.title}](#${anchor})`);
  }
  lines.push("");
  lines.push("---");
  lines.push("");

  // Notes
  for (const note of notes) {
    lines.push(formatNoteForBundle(note, linkMode, noteIdToAnchor));
  }

  return lines.join("\n");
}

/**
 * Format outline export (MOC-driven).
 */
function formatOutlineExport(
  mocNote: Note,
  linkedNotes: Note[],
  storePath: string,
  linkMode: LinkMode,
): string {
  const lines: string[] = [];

  // Build anchor map
  const noteIdToAnchor = new Map<string, string>();
  noteIdToAnchor.set(
    mocNote.frontmatter.id,
    titleToAnchor(mocNote.frontmatter.title),
  );
  for (const note of linkedNotes) {
    noteIdToAnchor.set(
      note.frontmatter.id,
      titleToAnchor(note.frontmatter.title),
    );
  }

  lines.push(`# ${mocNote.frontmatter.title}`);
  lines.push("");
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push(`Store: ${storePath}`);
  lines.push(`Outline based on MOC: ${mocNote.frontmatter.id}`);
  lines.push("");

  // Include MOC body as introduction/structure
  if (mocNote.body.trim()) {
    lines.push("## Overview");
    lines.push("");
    const mocBody = rewriteLinks(mocNote.body, linkMode, noteIdToAnchor);
    lines.push(mocBody);
    lines.push("");
    lines.push("---");
    lines.push("");
  }

  // Include linked notes in the order they appear in the MOC
  lines.push("## Contents");
  lines.push("");

  for (const note of linkedNotes) {
    lines.push(formatNoteForBundle(note, linkMode, noteIdToAnchor));
  }

  return lines.join("\n");
}

/**
 * Format a single source for bibliography.
 */
function formatSourceEntry(
  source: Source,
  noteTitle: string,
  noteId: string,
): string {
  const parts: string[] = [];

  if (source.title) {
    parts.push(`**${source.title}**`);
  }

  if (source.url) {
    parts.push(`<${source.url}>`);
  }

  if (source.accessed) {
    parts.push(`(Accessed: ${source.accessed})`);
  }

  parts.push(`[From: ${noteTitle} (${noteId})]`);

  return parts.join(" ");
}

/**
 * Format bibliography export.
 */
function formatBibliographyExport(notes: Note[], storePath: string): string {
  const lines: string[] = [];

  lines.push("# Bibliography");
  lines.push("");
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push(`Store: ${storePath}`);
  lines.push("");

  // Collect all sources with their origin notes
  const sources: Array<{ source: Source; noteTitle: string; noteId: string }> =
    [];

  for (const note of notes) {
    if (note.frontmatter.sources) {
      for (const source of note.frontmatter.sources) {
        sources.push({
          source,
          noteTitle: note.frontmatter.title,
          noteId: note.frontmatter.id,
        });
      }
    }
  }

  if (sources.length === 0) {
    lines.push("*No sources found in selected notes.*");
    return lines.join("\n");
  }

  // Sort sources by title, then URL
  sources.sort((a, b) => {
    const titleA = a.source.title || a.source.url || "";
    const titleB = b.source.title || b.source.url || "";
    return titleA.localeCompare(titleB);
  });

  // Format as list
  lines.push("## Sources");
  lines.push("");

  for (const entry of sources) {
    lines.push(
      `- ${formatSourceEntry(entry.source, entry.noteTitle, entry.noteId)}`,
    );
  }

  lines.push("");

  // Also provide raw count
  lines.push("---");
  lines.push("");
  lines.push(`Total sources: ${sources.length}`);
  lines.push(`From ${notes.length} notes`);

  return lines.join("\n");
}

/**
 * Format JSON export output.
 */
function formatJsonExport(
  notes: Note[],
  storePath: string,
  mode: "bundle" | "outline" | "bibliography",
  mocId?: string,
): object {
  const base = {
    generated_at: new Date().toISOString(),
    store: storePath,
    mode,
    note_count: notes.length,
  };

  if (mode === "bibliography") {
    const sources: Array<{
      title?: string;
      url?: string;
      accessed?: string;
      from_note_id: string;
      from_note_title: string;
    }> = [];

    for (const note of notes) {
      if (note.frontmatter.sources) {
        for (const source of note.frontmatter.sources) {
          sources.push({
            ...source,
            from_note_id: note.frontmatter.id,
            from_note_title: note.frontmatter.title,
          });
        }
      }
    }

    return {
      ...base,
      source_count: sources.length,
      sources,
    };
  }

  return {
    ...base,
    moc_id: mocId,
    notes: notes.map((note) => ({
      id: note.frontmatter.id,
      title: note.frontmatter.title,
      type: note.frontmatter.type || "fleeting",
      tags: note.frontmatter.tags || [],
      created: note.frontmatter.created,
      sources: note.frontmatter.sources || [],
      content: note.body,
    })),
  };
}

export const exportCommand = new Command("export")
  .description("Export notes for use outside the store")
  .option(
    "--note <id>",
    "include note by ID (repeatable)",
    (val, prev: string[]) => prev.concat([val]),
    [],
  )
  .option("--tag <tag>", "include notes with tag")
  .option("--moc <id>", "use MOC as outline (enables outline mode)")
  .option("--query <text>", "include notes matching search query")
  .option(
    "--mode <mode>",
    "export mode: bundle, outline, bibliography",
    "bundle",
  )
  .option(
    "--links <mode>",
    "link handling: preserve, markdown, anchors",
    "preserve",
  )
  .option("--output <file>", "output file path (default: stdout)")
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
      let mode = options.mode as "bundle" | "outline" | "bibliography";
      const linkMode = options.links as LinkMode;
      let mocNote: Note | null = null;

      // If --moc is specified, default to outline mode
      if (options.moc && mode === "bundle") {
        mode = "outline";
      }

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
        mocNote = findNote(store.storePath, options.moc as string);
        if (mocNote) {
          // For outline mode, don't add MOC itself to selectedIds
          // For other modes, include MOC and its linked notes
          if (mode !== "outline") {
            selectedIds.add(mocNote.frontmatter.id);
          }

          // Get notes linked from MOC
          const links = getOutgoingLinks(index, mocNote.frontmatter.id);
          for (const link of links) {
            selectedIds.add(link.to);
          }
        } else {
          if (globalOpts.json) {
            console.log(
              JSON.stringify({
                status: "error",
                error: `MOC not found: ${options.moc}`,
              }),
            );
          } else {
            console.error(`Error: MOC not found: ${options.moc}`);
          }
          process.exit(ExitCodes.DATA_ERROR);
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

      // Validate selection
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

      const relativePath =
        path.relative(process.cwd(), store.storePath) || ".qipu";

      // Generate output based on mode
      let output: string;

      if (globalOpts.json) {
        output = JSON.stringify(
          formatJsonExport(notes, relativePath, mode, mocNote?.frontmatter.id),
          null,
          2,
        );
      } else {
        switch (mode) {
          case "outline":
            if (!mocNote) {
              console.error(
                "Error: Outline mode requires --moc <id> to specify the outline structure",
              );
              process.exit(ExitCodes.USAGE_ERROR);
            }
            output = formatOutlineExport(
              mocNote,
              notes,
              relativePath,
              linkMode,
            );
            break;

          case "bibliography":
            output = formatBibliographyExport(notes, relativePath);
            break;

          case "bundle":
          default:
            output = formatBundleExport(notes, relativePath, linkMode);
            break;
        }
      }

      // Output to file or stdout
      if (options.output) {
        const fs = require("node:fs");
        fs.writeFileSync(options.output as string, output);
        if (!globalOpts.quiet) {
          console.log(`Exported to: ${options.output}`);
        }
      } else {
        console.log(output);
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
