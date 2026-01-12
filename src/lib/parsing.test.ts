/**
 * Tests for parsing utilities.
 */

import { describe, it, expect } from "vitest";
import {
  extractWikiLinks,
  extractMarkdownLinks,
  extractInlineLinks,
  inlineToTypedLinks,
  extractSummary,
  extractHashtags,
} from "./parsing.js";

describe("Link Extraction", () => {
  describe("extractWikiLinks", () => {
    it("should extract simple wiki links", () => {
      const body = "See [[qp-abc1]] and [[qp-def2]] for more.";
      const links = extractWikiLinks(body);

      expect(links).toHaveLength(2);
      expect(links[0].target).toBe("qp-abc1");
      expect(links[1].target).toBe("qp-def2");
    });

    it("should extract wiki links with labels", () => {
      const body = "See [[qp-abc1|my note]] for details.";
      const links = extractWikiLinks(body);

      expect(links).toHaveLength(1);
      expect(links[0].target).toBe("qp-abc1");
    });

    it("should handle empty body", () => {
      const links = extractWikiLinks("");
      expect(links).toHaveLength(0);
    });

    it("should track link offsets", () => {
      const body = "Start [[qp-abc1]] middle [[qp-def2]] end";
      const links = extractWikiLinks(body);

      expect(links[0].offset).toBe(6);
      expect(links[1].offset).toBe(25);
    });
  });

  describe("extractMarkdownLinks", () => {
    it("should extract markdown links to .md files", () => {
      const body =
        "See [note one](notes/note-1.md) and [note two](mocs/topic.md).";
      const links = extractMarkdownLinks(body);

      expect(links).toHaveLength(2);
      expect(links[0].target).toBe("notes/note-1.md");
      expect(links[0].label).toBe("note one");
      expect(links[1].target).toBe("mocs/topic.md");
    });

    it("should ignore external URLs", () => {
      const body =
        "See [Google](https://google.com) and [local](notes/local.md).";
      const links = extractMarkdownLinks(body);

      expect(links).toHaveLength(1);
      expect(links[0].target).toBe("notes/local.md");
    });

    it("should ignore non-.md links", () => {
      const body = "See [image](images/photo.png) and [note](notes/note.md).";
      const links = extractMarkdownLinks(body);

      expect(links).toHaveLength(1);
      expect(links[0].target).toBe("notes/note.md");
    });
  });

  describe("extractInlineLinks", () => {
    it("should combine wiki and markdown links", () => {
      const body = "Wiki: [[qp-abc1]], MD: [note](notes/test.md)";
      const links = extractInlineLinks(body);

      expect(links).toHaveLength(2);
      expect(links[0].target).toBe("qp-abc1");
      expect(links[1].target).toBe("notes/test.md");
    });

    it("should sort links by offset", () => {
      const body = "First [md](test.md) then [[wiki]]";
      const links = extractInlineLinks(body);

      expect(links[0].offset).toBeLessThan(links[1].offset);
    });
  });

  describe("inlineToTypedLinks", () => {
    it("should convert inline links to typed links with related type", () => {
      const inlineLinks = [
        { target: "qp-abc1", source: "inline" as const, offset: 0 },
        { target: "notes/test.md", source: "inline" as const, offset: 20 },
      ];

      const typed = inlineToTypedLinks(inlineLinks);

      expect(typed).toHaveLength(2);
      expect(typed[0].id).toBe("qp-abc1");
      expect(typed[0].type).toBe("related");
      expect(typed[0].source).toBe("inline");
    });
  });
});

describe("Summary Extraction", () => {
  describe("extractSummary", () => {
    it("should extract from ## Summary section", () => {
      const body = `# Title

Some intro text.

## Summary

This is the summary paragraph.

## Other Section

More content.`;

      const summary = extractSummary(body);
      expect(summary).toBe("This is the summary paragraph.");
    });

    it("should fall back to first paragraph", () => {
      const body = `This is the first paragraph that serves as summary.

This is the second paragraph with more details.`;

      const summary = extractSummary(body);
      expect(summary).toBe(
        "This is the first paragraph that serves as summary.",
      );
    });

    it("should remove markdown formatting", () => {
      const body = "This has **bold** and [[links]] in it.";
      const summary = extractSummary(body);

      expect(summary).not.toContain("**");
      expect(summary).not.toContain("[[");
    });

    it("should limit summary length", () => {
      const body = "A".repeat(500);
      const summary = extractSummary(body);

      expect(summary.length).toBeLessThanOrEqual(200);
    });

    it("should handle empty body", () => {
      const summary = extractSummary("");
      expect(summary).toBe("");
    });
  });
});

describe("Hashtag Extraction", () => {
  describe("extractHashtags", () => {
    it("should extract hashtags from body", () => {
      const body = "This note is about #typescript and #testing.";
      const tags = extractHashtags(body);

      expect(tags).toContain("typescript");
      expect(tags).toContain("testing");
    });

    it("should lowercase tags", () => {
      const body = "Using #TypeScript today.";
      const tags = extractHashtags(body);

      expect(tags).toContain("typescript");
      expect(tags).not.toContain("TypeScript");
    });

    it("should deduplicate tags", () => {
      const body = "About #test and more #test content.";
      const tags = extractHashtags(body);

      expect(tags.filter((t) => t === "test")).toHaveLength(1);
    });

    it("should sort tags alphabetically", () => {
      const body = "#zebra #apple #mango";
      const tags = extractHashtags(body);

      expect(tags).toEqual(["apple", "mango", "zebra"]);
    });

    it("should handle empty body", () => {
      const tags = extractHashtags("");
      expect(tags).toHaveLength(0);
    });
  });
});
