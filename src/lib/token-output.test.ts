/**
 * Tests for token-optimized output format.
 *
 * Why: Token-optimized output is critical for LLM context efficiency.
 * These tests verify:
 * - Record formatting follows spec (H, N, S, E, B prefixes)
 * - Budget tracking works correctly
 * - Summary extraction prioritizes correctly
 * - Output is deterministic
 */

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { initStore, createNote } from "./storage.js";
import {
  formatHeader,
  formatNoteRecord,
  formatSummaryRecord,
  formatEdgeRecord,
  formatBodyRecords,
  formatTokenContext,
  estimateTokens,
  getSummary,
  TokenOutputBuilder,
  TOKEN_VERSION,
} from "./token-output.js";
import { GraphEdge } from "./indexing.js";

describe("Token Output Format", () => {
  describe("formatHeader", () => {
    it("should format basic header", () => {
      const header = formatHeader({
        store: ".qipu",
        mode: "context",
      });

      expect(header).toContain("H");
      expect(header).toContain("qipu=1");
      expect(header).toContain(`token=${TOKEN_VERSION}`);
      expect(header).toContain("store=.qipu");
      expect(header).toContain("mode=context");
    });

    it("should include optional fields when provided", () => {
      const header = formatHeader({
        store: ".qipu",
        mode: "link.tree",
        root: "qp-abc",
        direction: "both",
        maxDepth: 3,
        truncated: true,
      });

      expect(header).toContain("root=qp-abc");
      expect(header).toContain("direction=both");
      expect(header).toContain("max_depth=3");
      expect(header).toContain("truncated=true");
    });
  });

  describe("formatNoteRecord", () => {
    it("should format note metadata", () => {
      const record = formatNoteRecord({
        id: "qp-test",
        title: "Test Note",
        type: "permanent",
        tags: ["tag1", "tag2"],
        path: "",
        created: "",
        updated: "",
      });

      expect(record).toBe('N qp-test permanent "Test Note" tags=tag1,tag2');
    });

    it("should escape quotes in title", () => {
      const record = formatNoteRecord({
        id: "qp-test",
        title: 'Note with "quotes"',
        type: "fleeting",
        tags: [],
        path: "",
        created: "",
        updated: "",
      });

      expect(record).toContain('\\"quotes\\"');
    });

    it("should include path when requested", () => {
      const record = formatNoteRecord(
        {
          id: "qp-test",
          title: "Test",
          type: "fleeting",
          tags: [],
          path: ".qipu/notes/test.md",
          created: "",
          updated: "",
        },
        true,
      );

      expect(record).toContain("path=.qipu/notes/test.md");
    });
  });

  describe("formatSummaryRecord", () => {
    it("should format summary on single line", () => {
      const record = formatSummaryRecord(
        "qp-test",
        "This is a summary.\nWith multiple lines.",
      );

      expect(record).toBe("S qp-test This is a summary. With multiple lines.");
    });

    it("should truncate long summaries", () => {
      const longText = "x".repeat(250);
      const record = formatSummaryRecord("qp-test", longText);

      expect(record.length).toBeLessThanOrEqual(200 + 10); // id + prefix
      expect(record).toContain("...");
    });
  });

  describe("formatEdgeRecord", () => {
    it("should format edge", () => {
      const edge: GraphEdge = {
        from: "qp-a",
        to: "qp-b",
        type: "supports",
        source: "typed",
      };

      const record = formatEdgeRecord(edge);
      expect(record).toBe("E qp-a supports qp-b typed");
    });
  });

  describe("formatBodyRecords", () => {
    it("should format body with prefix", () => {
      const body = "Line 1\nLine 2";
      const records = formatBodyRecords("qp-test", body);

      expect(records).toContain("B qp-test");
      expect(records).toContain("Line 1");
      expect(records).toContain("Line 2");
    });
  });

  describe("estimateTokens", () => {
    it("should estimate ~4 chars per token", () => {
      const text = "a".repeat(100);
      expect(estimateTokens(text)).toBe(25);
    });

    it("should round up", () => {
      const text = "abc";
      expect(estimateTokens(text)).toBe(1);
    });
  });

  describe("TokenOutputBuilder", () => {
    it("should track character count", () => {
      const builder = new TokenOutputBuilder();
      builder.addLine("Hello World");

      expect(builder.getCharCount()).toBe(12); // 11 + newline
    });

    it("should respect maxChars budget", () => {
      const builder = new TokenOutputBuilder({ maxChars: 20 });

      builder.addLine("Short");
      const added = builder.addLine("This is a much longer line");

      expect(added).toBe(false);
      expect(builder.isTruncated()).toBe(true);
    });

    it("should respect maxTokens budget", () => {
      const builder = new TokenOutputBuilder({ maxTokens: 5 });

      builder.addLine("Short");
      const added = builder.addLine("This exceeds the token budget");

      expect(added).toBe(false);
      expect(builder.isTruncated()).toBe(true);
    });

    it("should build output", () => {
      const builder = new TokenOutputBuilder();
      builder.addLine("Line 1");
      builder.addLine("Line 2");

      expect(builder.build()).toBe("Line 1\nLine 2");
    });
  });

  describe("getSummary", () => {
    let tempDir: string;
    let storePath: string;

    beforeEach(() => {
      tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-token-test-"));
      const result = initStore(tempDir);
      storePath = result.storePath;
    });

    afterEach(() => {
      fs.rmSync(tempDir, { recursive: true, force: true });
    });

    it("should extract from ## Summary section", () => {
      const note = createNote(storePath, {
        title: "Test",
        body: "# Title\n\n## Summary\n\nThis is the summary.\n\n## Details\n\nMore content.",
      });

      expect(getSummary(note)).toBe("This is the summary.");
    });

    it("should fall back to first paragraph", () => {
      const note = createNote(storePath, {
        title: "Test",
        body: "First paragraph here.\n\nSecond paragraph.",
      });

      expect(getSummary(note)).toBe("First paragraph here.");
    });

    it("should use frontmatter summary if present", () => {
      const note = createNote(storePath, {
        title: "Test",
        body: "Some content.",
      });
      note.frontmatter.summary = "Frontmatter summary";

      expect(getSummary(note)).toBe("Frontmatter summary");
    });
  });

  describe("formatTokenContext", () => {
    let tempDir: string;
    let storePath: string;

    beforeEach(() => {
      tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "qipu-context-test-"));
      const result = initStore(tempDir);
      storePath = result.storePath;
    });

    afterEach(() => {
      fs.rmSync(tempDir, { recursive: true, force: true });
    });

    it("should format context with header and notes", () => {
      const note = createNote(storePath, {
        title: "Test Note",
        body: "Content here.",
      });

      const output = formatTokenContext([note], {
        store: ".qipu",
      });

      expect(output).toContain("H qipu=1");
      expect(output).toContain("mode=context");
      expect(output).toContain("N qp-");
      expect(output).toContain('"Test Note"');
      expect(output).toContain("S qp-");
    });

    it("should include body with --with-body", () => {
      const note = createNote(storePath, {
        title: "Test",
        body: "Body content here.",
      });

      const output = formatTokenContext([note], {
        store: ".qipu",
        withBody: true,
      });

      expect(output).toContain("B qp-");
      expect(output).toContain("Body content here.");
    });
  });
});
