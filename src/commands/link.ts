/**
 * qipu link - Manage and traverse note links.
 *
 * Based on specs/cli-interface.md and specs/graph-traversal.md.
 *
 * Subcommands:
 * - add: Create a typed link between notes
 * - remove: Remove a link between notes
 * - list: List links for a note
 * - tree: Show traversal tree from a note
 * - path: Find path between two notes
 *
 * Why: Links are the backbone of the Zettelkasten knowledge graph. Managing
 * and traversing links enables discovery of related concepts, evidence chains,
 * and knowledge structures. The deterministic traversal is optimized for LLM
 * context building.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes, LinkType, TypedLink, Note } from "../lib/models.js";
import { resolveStore, findNote, readNote, writeNote } from "../lib/storage.js";
import {
  getIndex,
  getBacklinks,
  getOutgoingLinks,
  getIncomingLinks,
  GraphEdge,
  StoreIndex,
  NoteMetadata,
} from "../lib/indexing.js";

/** Valid link types */
const VALID_LINK_TYPES: LinkType[] = [
  "related",
  "derived-from",
  "supports",
  "contradicts",
  "part-of",
  "compacts",
];

/**
 * Add a typed link between two notes.
 */
function addLink(
  storePath: string,
  fromId: string,
  toId: string,
  linkType: LinkType,
): { from: Note; to: Note } {
  const fromNote = findNote(storePath, fromId);
  if (!fromNote) {
    throw new Error(`Note not found: ${fromId}`);
  }

  const toNote = findNote(storePath, toId);
  if (!toNote) {
    throw new Error(`Note not found: ${toId}`);
  }

  // Check if link already exists
  const existingLinks = fromNote.frontmatter.links || [];
  const existingLink = existingLinks.find(
    (l) => l.id === toNote.frontmatter.id && l.type === linkType,
  );

  if (existingLink) {
    throw new Error(`Link already exists: ${fromId} -[${linkType}]-> ${toId}`);
  }

  // Add the link
  const newLink: TypedLink = {
    type: linkType,
    id: toNote.frontmatter.id,
  };

  fromNote.frontmatter.links = [...existingLinks, newLink];
  fromNote.frontmatter.updated = new Date().toISOString();

  writeNote(fromNote, fromNote.path!);

  return { from: fromNote, to: toNote };
}

/**
 * Remove a link between two notes.
 */
function removeLink(
  storePath: string,
  fromId: string,
  toId: string,
  linkType?: LinkType,
): { from: Note; removed: number } {
  const fromNote = findNote(storePath, fromId);
  if (!fromNote) {
    throw new Error(`Note not found: ${fromId}`);
  }

  const toNote = findNote(storePath, toId);
  if (!toNote) {
    throw new Error(`Note not found: ${toId}`);
  }

  const existingLinks = fromNote.frontmatter.links || [];
  const targetId = toNote.frontmatter.id;

  // Filter out matching links
  const remaining = existingLinks.filter((l) => {
    if (l.id !== targetId) return true;
    if (linkType && l.type !== linkType) return true;
    return false;
  });

  const removed = existingLinks.length - remaining.length;

  if (removed === 0) {
    throw new Error(`No matching link found from ${fromId} to ${toId}`);
  }

  fromNote.frontmatter.links = remaining.length > 0 ? remaining : undefined;
  fromNote.frontmatter.updated = new Date().toISOString();

  writeNote(fromNote, fromNote.path!);

  return { from: fromNote, removed };
}

/**
 * BFS traversal result.
 */
interface TraversalResult {
  root: string;
  direction: "out" | "in" | "both";
  maxDepth: number;
  truncated: boolean;
  nodes: NoteMetadata[];
  edges: GraphEdge[];
  spanningTree: Array<{ parent: string; child: string; depth: number }>;
}

/**
 * Perform BFS traversal from a starting note.
 */
function traverseGraph(
  index: StoreIndex,
  startId: string,
  options: {
    direction?: "out" | "in" | "both";
    maxDepth?: number;
    maxNodes?: number;
    typedOnly?: boolean;
    inlineOnly?: boolean;
    types?: string[];
    excludeTypes?: string[];
  } = {},
): TraversalResult {
  const direction = options.direction || "both";
  const maxDepth = options.maxDepth ?? 3;
  const maxNodes = options.maxNodes ?? Infinity;

  const visited = new Set<string>();
  const nodes: NoteMetadata[] = [];
  const edges: GraphEdge[] = [];
  const spanningTree: Array<{ parent: string; child: string; depth: number }> =
    [];

  // BFS queue: [nodeId, parentId, depth]
  const queue: Array<[string, string | null, number]> = [[startId, null, 0]];
  let truncated = false;

  while (queue.length > 0) {
    const [currentId, parentId, depth] = queue.shift()!;

    if (visited.has(currentId)) continue;
    if (nodes.length >= maxNodes) {
      truncated = true;
      break;
    }

    visited.add(currentId);

    // Add node metadata
    const meta = index.metadata[currentId];
    if (meta) {
      nodes.push(meta);
    }

    // Add to spanning tree
    if (parentId !== null) {
      spanningTree.push({ parent: parentId, child: currentId, depth });
    }

    // Stop expanding at max depth
    if (depth >= maxDepth) continue;

    // Get edges based on direction
    let neighborEdges: GraphEdge[] = [];

    if (direction === "out" || direction === "both") {
      neighborEdges.push(...index.edges.filter((e) => e.from === currentId));
    }

    if (direction === "in" || direction === "both") {
      neighborEdges.push(
        ...index.edges
          .filter((e) => e.to === currentId)
          .map((e) => ({
            ...e,
            // Reverse direction for incoming edges
            from: e.to,
            to: e.from,
          })),
      );
    }

    // Apply filters
    neighborEdges = neighborEdges.filter((e) => {
      if (options.typedOnly && e.source !== "typed") return false;
      if (options.inlineOnly && e.source !== "inline") return false;
      if (options.types && options.types.length > 0) {
        if (!options.types.includes(e.type)) return false;
      }
      if (options.excludeTypes && options.excludeTypes.length > 0) {
        if (options.excludeTypes.includes(e.type)) return false;
      }
      return true;
    });

    // Sort for determinism
    neighborEdges.sort((a, b) => {
      if (a.type !== b.type) return a.type.localeCompare(b.type);
      return a.to.localeCompare(b.to);
    });

    // Add edges and queue neighbors
    for (const edge of neighborEdges) {
      // Store original edge (not reversed)
      if (
        !edges.some(
          (e) =>
            e.from === edge.from && e.to === edge.to && e.type === edge.type,
        )
      ) {
        edges.push(edge);
      }

      // Queue the neighbor (which is edge.to due to potential reversal above)
      const neighborId = edge.to;
      if (!visited.has(neighborId)) {
        queue.push([neighborId, currentId, depth + 1]);
      }
    }
  }

  return {
    root: startId,
    direction,
    maxDepth,
    truncated,
    nodes,
    edges,
    spanningTree,
  };
}

/**
 * Find shortest path between two notes using BFS.
 */
function findPath(
  index: StoreIndex,
  fromId: string,
  toId: string,
  options: {
    direction?: "out" | "in" | "both";
    maxDepth?: number;
    typedOnly?: boolean;
    inlineOnly?: boolean;
    types?: string[];
    excludeTypes?: string[];
  } = {},
): { path: string[]; edges: GraphEdge[] } | null {
  const direction = options.direction || "both";
  const maxDepth = options.maxDepth ?? 10;

  // BFS with parent tracking
  const visited = new Map<
    string,
    { parent: string | null; edge: GraphEdge | null }
  >();
  const queue: Array<[string, number]> = [[fromId, 0]];

  visited.set(fromId, { parent: null, edge: null });

  while (queue.length > 0) {
    const [currentId, depth] = queue.shift()!;

    if (currentId === toId) {
      // Reconstruct path
      const pathIds: string[] = [];
      const pathEdges: GraphEdge[] = [];
      let current: string | null = currentId;

      while (current !== null) {
        pathIds.unshift(current);
        const info = visited.get(current);
        if (info?.edge) {
          pathEdges.unshift(info.edge);
        }
        current = info?.parent || null;
      }

      return { path: pathIds, edges: pathEdges };
    }

    if (depth >= maxDepth) continue;

    // Get edges
    let neighborEdges: GraphEdge[] = [];

    if (direction === "out" || direction === "both") {
      neighborEdges.push(...index.edges.filter((e) => e.from === currentId));
    }

    if (direction === "in" || direction === "both") {
      neighborEdges.push(
        ...index.edges
          .filter((e) => e.to === currentId)
          .map((e) => ({
            from: e.to,
            to: e.from,
            type: e.type,
            source: e.source,
          })),
      );
    }

    // Apply filters
    neighborEdges = neighborEdges.filter((e) => {
      if (options.typedOnly && e.source !== "typed") return false;
      if (options.inlineOnly && e.source !== "inline") return false;
      if (options.types && !options.types.includes(e.type)) return false;
      if (options.excludeTypes && options.excludeTypes.includes(e.type))
        return false;
      return true;
    });

    for (const edge of neighborEdges) {
      const neighborId = edge.to;
      if (!visited.has(neighborId)) {
        visited.set(neighborId, { parent: currentId, edge });
        queue.push([neighborId, depth + 1]);
      }
    }
  }

  return null;
}

/**
 * Format tree for human-readable output.
 */
function formatTree(result: TraversalResult, index: StoreIndex): string {
  const lines: string[] = [];

  // Build tree structure
  const children = new Map<string, string[]>();
  for (const { parent, child } of result.spanningTree) {
    if (!children.has(parent)) {
      children.set(parent, []);
    }
    children.get(parent)!.push(child);
  }

  // Recursive tree formatter
  function formatNode(id: string, prefix: string, isLast: boolean): void {
    const meta = index.metadata[id];
    const title = meta?.title || id;
    const type = meta?.type || "unknown";

    const connector = isLast ? "└── " : "├── ";
    lines.push(`${prefix}${connector}${id} ${title} [${type}]`);

    const childIds = children.get(id) || [];
    const newPrefix = prefix + (isLast ? "    " : "│   ");

    childIds.forEach((childId, i) => {
      formatNode(childId, newPrefix, i === childIds.length - 1);
    });
  }

  // Format root
  const rootMeta = index.metadata[result.root];
  const rootTitle = rootMeta?.title || result.root;
  const rootType = rootMeta?.type || "unknown";
  lines.push(`${result.root} ${rootTitle} [${rootType}]`);

  const rootChildren = children.get(result.root) || [];
  rootChildren.forEach((childId, i) => {
    formatNode(childId, "", i === rootChildren.length - 1);
  });

  if (result.truncated) {
    lines.push("\n(truncated)");
  }

  return lines.join("\n");
}

// ============ Subcommands ============

const linkAddCommand = new Command("add")
  .description("Add a typed link between two notes")
  .argument("<from>", "source note ID or path")
  .argument("<to>", "target note ID or path")
  .requiredOption(
    "-t, --type <type>",
    `link type (${VALID_LINK_TYPES.join(", ")})`,
  )
  .action(
    (
      from: string,
      to: string,
      options: Record<string, unknown>,
      command: Command,
    ) => {
      const globalOpts = command.parent?.parent?.opts() || {};

      const store = resolveStore({
        store: globalOpts.store as string | undefined,
        root: globalOpts.root as string | undefined,
      });

      if (!store) {
        console.error('Error: No store found. Run "qipu init" first.');
        process.exit(ExitCodes.DATA_ERROR);
      }

      const linkType = options.type as string;
      if (!VALID_LINK_TYPES.includes(linkType as LinkType)) {
        console.error(
          `Error: Invalid link type "${linkType}". Valid types: ${VALID_LINK_TYPES.join(", ")}`,
        );
        process.exit(ExitCodes.USAGE_ERROR);
      }

      try {
        const { from: fromNote, to: toNote } = addLink(
          store.storePath,
          from,
          to,
          linkType as LinkType,
        );

        if (globalOpts.json) {
          console.log(
            JSON.stringify({
              status: "created",
              from: fromNote.frontmatter.id,
              to: toNote.frontmatter.id,
              type: linkType,
            }),
          );
        } else {
          console.log(
            `Added link: ${fromNote.frontmatter.id} -[${linkType}]-> ${toNote.frontmatter.id}`,
          );
        }

        process.exit(ExitCodes.SUCCESS);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error(`Error: ${message}`);
        process.exit(ExitCodes.FAILURE);
      }
    },
  );

const linkRemoveCommand = new Command("remove")
  .description("Remove a link between two notes")
  .argument("<from>", "source note ID or path")
  .argument("<to>", "target note ID or path")
  .option("-t, --type <type>", "link type (removes all links if not specified)")
  .action(
    (
      from: string,
      to: string,
      options: Record<string, unknown>,
      command: Command,
    ) => {
      const globalOpts = command.parent?.parent?.opts() || {};

      const store = resolveStore({
        store: globalOpts.store as string | undefined,
        root: globalOpts.root as string | undefined,
      });

      if (!store) {
        console.error('Error: No store found. Run "qipu init" first.');
        process.exit(ExitCodes.DATA_ERROR);
      }

      try {
        const { from: fromNote, removed } = removeLink(
          store.storePath,
          from,
          to,
          options.type as LinkType | undefined,
        );

        if (globalOpts.json) {
          console.log(
            JSON.stringify({
              status: "removed",
              from: fromNote.frontmatter.id,
              to,
              removed,
            }),
          );
        } else {
          console.log(
            `Removed ${removed} link(s) from ${fromNote.frontmatter.id} to ${to}`,
          );
        }

        process.exit(ExitCodes.SUCCESS);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error(`Error: ${message}`);
        process.exit(ExitCodes.FAILURE);
      }
    },
  );

const linkListCommand = new Command("list")
  .description("List links for a note")
  .argument("<id>", "note ID or path")
  .option(
    "-d, --direction <dir>",
    "direction: out, in, both (default: both)",
    "both",
  )
  .option("--typed-only", "show only typed links")
  .option("--inline-only", "show only inline links")
  .option("-t, --type <type>", "filter by link type")
  .action((id: string, options: Record<string, unknown>, command: Command) => {
    const globalOpts = command.parent?.parent?.opts() || {};

    const store = resolveStore({
      store: globalOpts.store as string | undefined,
      root: globalOpts.root as string | undefined,
    });

    if (!store) {
      console.error('Error: No store found. Run "qipu init" first.');
      process.exit(ExitCodes.DATA_ERROR);
    }

    try {
      const index = getIndex(store.storePath);
      const note = findNote(store.storePath, id);

      if (!note) {
        console.error(`Error: Note not found: ${id}`);
        process.exit(ExitCodes.DATA_ERROR);
      }

      const noteId = note.frontmatter.id;
      const direction = options.direction as string;

      let outgoing: GraphEdge[] = [];
      let incoming: GraphEdge[] = [];

      if (direction === "out" || direction === "both") {
        outgoing = getOutgoingLinks(index, noteId);
      }

      if (direction === "in" || direction === "both") {
        incoming = getIncomingLinks(index, noteId);
      }

      // Apply filters
      const filterEdges = (edges: GraphEdge[]): GraphEdge[] => {
        return edges.filter((e) => {
          if (options.typedOnly && e.source !== "typed") return false;
          if (options.inlineOnly && e.source !== "inline") return false;
          if (options.type && e.type !== options.type) return false;
          return true;
        });
      };

      outgoing = filterEdges(outgoing);
      incoming = filterEdges(incoming);

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            id: noteId,
            outgoing: outgoing.map((e) => ({
              to: e.to,
              type: e.type,
              source: e.source,
            })),
            incoming: incoming.map((e) => ({
              from: e.from,
              type: e.type,
              source: e.source,
            })),
          }),
        );
      } else {
        if (outgoing.length > 0) {
          console.log("Outgoing:");
          for (const edge of outgoing) {
            const target = index.metadata[edge.to];
            const title = target?.title || edge.to;
            console.log(
              `  -> ${edge.to} ${title} [${edge.type}] (${edge.source})`,
            );
          }
        }

        if (incoming.length > 0) {
          console.log("Incoming:");
          for (const edge of incoming) {
            const source = index.metadata[edge.from];
            const title = source?.title || edge.from;
            console.log(
              `  <- ${edge.from} ${title} [${edge.type}] (${edge.source})`,
            );
          }
        }

        if (outgoing.length === 0 && incoming.length === 0) {
          console.log("No links found");
        }
      }

      process.exit(ExitCodes.SUCCESS);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error(`Error: ${message}`);
      process.exit(ExitCodes.FAILURE);
    }
  });

const linkTreeCommand = new Command("tree")
  .description("Show traversal tree from a note")
  .argument("<id>", "starting note ID or path")
  .option(
    "-d, --direction <dir>",
    "direction: out, in, both (default: both)",
    "both",
  )
  .option("--max-depth <n>", "maximum traversal depth (default: 3)", "3")
  .option("--max-nodes <n>", "maximum nodes to visit")
  .option("--typed-only", "traverse only typed links")
  .option("--inline-only", "traverse only inline links")
  .option("-t, --type <type>", "include only this link type (repeatable)")
  .option("--exclude-type <type>", "exclude this link type (repeatable)")
  .action((id: string, options: Record<string, unknown>, command: Command) => {
    const globalOpts = command.parent?.parent?.opts() || {};

    const store = resolveStore({
      store: globalOpts.store as string | undefined,
      root: globalOpts.root as string | undefined,
    });

    if (!store) {
      console.error('Error: No store found. Run "qipu init" first.');
      process.exit(ExitCodes.DATA_ERROR);
    }

    try {
      const index = getIndex(store.storePath);
      const note = findNote(store.storePath, id);

      if (!note) {
        console.error(`Error: Note not found: ${id}`);
        process.exit(ExitCodes.DATA_ERROR);
      }

      const result = traverseGraph(index, note.frontmatter.id, {
        direction: options.direction as "out" | "in" | "both",
        maxDepth: parseInt(options.maxDepth as string, 10) || 3,
        maxNodes: options.maxNodes
          ? parseInt(options.maxNodes as string, 10)
          : undefined,
        typedOnly: options.typedOnly === true,
        inlineOnly: options.inlineOnly === true,
        types: options.type
          ? Array.isArray(options.type)
            ? options.type
            : [options.type]
          : undefined,
        excludeTypes: options.excludeType
          ? Array.isArray(options.excludeType)
            ? options.excludeType
            : [options.excludeType]
          : undefined,
      });

      if (globalOpts.json) {
        console.log(JSON.stringify(result, null, 2));
      } else {
        console.log(formatTree(result, index));
      }

      process.exit(ExitCodes.SUCCESS);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error(`Error: ${message}`);
      process.exit(ExitCodes.FAILURE);
    }
  });

const linkPathCommand = new Command("path")
  .description("Find path between two notes")
  .argument("<from>", "starting note ID or path")
  .argument("<to>", "target note ID or path")
  .option(
    "-d, --direction <dir>",
    "direction: out, in, both (default: both)",
    "both",
  )
  .option("--max-depth <n>", "maximum search depth (default: 10)", "10")
  .option("--typed-only", "use only typed links")
  .option("--inline-only", "use only inline links")
  .action(
    (
      from: string,
      to: string,
      options: Record<string, unknown>,
      command: Command,
    ) => {
      const globalOpts = command.parent?.parent?.opts() || {};

      const store = resolveStore({
        store: globalOpts.store as string | undefined,
        root: globalOpts.root as string | undefined,
      });

      if (!store) {
        console.error('Error: No store found. Run "qipu init" first.');
        process.exit(ExitCodes.DATA_ERROR);
      }

      try {
        const index = getIndex(store.storePath);

        const fromNote = findNote(store.storePath, from);
        if (!fromNote) {
          console.error(`Error: Note not found: ${from}`);
          process.exit(ExitCodes.DATA_ERROR);
        }

        const toNote = findNote(store.storePath, to);
        if (!toNote) {
          console.error(`Error: Note not found: ${to}`);
          process.exit(ExitCodes.DATA_ERROR);
        }

        const result = findPath(
          index,
          fromNote.frontmatter.id,
          toNote.frontmatter.id,
          {
            direction: options.direction as "out" | "in" | "both",
            maxDepth: parseInt(options.maxDepth as string, 10) || 10,
            typedOnly: options.typedOnly === true,
            inlineOnly: options.inlineOnly === true,
          },
        );

        if (!result) {
          if (globalOpts.json) {
            console.log(
              JSON.stringify({
                from: fromNote.frontmatter.id,
                to: toNote.frontmatter.id,
                found: false,
                path: null,
              }),
            );
          } else {
            console.log(
              `No path found from ${fromNote.frontmatter.id} to ${toNote.frontmatter.id}`,
            );
          }
          process.exit(ExitCodes.SUCCESS);
        }

        if (globalOpts.json) {
          console.log(
            JSON.stringify({
              from: fromNote.frontmatter.id,
              to: toNote.frontmatter.id,
              found: true,
              length: result.path.length - 1,
              path: result.path.map((id) => {
                const meta = index.metadata[id];
                return {
                  id,
                  title: meta?.title || id,
                  type: meta?.type || "unknown",
                };
              }),
              edges: result.edges,
            }),
          );
        } else {
          console.log(
            `Path found (${result.path.length - 1} hop${result.path.length - 1 !== 1 ? "s" : ""}):\n`,
          );

          for (let i = 0; i < result.path.length; i++) {
            const id = result.path[i];
            const meta = index.metadata[id];
            const title = meta?.title || id;

            if (i > 0) {
              const edge = result.edges[i - 1];
              console.log(`    |`);
              console.log(`    | [${edge.type}] (${edge.source})`);
              console.log(`    v`);
            }

            console.log(`${id} ${title}`);
          }
        }

        process.exit(ExitCodes.SUCCESS);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error(`Error: ${message}`);
        process.exit(ExitCodes.FAILURE);
      }
    },
  );

// ============ Main link command ============

export const linkCommand = new Command("link")
  .description("Manage and traverse note links")
  .addCommand(linkAddCommand)
  .addCommand(linkRemoveCommand)
  .addCommand(linkListCommand)
  .addCommand(linkTreeCommand)
  .addCommand(linkPathCommand);
