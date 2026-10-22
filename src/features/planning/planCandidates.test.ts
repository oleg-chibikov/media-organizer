import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";

const photo = { id: "a", kind: "photo" } as FileRecord;
const other = { id: "b", kind: "unsupported" } as FileRecord;
const ready = {
  file_id: "a",
  metadata_status: "ready",
  chosen_date: "2026-01-01"
} as MetadataResult;

describe("claimPlanCandidates", () => {
  it("claims media files once per metadata signature", () => {
    const signatures = new Map<string, string>();
    const unsupported = { ...ready, file_id: "b" };
    const first = claimPlanCandidates([photo, other], [ready, unsupported], signatures, new Map());
    expect(first.files).toEqual([photo]);
    expect(claimPlanCandidates([photo], [ready], signatures, new Map()).files).toEqual([]);
  });

  it("restores signatures after a failed request", () => {
    const signatures = new Map<string, string>();
    const claimed = claimPlanCandidates([photo], [ready], signatures, new Map());
    restorePlanSignatures(signatures, claimed.previousSignatures);
    expect(signatures.size).toBe(0);
  });

  it("claims a file again when its place folder changes", () => {
    const signatures = new Map<string, string>();
    claimPlanCandidates([photo], [ready], signatures, new Map());
    const places = new Map([["a", "Yosemite"]]);
    expect(claimPlanCandidates([photo], [ready], signatures, places).files).toEqual([photo]);
    expect(claimPlanCandidates([photo], [ready], signatures, places).files).toEqual([]);
  });
});
