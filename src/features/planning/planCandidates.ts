import type { FileRecord, MetadataResult } from "../../lib/types";

export interface PlanCandidates {
  files: FileRecord[];
  metadata: MetadataResult[];
  previousSignatures: Map<string, string | undefined>;
}

// A file is planned again only when its metadata or its place folder changes its destination.
function planningSignature(metadata: MetadataResult, placeKey: string | undefined): string {
  return `${metadata.metadata_status}:${metadata.chosen_date ?? ""}:${placeKey ?? ""}`;
}

// Mutates `signatures`: records the new signature for every file it selects.
export function claimPlanCandidates(
  rows: FileRecord[],
  metadataResults: MetadataResult[],
  signatures: Map<string, string>,
  placeKeys: Map<string, string>
): PlanCandidates {
  const rowsById = new Map(rows.map((row) => [row.id, row]));
  const candidates: PlanCandidates = { files: [], metadata: [], previousSignatures: new Map() };

  for (const metadata of metadataResults) {
    const row = rowsById.get(metadata.file_id);
    if (!row || (row.kind !== "photo" && row.kind !== "video")) {
      continue;
    }
    const nextSignature = planningSignature(metadata, placeKeys.get(row.id));
    const previousSignature = signatures.get(row.id);
    if (previousSignature === nextSignature) {
      continue;
    }
    candidates.previousSignatures.set(row.id, previousSignature);
    signatures.set(row.id, nextSignature);
    candidates.files.push(row);
    candidates.metadata.push(metadata);
  }

  return candidates;
}

// Mutates `signatures`: undoes a claim so a failed request can be planned again.
export function restorePlanSignatures(
  signatures: Map<string, string>,
  previousSignatures: Map<string, string | undefined>
): void {
  for (const [fileId, previousSignature] of previousSignatures) {
    if (previousSignature === undefined) {
      signatures.delete(fileId);
    } else {
      signatures.set(fileId, previousSignature);
    }
  }
}
