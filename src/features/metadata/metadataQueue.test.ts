import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult } from "../../lib/types";
import {
  createMetadataQueue,
  enqueueRowsNeedingMetadata,
  takeMetadataBatch
} from "./metadataQueue";

function row(id: string, kind: FileRecord["kind"] = "photo"): FileRecord {
  return {
    id,
    source_path: `/in/${id}`,
    relative_path: id,
    extension: "jpg",
    original_name: id,
    kind,
    scan_status: "discovered"
  };
}

describe("metadataQueue", () => {
  it("queues only media rows without metadata, up to capacity", () => {
    const queue = createMetadataQueue();
    const known = { b: { file_id: "b" } as MetadataResult };
    enqueueRowsNeedingMetadata(
      queue,
      [row("a"), row("b"), row("c", "unsupported"), row("d"), row("e")],
      known,
      2
    );
    expect(queue.pending).toEqual(["a", "d"]);
  });

  it("takes a batch of known rows and marks them requested", () => {
    const queue = createMetadataQueue();
    queue.pending.push("a", "missing", "b");
    queue.queuedIds.add("a").add("missing").add("b");
    const batch = takeMetadataBatch(
      queue,
      new Map([
        ["a", row("a")],
        ["b", row("b")]
      ])
    );
    expect(batch.map((item) => item.id)).toEqual(["a", "b"]);
    expect([...queue.requestedIds]).toEqual(["a", "b"]);
    expect(queue.queuedIds.size).toBe(0);
  });
});
