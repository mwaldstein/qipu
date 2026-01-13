/**
 * Indexing system for Qipu stores.
 *
 * Based on specs/indexing-search.md - builds derived views for fast navigation:
 * - Metadata index: id -> {title, type, tags, path, created, updated}
 * - Tag index: tag -> [ids...]
 * - Backlink index: id -> [ids that link to it]
 * - Graph: adjacency list of all links (inline + typed)
 *
 * Why: Indexes enable O(1) lookups for common operations instead of scanning
 * all files. This is critical for LLM workflows where response time matters.
 * Cache stored in .qipu/.cache/ for git-friendliness (derived, gitignored).
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { Note, TypedLink } from "./models.js";
import { listNotes, readNote } from "./storage.js";
import {
  extractInlineLinks,
  inlineToTypedLinks,
  InlineLink,
} from "./parsing.js";

/**
 * Metadata entry for a note in the index.
 */
export interface NoteMetadata {
  id: string;
  title: string;
  type: string;
  tags: string[];
  path: string;
  created: string;
  updated: string;
}

/**
 * Edge in the link graph.
 */
export interface GraphEdge {
  /** Source note ID */
  from: string;
  /** Target note ID */
  to: string;
  /** Link type */
  type: string;
  /** Link source (inline or typed) */
  source: "inline" | "typed";
}

/**
 * Complete index structure for a store.
 */
export interface StoreIndex {
  /** Version for forward compatibility */
  version: number;
  /** When the index was last built */
  built_at: string;
  /** Metadata index: id -> metadata */
  metadata: Record<string, NoteMetadata>;
  /** Tag index: tag -> [ids] */
  tags: Record<string, string[]>;
  /** Backlink index: id -> [ids that link to it] */
  backlinks: Record<string, string[]>;
  /** All edges in the graph */
  edges: GraphEdge[];
  /** File modification times for incremental updates */
  mtimes: Record<string, number>;
}

/** Current index format version */
const INDEX_VERSION = 1;

/** Cache directory within store */
const CACHE_DIR = ".cache";

/** Index file name */
const INDEX_FILE = "index.json";

/**
 * Get the path to the index file.
 */
export function getIndexPath(storePath: string): string {
  return path.join(storePath, CACHE_DIR, INDEX_FILE);
}

/**
 * Load existing index from disk.
 * Returns null if no index exists or it's invalid.
 */
export function loadIndex(storePath: string): StoreIndex | null {
  const indexPath = getIndexPath(storePath);

  if (!fs.existsSync(indexPath)) {
    return null;
  }

  try {
    const content = fs.readFileSync(indexPath, "utf-8");
    const index = JSON.parse(content) as StoreIndex;

    // Check version compatibility
    if (index.version !== INDEX_VERSION) {
      return null;
    }

    return index;
  } catch {
    return null;
  }
}

/**
 * Save index to disk.
 */
export function saveIndex(storePath: string, index: StoreIndex): void {
  const indexPath = getIndexPath(storePath);
  const cacheDir = path.dirname(indexPath);

  // Ensure cache directory exists
  fs.mkdirSync(cacheDir, { recursive: true });

  // Write index with pretty formatting for debugging
  fs.writeFileSync(indexPath, JSON.stringify(index, null, 2));
}

/**
 * Create an empty index.
 */
export function createEmptyIndex(): StoreIndex {
  return {
    version: INDEX_VERSION,
    built_at: new Date().toISOString(),
    metadata: {},
    tags: {},
    backlinks: {},
    edges: [],
    mtimes: {},
  };
}

/**
 * Extract all links from a note (both inline and typed).
 */
export function extractAllLinks(note: Note): {
  inlineLinks: InlineLink[];
  typedLinks: TypedLink[];
} {
  const inlineLinks = extractInlineLinks(note.body);
  const typedLinks = note.frontmatter.links || [];

  return { inlineLinks, typedLinks };
}

/**
 * Index a single note and return its metadata and edges.
 */
export function indexNote(note: Note): {
  metadata: NoteMetadata;
  edges: GraphEdge[];
} {
  const fm = note.frontmatter;
  const id = fm.id;

  const metadata: NoteMetadata = {
    id,
    title: fm.title,
    type: fm.type || "fleeting",
    tags: fm.tags || [],
    path: note.path || "",
    created: fm.created || "",
    updated: fm.updated || "",
  };

  const edges: GraphEdge[] = [];

  // Extract inline links
  const { inlineLinks, typedLinks } = extractAllLinks(note);

  // Convert inline links to edges
  for (const link of inlineLinks) {
    edges.push({
      from: id,
      to: link.target,
      type: "related",
      source: "inline",
    });
  }

  // Add typed links as edges
  for (const link of typedLinks) {
    edges.push({
      from: id,
      to: link.id,
      type: link.type,
      source: "typed",
    });
  }

  return { metadata, edges };
}

/**
 * Build complete index from scratch.
 */
export function buildIndex(storePath: string): StoreIndex {
  const index = createEmptyIndex();
  const notes = listNotes(storePath);

  for (const note of notes) {
    if (!note.path) continue;

    // Get file mtime
    try {
      const stat = fs.statSync(note.path);
      index.mtimes[note.frontmatter.id] = stat.mtimeMs;
    } catch {
      // Skip if file is inaccessible
      continue;
    }

    const { metadata, edges } = indexNote(note);

    // Add to metadata index
    index.metadata[metadata.id] = metadata;

    // Add to tag index
    for (const tag of metadata.tags) {
      if (!index.tags[tag]) {
        index.tags[tag] = [];
      }
      if (!index.tags[tag].includes(metadata.id)) {
        index.tags[tag].push(metadata.id);
      }
    }

    // Add edges
    index.edges.push(...edges);
  }

  // Build backlink index from edges
  for (const edge of index.edges) {
    if (!index.backlinks[edge.to]) {
      index.backlinks[edge.to] = [];
    }
    if (!index.backlinks[edge.to].includes(edge.from)) {
      index.backlinks[edge.to].push(edge.from);
    }
  }

  // Sort tag arrays for deterministic output
  for (const tag of Object.keys(index.tags)) {
    index.tags[tag].sort();
  }

  // Sort backlink arrays for deterministic output
  for (const id of Object.keys(index.backlinks)) {
    index.backlinks[id].sort();
  }

  // Sort edges for deterministic output
  index.edges.sort((a, b) => {
    if (a.from !== b.from) return a.from.localeCompare(b.from);
    if (a.to !== b.to) return a.to.localeCompare(b.to);
    return a.type.localeCompare(b.type);
  });

  index.built_at = new Date().toISOString();

  return index;
}

/**
 * Check if a note needs reindexing based on mtime.
 */
function needsReindex(
  notePath: string,
  noteId: string,
  existingIndex: StoreIndex,
): boolean {
  try {
    const stat = fs.statSync(notePath);
    const storedMtime = existingIndex.mtimes[noteId];

    if (!storedMtime) {
      return true; // New note
    }

    return stat.mtimeMs > storedMtime;
  } catch {
    return true;
  }
}

/**
 * Update index incrementally (only changed files).
 * Falls back to full rebuild if incremental update is not possible.
 */
export function updateIndex(
  storePath: string,
  existingIndex: StoreIndex | null,
): StoreIndex {
  // If no existing index, do full rebuild
  if (!existingIndex) {
    return buildIndex(storePath);
  }

  const notes = listNotes(storePath);
  const currentIds = new Set(notes.map((n) => n.frontmatter.id));
  const indexedIds = new Set(Object.keys(existingIndex.metadata));

  // Check for removed notes
  const removedIds = [...indexedIds].filter((id) => !currentIds.has(id));

  // Check for new/changed notes
  const changedNotes = notes.filter((note) => {
    if (!note.path) return false;
    return needsReindex(note.path, note.frontmatter.id, existingIndex);
  });

  // If no changes, return existing index
  if (removedIds.length === 0 && changedNotes.length === 0) {
    return existingIndex;
  }

  // For simplicity, do full rebuild if anything changed
  // A more sophisticated implementation could do incremental updates
  return buildIndex(storePath);
}

/**
 * Get or build index (load from cache if valid, otherwise rebuild).
 */
export function getIndex(storePath: string, forceRebuild = false): StoreIndex {
  if (forceRebuild) {
    const index = buildIndex(storePath);
    saveIndex(storePath, index);
    return index;
  }

  const existing = loadIndex(storePath);
  const updated = updateIndex(storePath, existing);

  // Save if index was updated
  if (updated !== existing) {
    saveIndex(storePath, updated);
  }

  return updated;
}

/**
 * Get notes by tag using the index.
 */
export function getNotesByTag(index: StoreIndex, tag: string): string[] {
  return index.tags[tag] || [];
}

/**
 * Get backlinks for a note using the index.
 */
export function getBacklinks(index: StoreIndex, id: string): string[] {
  return index.backlinks[id] || [];
}

/**
 * Get outgoing links for a note using the index.
 */
export function getOutgoingLinks(index: StoreIndex, id: string): GraphEdge[] {
  return index.edges.filter((edge) => edge.from === id);
}

/**
 * Get incoming links for a note using the index.
 */
export function getIncomingLinks(index: StoreIndex, id: string): GraphEdge[] {
  return index.edges.filter((edge) => edge.to === id);
}

/**
 * Get all tags in the store.
 */
export function getAllTags(index: StoreIndex): string[] {
  return Object.keys(index.tags).sort();
}

/**
 * Get note metadata by ID.
 */
export function getMetadata(
  index: StoreIndex,
  id: string,
): NoteMetadata | null {
  return index.metadata[id] || null;
}
