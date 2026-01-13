/**
 * Qipu - Zettelkasten-inspired knowledge management for LLM workflows.
 *
 * This is the library entry point for programmatic usage.
 * For CLI usage, see cli.ts.
 */

// Re-export models
export * from "./lib/models.js";

// Re-export storage functions
export {
  discoverStore,
  resolveStore,
  initStore,
  loadConfig,
  generateId,
  slugify,
  noteFilename,
  noteDirectory,
  parseNote,
  serializeNote,
  readNote,
  writeNote,
  createNote,
  listNotes,
  findNote,
  STORE_DIR,
  STORE_SUBDIRS,
  CONFIG_FILE,
} from "./lib/storage.js";

// Re-export parsing utilities
export {
  extractWikiLinks,
  extractMarkdownLinks,
  extractInlineLinks,
  inlineToTypedLinks,
  extractSummary,
  extractHashtags,
} from "./lib/parsing.js";

// Re-export indexing functions
export {
  buildIndex,
  loadIndex,
  saveIndex,
  getIndex,
  getBacklinks,
  getNotesByTag,
  getOutgoingLinks,
  getIncomingLinks,
  getAllTags,
  getMetadata,
  getIndexPath,
} from "./lib/indexing.js";

// Re-export indexing types
export type { NoteMetadata, GraphEdge, StoreIndex } from "./lib/indexing.js";
