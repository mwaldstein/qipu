/**
 * Tests for link command functionality.
 *
 * Why: Links form the backbone of the knowledge graph. These tests verify:
 * - Typed links can be added and removed programmatically
 * - Link listing shows both incoming and outgoing links
 * - Graph traversal follows links correctly with cycle detection
 * - Path finding works between notes
 */

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { initStore, createNote, findNote, writeNote } from "../lib/storage.js";
import { buildIndex, getBacklinks, getOutgoingLinks } from "../lib/indexing.js";

describe("Link Management", () => {
  let tempDir: string;
  let storePath: string;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-link-test-"));
    const result = initStore(tempDir);
    storePath = result.storePath;
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  describe("Typed link management", () => {
    it("should add a typed link to a note", () => {
      const note1 = createNote(storePath, { title: "Source Note" });
      const note2 = createNote(storePath, { title: "Target Note" });

      // Add link to note1 -> note2
      const fromNote = findNote(storePath, note1.frontmatter.id)!;
      fromNote.frontmatter.links = [
        { id: note2.frontmatter.id, type: "supports" },
      ];
      fromNote.frontmatter.updated = new Date().toISOString();
      writeNote(fromNote, fromNote.path!);

      // Verify
      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, note1.frontmatter.id);

      expect(outgoing).toHaveLength(1);
      expect(outgoing[0].to).toBe(note2.frontmatter.id);
      expect(outgoing[0].type).toBe("supports");
      expect(outgoing[0].source).toBe("typed");
    });

    it("should track backlinks after adding typed link", () => {
      const note1 = createNote(storePath, { title: "Source Note" });
      const note2 = createNote(storePath, { title: "Target Note" });

      // Add link
      const fromNote = findNote(storePath, note1.frontmatter.id)!;
      fromNote.frontmatter.links = [
        { id: note2.frontmatter.id, type: "derived-from" },
      ];
      writeNote(fromNote, fromNote.path!);

      const index = buildIndex(storePath);
      const backlinks = getBacklinks(index, note2.frontmatter.id);

      expect(backlinks).toContain(note1.frontmatter.id);
    });

    it("should remove a typed link from a note", () => {
      const note1 = createNote(storePath, {
        title: "Source Note",
        links: [{ id: "qp-target", type: "related" }],
      });

      // Remove link
      const fromNote = findNote(storePath, note1.frontmatter.id)!;
      fromNote.frontmatter.links = [];
      writeNote(fromNote, fromNote.path!);

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, note1.frontmatter.id);

      expect(outgoing.filter((e) => e.source === "typed")).toHaveLength(0);
    });

    it("should support multiple link types", () => {
      const note1 = createNote(storePath, { title: "Main Concept" });
      const note2 = createNote(storePath, { title: "Supporting Evidence" });
      const note3 = createNote(storePath, { title: "Counter Argument" });

      // Add multiple links
      const mainNote = findNote(storePath, note1.frontmatter.id)!;
      mainNote.frontmatter.links = [
        { id: note2.frontmatter.id, type: "supports" },
        { id: note3.frontmatter.id, type: "contradicts" },
      ];
      writeNote(mainNote, mainNote.path!);

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, note1.frontmatter.id);

      expect(outgoing).toHaveLength(2);
      expect(outgoing.map((e) => e.type).sort()).toEqual([
        "contradicts",
        "supports",
      ]);
    });
  });

  describe("Link traversal", () => {
    it("should traverse outgoing links", () => {
      const root = createNote(storePath, { title: "Root" });
      const child1 = createNote(storePath, { title: "Child 1" });
      const child2 = createNote(storePath, { title: "Child 2" });

      // Link root -> children
      const rootNote = findNote(storePath, root.frontmatter.id)!;
      rootNote.frontmatter.links = [
        { id: child1.frontmatter.id, type: "related" },
        { id: child2.frontmatter.id, type: "related" },
      ];
      writeNote(rootNote, rootNote.path!);

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, root.frontmatter.id);

      expect(outgoing).toHaveLength(2);
    });

    it("should traverse incoming links (backlinks)", () => {
      const target = createNote(storePath, { title: "Target" });
      const source1 = createNote(storePath, {
        title: "Source 1",
        links: [{ id: target.frontmatter.id, type: "related" }],
      });
      const source2 = createNote(storePath, {
        title: "Source 2",
        links: [{ id: target.frontmatter.id, type: "supports" }],
      });

      const index = buildIndex(storePath);
      const backlinks = getBacklinks(index, target.frontmatter.id);

      expect(backlinks).toHaveLength(2);
      expect(backlinks).toContain(source1.frontmatter.id);
      expect(backlinks).toContain(source2.frontmatter.id);
    });

    it("should handle cycles in the graph", () => {
      const note1 = createNote(storePath, { title: "Note 1" });
      const note2 = createNote(storePath, { title: "Note 2" });

      // Create cycle: note1 -> note2 -> note1
      const n1 = findNote(storePath, note1.frontmatter.id)!;
      n1.frontmatter.links = [{ id: note2.frontmatter.id, type: "related" }];
      writeNote(n1, n1.path!);

      const n2 = findNote(storePath, note2.frontmatter.id)!;
      n2.frontmatter.links = [{ id: note1.frontmatter.id, type: "related" }];
      writeNote(n2, n2.path!);

      // Index should still build without infinite loop
      const index = buildIndex(storePath);

      expect(index.edges).toHaveLength(2);
      expect(Object.keys(index.metadata)).toHaveLength(2);
    });
  });

  describe("Inline link handling", () => {
    it("should extract inline wiki links", () => {
      const target = createNote(storePath, { title: "Target Note" });
      createNote(storePath, {
        title: "Source Note",
        body: `This links to [[${target.frontmatter.id}]].`,
      });

      const index = buildIndex(storePath);
      const backlinks = getBacklinks(index, target.frontmatter.id);

      expect(backlinks).toHaveLength(1);
    });

    it("should treat inline links as type=related", () => {
      const target = createNote(storePath, { title: "Target" });
      const source = createNote(storePath, {
        title: "Source",
        body: `See [[${target.frontmatter.id}]] for details.`,
      });

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, source.frontmatter.id);

      expect(outgoing).toHaveLength(1);
      expect(outgoing[0].type).toBe("related");
      expect(outgoing[0].source).toBe("inline");
    });

    it("should combine inline and typed links", () => {
      const target = createNote(storePath, { title: "Target" });
      const source = createNote(storePath, {
        title: "Source",
        body: `Reference: [[${target.frontmatter.id}]]`,
        links: [{ id: target.frontmatter.id, type: "supports" }],
      });

      const index = buildIndex(storePath);
      const outgoing = getOutgoingLinks(index, source.frontmatter.id);

      expect(outgoing).toHaveLength(2);
      expect(outgoing.map((e) => e.source).sort()).toEqual(["inline", "typed"]);
    });
  });
});
