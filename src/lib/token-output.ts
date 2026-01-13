/**
 * Token-optimized output format for LLM context injection.
 *
 * Based on specs/token-optimized-output.md.
 *
 * Why: LLM tools have constrained context windows. Token-optimized output
 * minimizes formatting overhead while remaining human-debuggable. This format
 * is line-oriented, low-ceremony, and easy for LLMs to parse.
 *
 * Format:
 * - H: Header line (bundle metadata)
 * - N: Note metadata line
 * - S: Summary line
 * - E: Edge line
 * - B: Body lines
 */

import { Note } from "./models.js";
import { GraphEdge, NoteMetadata, StoreIndex } from "./indexing.js";
import { extractSummary } from "./parsing.js";

/** Token output format version */
export const TOKEN_VERSION = 1;

/** Default token estimation: ~4 chars per token */
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

/**
 * Header record options.
 */
export interface TokenHeaderOptions {
  store: string;
  mode: string;
  root?: string;
  direction?: string;
  maxDepth?: number;
  truncated?: boolean;
  notes?: number;
  query?: string;
}

/**
 * Format a header record.
 * Format: H qipu=1 token=1 store=<path> mode=<mode> [key=value...]
 */
export function formatHeader(options: TokenHeaderOptions): string {
  const parts = [
    "H",
    "qipu=1",
    `token=${TOKEN_VERSION}`,
    `store=${options.store}`,
    `mode=${options.mode}`,
  ];

  if (options.root) parts.push(`root=${options.root}`);
  if (options.direction) parts.push(`direction=${options.direction}`);
  if (options.maxDepth !== undefined)
    parts.push(`max_depth=${options.maxDepth}`);
  if (options.truncated !== undefined)
    parts.push(`truncated=${options.truncated}`);
  if (options.notes !== undefined) parts.push(`notes=${options.notes}`);
  if (options.query) parts.push(`query="${options.query}"`);

  return parts.join(" ");
}

/**
 * Format a note metadata record.
 * Format: N <id> <type> "<title>" tags=<tags> [path=<path>]
 */
export function formatNoteRecord(
  note: Note | NoteMetadata,
  includePath = false,
): string {
  const id = "frontmatter" in note ? note.frontmatter.id : note.id;
  const title = "frontmatter" in note ? note.frontmatter.title : note.title;
  const type =
    "frontmatter" in note ? note.frontmatter.type || "fleeting" : note.type;
  const tags = "frontmatter" in note ? note.frontmatter.tags || [] : note.tags;
  const path = "frontmatter" in note ? note.path : note.path;

  const parts = ["N", id, type, `"${title.replace(/"/g, '\\"')}"`];

  if (tags.length > 0) {
    parts.push(`tags=${tags.join(",")}`);
  }

  if (includePath && path) {
    parts.push(`path=${path}`);
  }

  return parts.join(" ");
}

/**
 * Format a summary record.
 * Format: S <id> <summary text>
 */
export function formatSummaryRecord(id: string, summary: string): string {
  // Truncate to single line, max 200 chars
  const oneLine = summary.replace(/\n/g, " ").trim();
  const truncated =
    oneLine.length > 200 ? oneLine.slice(0, 197) + "..." : oneLine;
  return `S ${id} ${truncated}`;
}

/**
 * Format an edge record.
 * Format: E <from> <type> <to> <source>
 */
export function formatEdgeRecord(edge: GraphEdge): string {
  return `E ${edge.from} ${edge.type} ${edge.to} ${edge.source}`;
}

/**
 * Format body records.
 * Format: B <id>\n<body lines>
 */
export function formatBodyRecords(id: string, body: string): string {
  const lines = [`B ${id}`];
  lines.push(body);
  return lines.join("\n");
}

/**
 * Get summary for a note (from frontmatter, ## Summary section, or first paragraph).
 */
export function getSummary(note: Note): string {
  // Priority 1: frontmatter summary
  if (note.frontmatter.summary) {
    return note.frontmatter.summary;
  }

  // Priority 2 & 3: extracted from body
  return extractSummary(note.body);
}

/**
 * Token output builder for accumulating records with budget tracking.
 */
export class TokenOutputBuilder {
  private lines: string[] = [];
  private charCount = 0;
  private maxChars?: number;
  private maxTokens?: number;
  private truncated = false;

  constructor(options: { maxChars?: number; maxTokens?: number } = {}) {
    this.maxChars = options.maxChars;
    this.maxTokens = options.maxTokens;
  }

  /**
   * Check if adding content would exceed budget.
   */
  wouldExceedBudget(content: string): boolean {
    const newChars = this.charCount + content.length + 1; // +1 for newline

    if (this.maxChars && newChars > this.maxChars) {
      return true;
    }

    if (
      this.maxTokens &&
      estimateTokens(content) + this.getTokenCount() > this.maxTokens
    ) {
      return true;
    }

    return false;
  }

  /**
   * Add a line to the output.
   * Returns false if budget exceeded and line was not added.
   */
  addLine(line: string): boolean {
    if (this.wouldExceedBudget(line)) {
      this.truncated = true;
      return false;
    }

    this.lines.push(line);
    this.charCount += line.length + 1;
    return true;
  }

  /**
   * Add multiple lines.
   */
  addLines(lines: string[]): boolean {
    const content = lines.join("\n");
    if (this.wouldExceedBudget(content)) {
      this.truncated = true;
      return false;
    }

    this.lines.push(...lines);
    this.charCount += content.length + lines.length;
    return true;
  }

  /**
   * Get current character count.
   */
  getCharCount(): number {
    return this.charCount;
  }

  /**
   * Get estimated token count.
   */
  getTokenCount(): number {
    return estimateTokens(this.lines.join("\n"));
  }

  /**
   * Check if output was truncated.
   */
  isTruncated(): boolean {
    return this.truncated;
  }

  /**
   * Build final output string.
   */
  build(): string {
    return this.lines.join("\n");
  }
}

/**
 * Format a complete context bundle in token format.
 */
export function formatTokenContext(
  notes: Note[],
  options: {
    store: string;
    withBody?: boolean;
    maxChars?: number;
    maxTokens?: number;
    query?: string;
  },
): string {
  const builder = new TokenOutputBuilder({
    maxChars: options.maxChars,
    maxTokens: options.maxTokens,
  });

  // Add notes
  for (const note of notes) {
    // Note metadata
    if (!builder.addLine(formatNoteRecord(note, true))) {
      break;
    }

    // Summary
    const summary = getSummary(note);
    if (summary) {
      if (!builder.addLine(formatSummaryRecord(note.frontmatter.id, summary))) {
        break;
      }
    }

    // Body (if requested)
    if (options.withBody && note.body) {
      if (!builder.addLine(formatBodyRecords(note.frontmatter.id, note.body))) {
        break;
      }
    }
  }

  // Prepend header with final truncation status
  const header = formatHeader({
    store: options.store,
    mode: "context",
    notes: notes.length,
    truncated: builder.isTruncated(),
    query: options.query,
  });

  return header + "\n" + builder.build();
}

/**
 * Format a traversal result in token format.
 */
export function formatTokenTraversal(
  root: string,
  nodes: NoteMetadata[],
  edges: GraphEdge[],
  options: {
    store: string;
    direction: string;
    maxDepth: number;
    truncated: boolean;
    withSummaries?: boolean;
    index?: StoreIndex;
  },
): string {
  const lines: string[] = [];

  // Header
  lines.push(
    formatHeader({
      store: options.store,
      mode: "link.tree",
      root,
      direction: options.direction,
      maxDepth: options.maxDepth,
      truncated: options.truncated,
    }),
  );

  // Nodes
  for (const node of nodes) {
    lines.push(formatNoteRecord(node, false));
  }

  // Edges
  for (const edge of edges) {
    lines.push(formatEdgeRecord(edge));
  }

  return lines.join("\n");
}

/**
 * Format a primer in token format.
 */
export function formatTokenPrimer(
  store: string,
  mocs: NoteMetadata[],
  recent: NoteMetadata[],
  commands: string[],
): string {
  const lines: string[] = [];

  // Header
  lines.push(
    formatHeader({
      store,
      mode: "prime",
      notes: mocs.length + recent.length,
    }),
  );

  // Commands reference
  lines.push("# Commands: " + commands.join(", "));

  // MOCs
  if (mocs.length > 0) {
    lines.push("# MOCs:");
    for (const moc of mocs) {
      lines.push(formatNoteRecord(moc, false));
    }
  }

  // Recent notes
  if (recent.length > 0) {
    lines.push("# Recent:");
    for (const note of recent) {
      lines.push(formatNoteRecord(note, false));
    }
  }

  return lines.join("\n");
}
