/**
 * Tests for storage layer.
 */

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import {
  initStore,
  discoverStore,
  resolveStore,
  loadConfig,
  generateId,
  slugify,
  noteFilename,
  parseNote,
  serializeNote,
  createNote,
  listNotes,
  findNote,
  STORE_DIR,
  STORE_SUBDIRS,
} from "./storage.js";

describe("Storage Layer", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-test-"));
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  describe("initStore", () => {
    it("should create store directory structure", () => {
      const { storePath } = initStore(tempDir);

      expect(fs.existsSync(storePath)).toBe(true);
      for (const subdir of STORE_SUBDIRS) {
        expect(fs.existsSync(path.join(storePath, subdir))).toBe(true);
      }
    });

    it("should create config.toml with defaults", () => {
      const { storePath } = initStore(tempDir);
      const configPath = path.join(storePath, "config.toml");

      expect(fs.existsSync(configPath)).toBe(true);
      const content = fs.readFileSync(configPath, "utf-8");
      expect(content).toContain("format_version");
      expect(content).toContain("id_scheme");
    });

    it("should support stealth mode", () => {
      initStore(tempDir, { stealth: true });
      const gitignore = path.join(tempDir, ".gitignore");

      expect(fs.existsSync(gitignore)).toBe(true);
      const content = fs.readFileSync(gitignore, "utf-8");
      expect(content).toContain(STORE_DIR);
    });

    it("should support visible mode", () => {
      const { storePath } = initStore(tempDir, { visible: true });

      expect(storePath).toContain("/qipu");
      expect(storePath).not.toContain("/.qipu");
      expect(fs.existsSync(storePath)).toBe(true);
    });
  });

  describe("discoverStore", () => {
    it("should find store in current directory", () => {
      initStore(tempDir);
      const result = discoverStore(tempDir);

      expect(result).not.toBeNull();
      expect(result!.storePath).toBe(path.join(tempDir, STORE_DIR));
      expect(result!.rootPath).toBe(tempDir);
    });

    it("should find store in parent directory", () => {
      initStore(tempDir);
      const subdir = path.join(tempDir, "subdir");
      fs.mkdirSync(subdir);

      const result = discoverStore(subdir);

      expect(result).not.toBeNull();
      expect(result!.storePath).toBe(path.join(tempDir, STORE_DIR));
    });

    it("should return null when no store exists", () => {
      const result = discoverStore(tempDir);
      expect(result).toBeNull();
    });
  });

  describe("resolveStore", () => {
    it("should use --store option when provided", () => {
      const { storePath } = initStore(tempDir);
      const result = resolveStore({ store: storePath });

      expect(result).not.toBeNull();
      expect(result!.storePath).toBe(storePath);
    });

    it("should walk up when --store not provided", () => {
      initStore(tempDir);
      const subdir = path.join(tempDir, "deep", "nested");
      fs.mkdirSync(subdir, { recursive: true });

      const result = resolveStore({ root: subdir });

      expect(result).not.toBeNull();
      expect(result!.storePath).toBe(path.join(tempDir, STORE_DIR));
    });
  });

  describe("loadConfig", () => {
    it("should load config with defaults", () => {
      const { storePath } = initStore(tempDir);
      const config = loadConfig(storePath);

      expect(config.format_version).toBe(1);
      expect(config.id_scheme).toBe("hash");
      expect(config.default_note_type).toBe("fleeting");
    });

    it("should return defaults when config missing", () => {
      const config = loadConfig("/nonexistent");

      expect(config.format_version).toBe(1);
      expect(config.id_scheme).toBe("hash");
    });
  });

  describe("generateId", () => {
    it("should generate IDs in qp-XXXX format", () => {
      const id = generateId("Test Note");

      expect(id).toMatch(/^qp-[a-f0-9]{4}$/);
    });

    it("should generate different IDs for same title", () => {
      const id1 = generateId("Test Note");
      const id2 = generateId("Test Note");

      expect(id1).not.toBe(id2);
    });
  });

  describe("slugify", () => {
    it("should convert title to URL-safe slug", () => {
      expect(slugify("Hello World")).toBe("hello-world");
      expect(slugify("Test & Notes!")).toBe("test-notes");
      expect(slugify("  Trim Spaces  ")).toBe("trim-spaces");
    });

    it("should limit slug length", () => {
      const longTitle =
        "This is a very long title that should be truncated to a reasonable length";
      const slug = slugify(longTitle);

      expect(slug.length).toBeLessThanOrEqual(50);
    });
  });

  describe("noteFilename", () => {
    it("should combine ID and slug", () => {
      expect(noteFilename("qp-abc1", "Hello World")).toBe(
        "qp-abc1-hello-world.md",
      );
    });

    it("should handle empty title", () => {
      expect(noteFilename("qp-abc1", "")).toBe("qp-abc1.md");
    });
  });

  describe("parseNote and serializeNote", () => {
    it("should round-trip note content", () => {
      const content = `---
id: qp-test
title: Test Note
type: permanent
created: 2026-01-12T00:00:00Z
updated: 2026-01-12T00:00:00Z
tags:
  - test
  - example
---

This is the body content.

With multiple paragraphs.`;

      const note = parseNote(content);
      expect(note.frontmatter.id).toBe("qp-test");
      expect(note.frontmatter.title).toBe("Test Note");
      expect(note.frontmatter.type).toBe("permanent");
      expect(note.frontmatter.tags).toEqual(["test", "example"]);
      expect(note.body).toContain("This is the body content");

      const serialized = serializeNote(note);
      const reparsed = parseNote(serialized);
      expect(reparsed.frontmatter.id).toBe(note.frontmatter.id);
      expect(reparsed.frontmatter.title).toBe(note.frontmatter.title);
      expect(reparsed.body).toBe(note.body);
    });
  });

  describe("createNote", () => {
    it("should create note file in store", () => {
      const { storePath } = initStore(tempDir);

      const note = createNote(storePath, {
        title: "My Test Note",
        type: "permanent",
        tags: ["test"],
      });

      expect(note.frontmatter.id).toMatch(/^qp-[a-f0-9]{4}$/);
      expect(note.frontmatter.title).toBe("My Test Note");
      expect(note.frontmatter.type).toBe("permanent");
      expect(note.path).toBeDefined();
      expect(fs.existsSync(note.path!)).toBe(true);
    });

    it("should store MOC notes in mocs directory", () => {
      const { storePath } = initStore(tempDir);

      const note = createNote(storePath, {
        title: "My MOC",
        type: "moc",
      });

      expect(note.path).toContain("/mocs/");
    });
  });

  describe("listNotes", () => {
    it("should list all notes", () => {
      const { storePath } = initStore(tempDir);

      createNote(storePath, { title: "Note 1" });
      createNote(storePath, { title: "Note 2" });

      const notes = listNotes(storePath);
      expect(notes).toHaveLength(2);
    });

    it("should filter by type", () => {
      const { storePath } = initStore(tempDir);

      createNote(storePath, { title: "Note 1", type: "permanent" });
      createNote(storePath, { title: "Note 2", type: "fleeting" });

      const notes = listNotes(storePath, { type: "permanent" });
      expect(notes).toHaveLength(1);
      expect(notes[0].frontmatter.type).toBe("permanent");
    });

    it("should filter by tag", () => {
      const { storePath } = initStore(tempDir);

      createNote(storePath, { title: "Note 1", tags: ["important"] });
      createNote(storePath, { title: "Note 2", tags: ["other"] });

      const notes = listNotes(storePath, { tag: "important" });
      expect(notes).toHaveLength(1);
      expect(notes[0].frontmatter.tags).toContain("important");
    });

    it("should return notes in deterministic order", () => {
      const { storePath } = initStore(tempDir);

      // Create notes with slight delay to ensure different timestamps
      createNote(storePath, { title: "Note A" });
      createNote(storePath, { title: "Note B" });

      const notes1 = listNotes(storePath);
      const notes2 = listNotes(storePath);

      expect(notes1.map((n) => n.frontmatter.id)).toEqual(
        notes2.map((n) => n.frontmatter.id),
      );
    });
  });

  describe("findNote", () => {
    it("should find note by ID", () => {
      const { storePath } = initStore(tempDir);
      const created = createNote(storePath, { title: "Test Note" });

      const found = findNote(storePath, created.frontmatter.id);

      expect(found).not.toBeNull();
      expect(found!.frontmatter.id).toBe(created.frontmatter.id);
    });

    it("should return null for unknown ID", () => {
      const { storePath } = initStore(tempDir);

      const found = findNote(storePath, "qp-nonexistent");

      expect(found).toBeNull();
    });
  });
});
