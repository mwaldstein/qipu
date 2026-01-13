/**
 * Tests for the indexing system.
 *
 * Why: The index is the foundation for fast navigation, search, and graph
 * traversal. These tests verify:
 * - Index correctly captures note metadata
 * - Tag index enables O(1) tag lookups
 * - Backlink index tracks inbound references
 * - Edge list captures all link relationships
 * - Incremental updates work correctly
 * - Index persistence and loading
 */

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { initStore, createNote, writeNote, readNote } from "./storage.js";
import {
  buildIndex,
  loadIndex,
  saveIndex,
  getIndex,
  getBacklinks,
  getNotesByTag,
  getOutgoingLinks,
  getAllTags,
  getMetadata,
  getIndexPath,
} from "./indexing.js";

describe("Indexing System", () => {
  let tempDir: string;
  let storePath: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-index-test-"));
    const result = initStore(tempDir);
    storePath = result.storePath;
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  describe("buildIndex", () => {
    it("should create empty index for empty store", () => {
      const index = buildIndex(storePath);

      expect(index.version).toBe(1);
      expect(Object.keys(index.metadata)).toHaveLength(0);
      expect(Object.keys(index.tags)).toHaveLength(0);
      expect(index.edges).toHaveLength(0);
    });

    it("should index note metadata correctly", () => {
      const note = createNote(storePath, {
        title: "Test Note",
        type: "permanent",
        tags: ["testing", "example"],
      });

      const index = buildIndex(storePath);

      expect(index.metadata[note.frontmatter.id]).toBeDefined();
      const meta = index.metadata[note.frontmatter.id];
      expect(meta.title).toBe("Test Note");
      expect(meta.type).toBe("permanent");
      expect(meta.tags).toEqual(["testing", "example"]);
    });

    it("should build tag index", () => {
      createNote(storePath, { title: "Note 1", tags: ["alpha", "beta"] });
      createNote(storePath, { title: "Note 2", tags: ["beta", "gamma"] });
      createNote(storePath, { title: "Note 3", tags: ["alpha"] });

      const index = buildIndex(storePath);

      expect(index.tags["alpha"]).toHaveLength(2);
      expect(index.tags["beta"]).toHaveLength(2);
      expect(index.tags["gamma"]).toHaveLength(1);
    });

    it("should extract inline wiki links", () => {
      const note1 = createNote(storePath, { title: "Note 1" });
      const note2 = createNote(storePath, {
        title: "Note 2",
        body: `This links to [[${note1.frontmatter.id}]].`,
      });

      const index = buildIndex(storePath);
      const edges = index.edges.filter((e) => e.from === note2.frontmatter.id);

      expect(edges).toHaveLength(1);
      expect(edges[0].to).toBe(note1.frontmatter.id);
      expect(edges[0].type).toBe("related");
      expect(edges[0].source).toBe("inline");
    });

    it("should build backlink index from edges", () => {
      const note1 = createNote(storePath, { title: "Target Note" });
      const note2 = createNote(storePath, {
        title: "Linking Note",
        body: `See [[${note1.frontmatter.id}]].`,
      });

      const index = buildIndex(storePath);
      const backlinks = index.backlinks[note1.frontmatter.id] || [];

      expect(backlinks).toContain(note2.frontmatter.id);
    });

    it("should include typed links from frontmatter", () => {
      const note1 = createNote(storePath, { title: "Source Note" });
      const note2 = createNote(storePath, {
        title: "Derived Note",
        links: [{ id: note1.frontmatter.id, type: "derived-from" }],
      });

      const index = buildIndex(storePath);
      const edges = index.edges.filter(
        (e) => e.from === note2.frontmatter.id && e.source === "typed",
      );

      expect(edges).toHaveLength(1);
      expect(edges[0].type).toBe("derived-from");
      expect(edges[0].to).toBe(note1.frontmatter.id);
    });

    it("should produce deterministic output", () => {
      createNote(storePath, { title: "Zebra", tags: ["c", "a", "b"] });
      createNote(storePath, { title: "Apple", tags: ["b", "d"] });

      const index1 = buildIndex(storePath);
      const index2 = buildIndex(storePath);

      // Tags should be sorted
      expect(getAllTags(index1)).toEqual(getAllTags(index2));
      expect(getAllTags(index1)).toEqual(["a", "b", "c", "d"]);

      // Edges should be sorted
      expect(index1.edges).toEqual(index2.edges);
    });
  });

  describe("Index persistence", () => {
    it("should save and load index", () => {
      createNote(storePath, { title: "Test Note", tags: ["example"] });

      const original = buildIndex(storePath);
      saveIndex(storePath, original);

      const loaded = loadIndex(storePath);

      expect(loaded).not.toBeNull();
      expect(loaded!.version).toBe(original.version);
      expect(loaded!.metadata).toEqual(original.metadata);
      expect(loaded!.tags).toEqual(original.tags);
    });

    it("should return null for missing index", () => {
      const loaded = loadIndex(storePath);
      expect(loaded).toBeNull();
    });

    it("should create index file in .cache directory", () => {
      createNote(storePath, { title: "Test" });

      const index = buildIndex(storePath);
      saveIndex(storePath, index);

      const indexPath = getIndexPath(storePath);
      expect(fs.existsSync(indexPath)).toBe(true);
      expect(indexPath).toContain(".cache");
    });
  });

  describe("getIndex", () => {
    it("should build and cache index on first call", () => {
      createNote(storePath, { title: "Test Note" });

      const index = getIndex(storePath);

      expect(Object.keys(index.metadata)).toHaveLength(1);
      expect(fs.existsSync(getIndexPath(storePath))).toBe(true);
    });

    it("should rebuild when forceRebuild is true", () => {
      createNote(storePath, { title: "Note 1" });
      const index1 = getIndex(storePath);

      createNote(storePath, { title: "Note 2" });
      const index2 = getIndex(storePath, true);

      expect(Object.keys(index2.metadata)).toHaveLength(2);
    });
  });

  describe("Index query helpers", () => {
    it("getNotesByTag should return matching note IDs", () => {
      const note1 = createNote(storePath, { title: "A", tags: ["shared"] });
      const note2 = createNote(storePath, { title: "B", tags: ["shared"] });
      createNote(storePath, { title: "C", tags: ["other"] });

      const index = buildIndex(storePath);
      const results = getNotesByTag(index, "shared");

      expect(results).toContain(note1.frontmatter.id);
      expect(results).toContain(note2.frontmatter.id);
      expect(results).toHaveLength(2);
    });

    it("getBacklinks should return linking note IDs", () => {
      const target = createNote(storePath, { title: "Target" });
      const linker1 = createNote(storePath, {
        title: "Linker 1",
        body: `[[${target.frontmatter.id}]]`,
      });
      const linker2 = createNote(storePath, {
        title: "Linker 2",
        links: [{ id: target.frontmatter.id, type: "related" }],
      });

      const index = buildIndex(storePath);
      const backlinks = getBacklinks(index, target.frontmatter.id);

      expect(backlinks).toContain(linker1.frontmatter.id);
      expect(backlinks).toContain(linker2.frontmatter.id);
    });

    it("getOutgoingLinks should return edges from a note", () => {
      const target1 = createNote(storePath, { title: "Target 1" });
      const target2 = createNote(storePath, { title: "Target 2" });
      const source = createNote(storePath, {
        title: "Source",
        body: `[[${target1.frontmatter.id}]]`,
        links: [{ id: target2.frontmatter.id, type: "supports" }],
      });

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, source.frontmatter.id);

      expect(outgoing).toHaveLength(2);
      expect(outgoing.map((e) => e.to)).toContain(target1.frontmatter.id);
      expect(outgoing.map((e) => e.to)).toContain(target2.frontmatter.id);
    });

    it("getMetadata should return note metadata by ID", () => {
      const note = createNote(storePath, {
        title: "Metadata Test",
        type: "literature",
        tags: ["test"],
      });

      const index = buildIndex(storePath);
      const meta = getMetadata(index, note.frontmatter.id);

      expect(meta).not.toBeNull();
      expect(meta!.title).toBe("Metadata Test");
      expect(meta!.type).toBe("literature");
    });
  });
});
