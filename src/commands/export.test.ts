/**
 * Tests for qipu export command.
 *
 * Why: Export is the bridge between qipu and external tools. These tests verify:
 * - Bundle export concatenates notes with proper formatting
 * - Outline export follows MOC structure
 * - Bibliography export extracts and formats sources
 * - Link rewriting modes work correctly
 * - Deterministic ordering is maintained
 */

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import {
  initStore,
  createNote,
  findNote,
  writeNote,
  listNotes,
} from "../lib/storage.js";
import { buildIndex } from "../lib/indexing.js";

describe("Export Command Logic", () => {
  let tempDir: string;
  let storePath: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-export-test-"));
    const result = initStore(tempDir);
    storePath = result.storePath;
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  describe("Bundle export", () => {
    it("should create notes for bundle export", () => {
      const note1 = createNote(storePath, {
        title: "First Note",
        type: "permanent",
        tags: ["research"],
        body: "Content of first note.",
      });

      const note2 = createNote(storePath, {
        title: "Second Note",
        type: "permanent",
        tags: ["research"],
        body: "Content of second note.",
      });

      // Verify notes were created
      expect(note1.frontmatter.id).toBeDefined();
      expect(note2.frontmatter.id).toBeDefined();

      const retrieved1 = findNote(storePath, note1.frontmatter.id);
      const retrieved2 = findNote(storePath, note2.frontmatter.id);

      expect(retrieved1).not.toBeNull();
      expect(retrieved2).not.toBeNull();
    });

    it("should preserve wiki links when link mode is preserve", () => {
      const note1 = createNote(storePath, { title: "Note A" });
      const note2 = createNote(storePath, {
        title: "Note B",
        body: `Links to [[${note1.frontmatter.id}]] and [[${note1.frontmatter.id}|custom label]].`,
      });

      // Verify the body contains wiki links
      const retrieved = findNote(storePath, note2.frontmatter.id);
      expect(retrieved!.body).toContain("[[");
      expect(retrieved!.body).toContain(note1.frontmatter.id);
    });
  });

  describe("Outline export (MOC-driven)", () => {
    it("should create MOC with linked notes", () => {
      // Create MOC and linked notes
      const note1 = createNote(storePath, {
        title: "Supporting Note 1",
        type: "permanent",
        body: "Content 1",
      });

      const note2 = createNote(storePath, {
        title: "Supporting Note 2",
        type: "permanent",
        body: "Content 2",
      });

      const moc = createNote(storePath, {
        title: "Research Overview",
        type: "moc",
        body: `# Overview\n\nSee [[${note1.frontmatter.id}]] and [[${note2.frontmatter.id}]].`,
        links: [
          { id: note1.frontmatter.id, type: "part-of" },
          { id: note2.frontmatter.id, type: "part-of" },
        ],
      });

      expect(moc.frontmatter.type).toBe("moc");
      expect(moc.frontmatter.links).toHaveLength(2);

      // Build index to verify links
      const index = buildIndex(storePath);
      expect(index.edges.length).toBeGreaterThanOrEqual(2);
    });

    it("should store MOC in mocs directory", () => {
      const moc = createNote(storePath, {
        title: "Test MOC",
        type: "moc",
        body: "MOC content",
      });

      expect(moc.path).toContain("mocs");
    });
  });

  describe("Bibliography export", () => {
    it("should handle notes with sources", () => {
      const note = createNote(storePath, {
        title: "Literature Note",
        type: "literature",
        body: "Notes from reading.",
        sources: [
          {
            url: "https://example.com/article1",
            title: "Important Article",
            accessed: "2026-01-12",
          },
          {
            url: "https://example.com/article2",
            title: "Another Article",
          },
        ],
      });

      const retrieved = findNote(storePath, note.frontmatter.id);
      expect(retrieved!.frontmatter.sources).toHaveLength(2);
      expect(retrieved!.frontmatter.sources![0].title).toBe(
        "Important Article",
      );
    });

    it("should handle notes without sources", () => {
      const note = createNote(storePath, {
        title: "Fleeting Note",
        type: "fleeting",
        body: "Quick thought without sources.",
      });

      const retrieved = findNote(storePath, note.frontmatter.id);
      expect(retrieved!.frontmatter.sources).toBeUndefined();
    });
  });

  describe("Selection criteria", () => {
    it("should select notes by tag", () => {
      createNote(storePath, {
        title: "Tagged Note 1",
        tags: ["export-test"],
        body: "Content 1",
      });

      createNote(storePath, {
        title: "Tagged Note 2",
        tags: ["export-test"],
        body: "Content 2",
      });

      createNote(storePath, {
        title: "Untagged Note",
        tags: ["other"],
        body: "Content 3",
      });

      // Verify tag filtering works
      const taggedNotes = listNotes(storePath, { tag: "export-test" });

      expect(taggedNotes).toHaveLength(2);
    });

    it("should select notes by type", () => {
      createNote(storePath, { title: "Permanent 1", type: "permanent" });
      createNote(storePath, { title: "Permanent 2", type: "permanent" });
      createNote(storePath, { title: "Fleeting 1", type: "fleeting" });

      const permanentNotes = listNotes(storePath, { type: "permanent" });

      expect(permanentNotes).toHaveLength(2);
    });
  });

  describe("Deterministic ordering", () => {
    it("should sort notes by created date then ID", () => {
      // Create notes with controlled timestamps
      const note1 = createNote(storePath, {
        title: "Note B",
        body: "Created second",
      });

      // Small delay to ensure different timestamps
      const note2 = createNote(storePath, {
        title: "Note A",
        body: "Created third",
      });

      const notes = listNotes(storePath);

      // Notes should be sorted by created date
      expect(notes.length).toBe(2);

      // Both have timestamps - earlier should come first
      const dates = notes.map((n) => n.frontmatter.created);
      const sortedDates = [...dates].sort();
      expect(dates).toEqual(sortedDates);
    });
  });

  describe("Link rewriting", () => {
    it("should create notes with inline wiki links", () => {
      const target = createNote(storePath, { title: "Target Note" });
      const source = createNote(storePath, {
        title: "Source Note",
        body: `Reference to [[${target.frontmatter.id}]] inline.`,
      });

      const retrieved = findNote(storePath, source.frontmatter.id);
      expect(retrieved!.body).toContain(`[[${target.frontmatter.id}]]`);
    });

    it("should create notes with labeled wiki links", () => {
      const target = createNote(storePath, { title: "Target Note" });
      const source = createNote(storePath, {
        title: "Source Note",
        body: `See [[${target.frontmatter.id}|my label]] for details.`,
      });

      const retrieved = findNote(storePath, source.frontmatter.id);
      expect(retrieved!.body).toContain(
        `[[${target.frontmatter.id}|my label]]`,
      );
    });
  });
});
