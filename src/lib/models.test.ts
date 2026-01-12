/**
 * Tests for core data models.
 */

import { describe, it, expect } from "vitest";
import {
  ExitCodes,
  DEFAULT_CONFIG,
  type NoteType,
  type LinkType,
} from "./models.js";

describe("ExitCodes", () => {
  it("should define correct exit codes per spec", () => {
    expect(ExitCodes.SUCCESS).toBe(0);
    expect(ExitCodes.FAILURE).toBe(1);
    expect(ExitCodes.USAGE_ERROR).toBe(2);
    expect(ExitCodes.DATA_ERROR).toBe(3);
  });
});

describe("DEFAULT_CONFIG", () => {
  it("should have sensible defaults", () => {
    expect(DEFAULT_CONFIG.format_version).toBe(1);
    expect(DEFAULT_CONFIG.id_scheme).toBe("hash");
    expect(DEFAULT_CONFIG.default_note_type).toBe("fleeting");
    expect(DEFAULT_CONFIG.editor).toBe("");
  });
});

describe("NoteType", () => {
  it("should accept valid note types", () => {
    const types: NoteType[] = ["fleeting", "literature", "permanent", "moc"];
    expect(types).toHaveLength(4);
  });
});

describe("LinkType", () => {
  it("should accept valid link types", () => {
    const types: LinkType[] = [
      "related",
      "derived-from",
      "supports",
      "contradicts",
      "part-of",
      "compacts",
    ];
    expect(types).toHaveLength(6);
  });
});
