/**
 * qipu init - Initialize a new store.
 *
 * Based on specs/cli-interface.md.
 */

import { Command } from "commander";
import * as path from "node:path";
import { ExitCodes } from "../lib/models.js";
import { initStore, resolveStore } from "../lib/storage.js";

export const initCommand = new Command("init")
  .description("Create a new Qipu store")
  .option("--stealth", "add store to .gitignore (local-only mode)")
  .option("--visible", "use qipu/ instead of .qipu/ (visible in file listings)")
  .option("--branch <name>", "configure protected-branch workflow (future)")
  .action((options, command) => {
    const globalOpts = command.parent?.opts() || {};
    const rootPath = globalOpts.root
      ? path.resolve(globalOpts.root)
      : process.cwd();

    // Check if store already exists
    const existing = resolveStore({ root: rootPath });
    if (existing) {
      if (!globalOpts.quiet) {
        console.log(`Store already exists at ${existing.storePath}`);
      }

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "exists",
            path: existing.storePath,
            root: existing.rootPath,
          }),
        );
      }

      process.exit(ExitCodes.SUCCESS);
    }

    try {
      const { storePath, rootPath: storeRoot } = initStore(rootPath, {
        stealth: options.stealth,
        visible: options.visible,
      });

      if (globalOpts.json) {
        console.log(
          JSON.stringify({
            status: "created",
            path: storePath,
            root: storeRoot,
            stealth: options.stealth || false,
            visible: options.visible || false,
          }),
        );
      } else if (!globalOpts.quiet) {
        console.log(`Initialized Qipu store at ${storePath}`);
        if (options.stealth) {
          console.log("Store added to .gitignore (stealth mode)");
        }
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
