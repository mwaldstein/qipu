#!/usr/bin/env node
/**
 * Qipu CLI entry point.
 *
 * Zettelkasten-inspired knowledge management for LLM workflows.
 * Based on specs/cli-interface.md.
 */

import { Command } from "commander";
import { ExitCodes } from "./lib/models.js";
import { initCommand } from "./commands/init.js";
import { createCommand } from "./commands/create.js";
import { listCommand } from "./commands/list.js";
import { showCommand } from "./commands/show.js";

const VERSION = "0.0.0";

const program = new Command();

program
  .name("qipu")
  .description("Zettelkasten-inspired knowledge management for LLM workflows")
  .version(VERSION, "-V, --version", "output the version number")
  .option("--store <path>", "path to store directory")
  .option("--root <path>", "root directory for store discovery")
  .option("--json", "output in JSON format")
  .option("--token", "output in token-optimized format")
  .option("-q, --quiet", "suppress non-essential output")
  .option("-v, --verbose", "show detailed output")
  .hook("preAction", (thisCommand) => {
    const opts = thisCommand.opts();
    // Validate mutually exclusive options
    if (opts.json && opts.token) {
      console.error("Error: --json and --token are mutually exclusive");
      process.exit(ExitCodes.USAGE_ERROR);
    }
  });

// Register commands
program.addCommand(initCommand);
program.addCommand(createCommand);
program.addCommand(listCommand);
program.addCommand(showCommand);

// Handle unknown commands
program.on("command:*", () => {
  console.error(`Error: Unknown command '${program.args[0]}'`);
  console.error('Run "qipu --help" for available commands.');
  process.exit(ExitCodes.USAGE_ERROR);
});

// Parse and execute
program.parseAsync(process.argv).catch((err: Error) => {
  if (process.env.DEBUG) {
    console.error(err);
  } else {
    console.error(`Error: ${err.message}`);
  }
  process.exit(ExitCodes.FAILURE);
});
