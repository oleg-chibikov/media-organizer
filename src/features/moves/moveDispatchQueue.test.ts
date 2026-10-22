import { describe, expect, it } from "vitest";
import { FileIdMap } from "../../lib/fileIdMap";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import {
  finishBatch,
  isDispatchDrained,
  recordMoveResult,
  releaseFailedBatch
} from "./moveDispatchCompletion";
import { createMoveDispatchQueue, enqueueReadyPlans, takeMoveBatch } from "./moveDispatchQueue";

const plan = (fileId: string, status: MovePlan["plan_status"] = "ready") =>
  ({ file_id: fileId, plan_status: status }) as MovePlan;
const PLANS = FileIdMap.from({ a: plan("a"), b: plan("b"), c: plan("c", "blocked_no_date") });
const ROWS = new Map([["a", { id: "a" } as FileRecord]]);
const METADATA = FileIdMap.from({
  a: { file_id: "a" } as MetadataResult,
  b: { file_id: "b" } as MetadataResult
});

describe("moveDispatchQueue", () => {
  it("queues ready plans once and skips files that already have a result", () => {
    const queue = createMoveDispatchQueue();
    enqueueReadyPlans(queue, PLANS, FileIdMap.from({ b: {} as MoveResult }));
    enqueueReadyPlans(queue, PLANS, FileIdMap.empty());
    expect(queue.pending).toEqual(["a"]);
    expect([...queue.completedIds]).toEqual(["b"]);
  });

  it("batches dispatchable files and drops the ones it cannot send", () => {
    const queue = createMoveDispatchQueue();
    enqueueReadyPlans(queue, PLANS, FileIdMap.empty());
    const batch = takeMoveBatch(queue, ROWS, METADATA, PLANS);
    expect(batch.fileIds).toEqual(["a"]);
    expect(queue.activeRequests).toBe(1);
    expect(queue.trackedIds.has("b")).toBe(false);
  });

  it("frees failed batches and counts each result once", () => {
    const queue = createMoveDispatchQueue();
    enqueueReadyPlans(queue, PLANS, FileIdMap.empty());
    const batch = takeMoveBatch(queue, ROWS, METADATA, PLANS);
    const failure = releaseFailedBatch(queue, batch.fileIds, "offline");
    expect(failure).toMatchObject({
      statusMessage: "Move dispatch failed: offline",
      completed: true
    });
    expect(queue.activeRequests).toBe(0);
    expect(queue.inflightIds.size).toBe(0);
    expect(recordMoveResult(queue, "a")).toBe(true);
    expect(recordMoveResult(queue, "a")).toBe(false);
  });

  it("queues the files a paused run didn't reach again", () => {
    const queue = createMoveDispatchQueue();
    const rows = new Map([...ROWS, ["b", { id: "b" } as FileRecord]]);
    enqueueReadyPlans(queue, PLANS, FileIdMap.empty());
    takeMoveBatch(queue, rows, METADATA, PLANS);
    recordMoveResult(queue, "a");
    finishBatch(queue);
    expect(queue.activeRequests).toBe(0);
    expect(queue.pending).toEqual(["b"]);
    expect(isDispatchDrained(queue)).toBe(false);
    expect(takeMoveBatch(queue, rows, METADATA, PLANS).fileIds).toEqual(["b"]);
  });

  it("queues a paused run of hundreds of thousands of files again", () => {
    const queue = createMoveDispatchQueue();
    const fileIds = Array.from({ length: 300_000 }, (_, index) => `f${index}`);
    fileIds.forEach((fileId) => queue.inflightIds.add(fileId));
    queue.pending = ["next"];
    finishBatch(queue);
    expect(queue.pending).toHaveLength(300_001);
    expect(queue.pending[queue.pending.length - 1]).toBe("next");
  });

  it("keeps a run open while files are still being checked", () => {
    const queue = createMoveDispatchQueue();
    queue.awaitingPlans = true;
    expect(isDispatchDrained(queue)).toBe(false);
    enqueueReadyPlans(queue, PLANS, FileIdMap.empty());
    const batch = takeMoveBatch(queue, ROWS, METADATA, PLANS);
    expect(releaseFailedBatch(queue, batch.fileIds, "offline")).not.toHaveProperty("completed");
    queue.awaitingPlans = false;
    expect(isDispatchDrained(queue)).toBe(true);
  });
});
