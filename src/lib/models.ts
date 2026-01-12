/**
 * Core data models for Qipu knowledge management system.
 *
 * Based on specs/knowledge-model.md - implements the Zettelkasten-inspired
 * note types and typed link system for both human note-taking and LLM workflows.
 */

/**
 * Note types following Zettelkasten methodology:
 * - fleeting: Quick capture, low ceremony, meant to be refined later
 * - literature: Notes derived from external sources (URLs, books, papers)
 * - permanent: Distilled insights, author's own words, standalone
 * - moc: Map of Content - curated index notes organizing topics
 */
export type NoteType = "fleeting" | "literature" | "permanent" | "moc";

/**
 * Typed link relationships for machine-friendly navigation:
 * - related: Soft relationship between notes
 * - derived-from: Note was created because of another note
 * - supports: Evidence supports a claim
 * - contradicts: Evidence contradicts a claim
 * - part-of: Note is part of a larger outline/MOC
 * - compacts: Digest note summarizes source notes (Phase 7)
 */
export type LinkType =
  | "related"
  | "derived-from"
  | "supports"
  | "contradicts"
  | "part-of"
  | "compacts";

/**
 * Source of a link - where it was defined:
 * - inline: Wiki-link in body text
 * - typed: Explicit typed link in frontmatter
 */
export type LinkSource = "inline" | "typed";

/**
 * A typed link to another note.
 */
export interface TypedLink {
  /** Target note ID (e.g., "qp-a1b2") */
  id: string;
  /** Relationship type */
  type: LinkType;
  /** Source of the link */
  source?: LinkSource;
}

/**
 * A source reference (for literature notes).
 */
export interface Source {
  /** URL of the source */
  url?: string;
  /** Title of the source */
  title?: string;
  /** When the source was accessed */
  accessed?: string;
}

/**
 * YAML frontmatter structure for a note file.
 */
export interface NoteFrontmatter {
  /** Unique note ID (e.g., "qp-a1b2") */
  id: string;
  /** Note title */
  title: string;
  /** Note type */
  type?: NoteType;
  /** ISO 8601 creation timestamp */
  created?: string;
  /** ISO 8601 last updated timestamp */
  updated?: string;
  /** List of tags */
  tags?: string[];
  /** List of source references */
  sources?: Source[];
  /** List of typed links */
  links?: TypedLink[];
  /** Optional summary for token-optimized output */
  summary?: string;
}

/**
 * A complete note with frontmatter and body content.
 */
export interface Note {
  /** YAML frontmatter data */
  frontmatter: NoteFrontmatter;
  /** Markdown body content */
  body: string;
  /** File path relative to store root */
  path?: string;
}

/**
 * Store configuration from config.toml.
 */
export interface StoreConfig {
  /** Format version for forward compatibility */
  format_version?: number;
  /** ID generation scheme */
  id_scheme?: "hash" | "ulid" | "timestamp";
  /** Default note type for new notes */
  default_note_type?: NoteType;
  /** Editor preference override */
  editor?: string;
}

/**
 * Default store configuration values.
 */
export const DEFAULT_CONFIG: Required<StoreConfig> = {
  format_version: 1,
  id_scheme: "hash",
  default_note_type: "fleeting",
  editor: "",
};

/**
 * Exit codes per specs/cli-interface.md.
 */
export const ExitCodes = {
  SUCCESS: 0,
  FAILURE: 1,
  USAGE_ERROR: 2,
  DATA_ERROR: 3,
} as const;

export type ExitCode = (typeof ExitCodes)[keyof typeof ExitCodes];
