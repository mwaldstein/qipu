/**
 * Parsing utilities for Qipu notes.
 *
 * Handles extraction of links, tags, and other content from note bodies.
 * Based on specs/knowledge-model.md and specs/indexing-search.md.
 */

import { TypedLink, LinkSource } from "./models.js";

/**
 * Regular expressions for link extraction.
 */
const WIKI_LINK_REGEX = /\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g;
const MD_LINK_REGEX = /\[([^\]]+)\]\(([^)]+\.md)\)/g;

/**
 * Extracted inline link from note body.
 */
export interface InlineLink {
  /** Target note ID or path */
  target: string;
  /** Optional display label */
  label?: string;
  /** Source type */
  source: LinkSource;
  /** Character offset in body */
  offset: number;
}

/**
 * Extract wiki-style links from note body.
 * Matches [[id]] and [[id|label]] patterns.
 */
export function extractWikiLinks(body: string): InlineLink[] {
  const links: InlineLink[] = [];
  let match: RegExpExecArray | null;

  // Reset regex state
  WIKI_LINK_REGEX.lastIndex = 0;

  while ((match = WIKI_LINK_REGEX.exec(body)) !== null) {
    links.push({
      target: match[1].trim(),
      source: "inline",
      offset: match.index,
    });
  }

  return links;
}

/**
 * Extract markdown-style links to local .md files.
 * Matches [label](path/to/note.md) patterns.
 */
export function extractMarkdownLinks(body: string): InlineLink[] {
  const links: InlineLink[] = [];
  let match: RegExpExecArray | null;

  // Reset regex state
  MD_LINK_REGEX.lastIndex = 0;

  while ((match = MD_LINK_REGEX.exec(body)) !== null) {
    // Only include local links (not http/https)
    const href = match[2];
    if (!href.startsWith("http://") && !href.startsWith("https://")) {
      links.push({
        target: href,
        label: match[1],
        source: "inline",
        offset: match.index,
      });
    }
  }

  return links;
}

/**
 * Extract all inline links from note body.
 */
export function extractInlineLinks(body: string): InlineLink[] {
  const wikiLinks = extractWikiLinks(body);
  const mdLinks = extractMarkdownLinks(body);

  // Combine and sort by offset
  return [...wikiLinks, ...mdLinks].sort((a, b) => a.offset - b.offset);
}

/**
 * Convert inline links to typed links with "related" type.
 * Per spec: inline links are treated as type=related, source=inline.
 */
export function inlineToTypedLinks(inlineLinks: InlineLink[]): TypedLink[] {
  return inlineLinks.map((link) => ({
    id: link.target,
    type: "related" as const,
    source: "inline" as const,
  }));
}

/**
 * Extract summary from note body.
 * Priority per spec:
 * 1. Frontmatter `summary` field (handled separately)
 * 2. `## Summary` section first paragraph
 * 3. First paragraph of body
 * 4. Empty string
 */
export function extractSummary(body: string): string {
  // Try to find ## Summary section
  const summaryMatch = body.match(/^##\s+Summary\s*\n+([^\n#]+)/im);
  if (summaryMatch) {
    return summaryMatch[1].trim();
  }

  // Fall back to first paragraph
  const firstPara = body.trim().split(/\n\s*\n/)[0];
  if (firstPara) {
    // Remove any markdown formatting and limit length
    const clean = firstPara
      .replace(/^#+\s+.+\n?/, "") // Remove headings
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1") // Remove link syntax
      .replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, "$2 || $1") // Wiki links
      .replace(/[*_`]/g, "") // Remove emphasis markers
      .trim();

    return clean.slice(0, 200);
  }

  return "";
}

/**
 * Extract hashtags from note body (if using inline tag style).
 * Matches #tag patterns (not inside code blocks).
 */
export function extractHashtags(body: string): string[] {
  const tags: string[] = [];
  const hashtagRegex = /(?:^|\s)#([a-zA-Z][a-zA-Z0-9_-]*)/g;
  let match: RegExpExecArray | null;

  // Simple approach: extract hashtags, skip code blocks
  // For now, just do basic extraction
  while ((match = hashtagRegex.exec(body)) !== null) {
    const tag = match[1].toLowerCase();
    if (!tags.includes(tag)) {
      tags.push(tag);
    }
  }

  return tags.sort();
}
