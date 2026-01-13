/**
 * qipu index - Build/refresh derived indexes.
 *
 * Based on specs/cli-interface.md and specs/indexing-search.md.
 *
 * Why: Indexes enable fast lookups for tags, backlinks, and graph traversal.
 * Without indexes, every operation requires scanning all note files. The index
 * command ensures the cache is fresh and provides visibility into index state.
 *
 * The --rebuild flag forces a complete rebuild, useful when the cache
 * might be corrupted or out of sync with the actual notes.
 */

import { Command } from "commander";
import { ExitCodes } from "../lib/models.js";
import { resolveStore } from "../lib/storage.js";
import { getIndex, getAllTags, StoreIndex } from "../lib/indexing.js";

/**
 * Format index statistics for human output.
 */
function formatIndexStats(index: StoreIndex): string {
  const noteCount = Object.keys(index.metadata).length;
  const tagCount = Object.keys(index.tags).length;
  const edgeCount = index.edges.length;
  const backlinkCount = Object.values(index.backlinks).reduce(
    (sum, arr) => sum + arr.length,
    0,
  );

  return [
    `Notes indexed: ${noteCount}`,
    `Tags: ${tagCount}`,
    `Edges: ${edgeCount}`,
    `Backlinks: ${backlinkCount}`,
    `Built: ${index.built_at}`,
  ].join("\n");
}

export const indexCommand = new Command("index")
  .description("Build/refresh derived indexes (tags, backlinks, graph)")
  .option("--rebuild", "force complete rebuild of index")
  .action((options: Record<string, unknown>, command: Command) => {
    const globalOpts = command.parent?.opts() || {};

    // Resolve store
    const store = resolveStore({
      store: globalOpts.store as string | undefined,
      root: globalOpts.root as string | undefined,
    });

    if (!store) {
      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "error",
            error: 'No store found. Run "qipu init" first.',
          }),
        );
      } else {
        console.error('Error: No store found. Run "qipu init" first.');
      }
      process.exit(ExitCodes.DATA_ERROR);
    }

    try {
      const forceRebuild = options.rebuild === true;

      if (!globalOpts.quiet) {
        if (forceRebuild) {
          console.error("Rebuilding index...");
        } else {
          console.error("Updating index...");
        }
      }

      const index = getIndex(store.storePath, forceRebuild);

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "success",
            notes: Object.keys(index.metadata).length,
            tags: Object.keys(index.tags).length,
            edges: index.edges.length,
            backlinks: Object.values(index.backlinks).reduce(
              (sum, arr) => sum + arr.length,
              0,
            ),
            built_at: index.built_at,
            all_tags: getAllTags(index),
          }),
        );
      } else {
        console.log(formatIndexStats(index));
      }

      process.exit(ExitCodes.SUCCESS);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "error",
            error: message,
          }),
        );
      } else {
        console.error(`Error: ${message}`);
      }

      process.exit(ExitCodes.FAILURE);
    }
  });
