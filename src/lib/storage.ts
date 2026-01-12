/**
 * Storage layer for Qipu stores.
 *
 * Handles store discovery, initialization, and note file operations.
 * Based on specs/storage-format.md.
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as crypto from "node:crypto";
import matter from "gray-matter";
import * as toml from "toml";
import {
  Note,
  NoteFrontmatter,
  StoreConfig,
  DEFAULT_CONFIG,
  NoteType,
  TypedLink,
} from "./models.js";

/** Store directory name (hidden by default) */
export const STORE_DIR = ".qipu";

/** Store subdirectories */
export const STORE_SUBDIRS = [
  "notes",
  "mocs",
  "attachments",
  "templates",
  ".cache",
] as const;

/** Config file name */
export const CONFIG_FILE = "config.toml";

/**
 * Result of store discovery.
 */
export interface StoreLocation {
  /** Absolute path to the store directory (.qipu/) */
  storePath: string;
  /** Absolute path to the store root (parent of .qipu/) */
  rootPath: string;
}

/**
 * Walk up from the given directory looking for a .qipu/ directory.
 * Returns null if not found.
 */
export function discoverStore(startDir: string): StoreLocation | null {
  let current = path.resolve(startDir);
  const root = path.parse(current).root;

  while (current !== root) {
    const candidate = path.join(current, STORE_DIR);
    if (fs.existsSync(candidate) && fs.statSync(candidate).isDirectory()) {
      return {
        storePath: candidate,
        rootPath: current,
      };
    }
    current = path.dirname(current);
  }

  // Check root directory as well
  const rootCandidate = path.join(root, STORE_DIR);
  if (
    fs.existsSync(rootCandidate) &&
    fs.statSync(rootCandidate).isDirectory()
  ) {
    return {
      storePath: rootCandidate,
      rootPath: root,
    };
  }

  return null;
}

/**
 * Resolve store location from command options.
 *
 * Resolution order per spec:
 * 1. If --store provided, resolve relative to --root (or cwd)
 * 2. Otherwise, walk up from --root/cwd looking for .qipu/
 */
export function resolveStore(options: {
  store?: string;
  root?: string;
}): StoreLocation | null {
  const rootDir = options.root ? path.resolve(options.root) : process.cwd();

  if (options.store) {
    const storePath = path.resolve(rootDir, options.store);
    if (fs.existsSync(storePath) && fs.statSync(storePath).isDirectory()) {
      return {
        storePath,
        rootPath: path.dirname(storePath),
      };
    }
    return null;
  }

  return discoverStore(rootDir);
}

/**
 * Initialize a new store at the given location.
 */
export function initStore(
  rootPath: string,
  options: { stealth?: boolean; visible?: boolean } = {},
): StoreLocation {
  const storeDirName = options.visible ? "qipu" : STORE_DIR;
  const storePath = path.join(rootPath, storeDirName);

  // Create store directory
  fs.mkdirSync(storePath, { recursive: true });

  // Create subdirectories
  for (const subdir of STORE_SUBDIRS) {
    fs.mkdirSync(path.join(storePath, subdir), { recursive: true });
  }

  // Create default config
  const config: StoreConfig = { ...DEFAULT_CONFIG };
  const configPath = path.join(storePath, CONFIG_FILE);
  if (!fs.existsSync(configPath)) {
    const configContent = `# Qipu store configuration
format_version = ${config.format_version}
id_scheme = "${config.id_scheme}"
default_note_type = "${config.default_note_type}"
`;
    fs.writeFileSync(configPath, configContent);
  }

  // Create .gitignore in .cache
  const cacheGitignore = path.join(storePath, ".cache", ".gitignore");
  if (!fs.existsSync(cacheGitignore)) {
    fs.writeFileSync(cacheGitignore, "*\n!.gitignore\n");
  }

  // Handle stealth mode - add store to parent's .gitignore
  if (options.stealth) {
    const parentGitignore = path.join(rootPath, ".gitignore");
    const ignoreEntry = `${storeDirName}/\n`;
    if (fs.existsSync(parentGitignore)) {
      const content = fs.readFileSync(parentGitignore, "utf-8");
      if (!content.includes(storeDirName)) {
        fs.appendFileSync(parentGitignore, ignoreEntry);
      }
    } else {
      fs.writeFileSync(parentGitignore, ignoreEntry);
    }
  }

  return { storePath, rootPath };
}

/**
 * Load store configuration.
 */
export function loadConfig(storePath: string): StoreConfig {
  const configPath = path.join(storePath, CONFIG_FILE);
  if (!fs.existsSync(configPath)) {
    return { ...DEFAULT_CONFIG };
  }

  try {
    const content = fs.readFileSync(configPath, "utf-8");
    const parsed = toml.parse(content);
    return { ...DEFAULT_CONFIG, ...parsed };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Generate a note ID using the hash scheme.
 * Format: qp-<hash> with adaptive length (minimum 4 chars, expand if collision).
 */
export function generateId(
  title: string,
  timestamp: Date = new Date(),
): string {
  const input = `${title}:${timestamp.toISOString()}:${Math.random()}`;
  const hash = crypto.createHash("sha256").update(input).digest("hex");
  // Start with 4 characters, can expand later if collisions detected
  return `qp-${hash.slice(0, 4)}`;
}

/**
 * Generate a URL-safe slug from a title.
 */
export function slugify(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 50); // Limit slug length
}

/**
 * Generate the filename for a note.
 * Format: <id>-<slug>.md
 */
export function noteFilename(id: string, title: string): string {
  const slug = slugify(title);
  return slug ? `${id}-${slug}.md` : `${id}.md`;
}

/**
 * Determine the directory for a note based on its type.
 */
export function noteDirectory(
  storePath: string,
  type: NoteType = "fleeting",
): string {
  if (type === "moc") {
    return path.join(storePath, "mocs");
  }
  return path.join(storePath, "notes");
}

/**
 * Parse a note file from its content.
 */
export function parseNote(content: string, filePath?: string): Note {
  const { data, content: body } = matter(content);
  const frontmatter = data as NoteFrontmatter;

  return {
    frontmatter,
    body: body.trim(),
    path: filePath,
  };
}

/**
 * Serialize a note to file content.
 * Uses deterministic key ordering for stable output.
 */
export function serializeNote(note: Note): string {
  const fm = note.frontmatter;

  // Build frontmatter object with deterministic key order
  const ordered: Record<string, unknown> = {};

  // Required fields first
  ordered.id = fm.id;
  ordered.title = fm.title;

  // Optional fields in consistent order
  if (fm.type) ordered.type = fm.type;
  if (fm.created) ordered.created = fm.created;
  if (fm.updated) ordered.updated = fm.updated;
  if (fm.tags && fm.tags.length > 0) ordered.tags = fm.tags;
  if (fm.summary) ordered.summary = fm.summary;
  if (fm.sources && fm.sources.length > 0) ordered.sources = fm.sources;
  if (fm.links && fm.links.length > 0) {
    // Ensure links have consistent structure
    ordered.links = fm.links.map((link: TypedLink) => ({
      type: link.type,
      id: link.id,
    }));
  }

  return matter.stringify(note.body, ordered);
}

/**
 * Read a note from a file path.
 */
export function readNote(filePath: string): Note {
  const content = fs.readFileSync(filePath, "utf-8");
  return parseNote(content, filePath);
}

/**
 * Write a note to a file path.
 */
export function writeNote(note: Note, filePath: string): void {
  const content = serializeNote(note);
  fs.writeFileSync(filePath, content);
}

/**
 * Create a new note and save it to the store.
 */
export function createNote(
  storePath: string,
  options: {
    title: string;
    type?: NoteType;
    tags?: string[];
    body?: string;
    sources?: { url?: string; title?: string }[];
    links?: TypedLink[];
  },
): Note {
  const now = new Date().toISOString();
  const type = options.type || DEFAULT_CONFIG.default_note_type;
  const id = generateId(options.title);

  const frontmatter: NoteFrontmatter = {
    id,
    title: options.title,
    type,
    created: now,
    updated: now,
  };

  if (options.tags && options.tags.length > 0) {
    frontmatter.tags = options.tags;
  }

  if (options.sources && options.sources.length > 0) {
    frontmatter.sources = options.sources;
  }

  if (options.links && options.links.length > 0) {
    frontmatter.links = options.links;
  }

  const note: Note = {
    frontmatter,
    body: options.body || "",
  };

  const dir = noteDirectory(storePath, type);
  const filename = noteFilename(id, options.title);
  const filePath = path.join(dir, filename);

  note.path = filePath;
  writeNote(note, filePath);

  return note;
}

/**
 * List all notes in the store.
 */
export function listNotes(
  storePath: string,
  options: { type?: NoteType; tag?: string } = {},
): Note[] {
  const notes: Note[] = [];
  const dirs = [path.join(storePath, "notes"), path.join(storePath, "mocs")];

  for (const dir of dirs) {
    if (!fs.existsSync(dir)) continue;

    const files = fs.readdirSync(dir).filter((f) => f.endsWith(".md"));
    for (const file of files) {
      const filePath = path.join(dir, file);
      try {
        const note = readNote(filePath);

        // Apply filters
        if (options.type && note.frontmatter.type !== options.type) continue;
        if (options.tag && !note.frontmatter.tags?.includes(options.tag))
          continue;

        notes.push(note);
      } catch {
        // Skip invalid files
      }
    }
  }

  // Sort deterministically by created date, then by ID
  notes.sort((a, b) => {
    const dateA = a.frontmatter.created || "";
    const dateB = b.frontmatter.created || "";
    if (dateA !== dateB) return dateA.localeCompare(dateB);
    return a.frontmatter.id.localeCompare(b.frontmatter.id);
  });

  return notes;
}

/**
 * Find a note by ID or path.
 */
export function findNote(storePath: string, idOrPath: string): Note | null {
  // If it looks like a path, try to read it directly
  if (idOrPath.includes("/") || idOrPath.endsWith(".md")) {
    const filePath = path.isAbsolute(idOrPath)
      ? idOrPath
      : path.join(storePath, "..", idOrPath);

    if (fs.existsSync(filePath)) {
      return readNote(filePath);
    }
  }

  // Otherwise, search by ID
  const notes = listNotes(storePath);
  return notes.find((n) => n.frontmatter.id === idOrPath) || null;
}
