import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import type { FileRecord, MetadataResult, MovePlan } from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
}

const STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Discovered",
  skipped_processed: "Skipped Processed",
  skipped_unsupported: "Skipped Unsupported",
  scan_error: "Scan Error"
};

export function FileTable({ rows, metadataByFileId, movePlansByFileId }: FileTableProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20
  });

  return (
    <div className="flex h-full flex-col">
      <div className="grid grid-cols-[14rem_minmax(0,1fr)_8rem_12rem_11rem_minmax(0,1fr)_14rem] border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400">
        <div className="px-3 py-2">Status</div>
        <div className="px-3 py-2">Source File</div>
        <div className="px-3 py-2">Kind</div>
        <div className="px-3 py-2">Original Name</div>
        <div className="px-3 py-2">Metadata Date</div>
        <div className="px-3 py-2">Destination</div>
        <div className="px-3 py-2">Issue</div>
      </div>

      <div ref={parentRef} className="min-h-0 flex-1 overflow-auto">
        <div
          className="relative w-full"
          style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            const isSkippedUnsupported = row.scan_status === "skipped_unsupported";
            const metadata = metadataByFileId[row.id];
            const movePlan = movePlansByFileId[row.id];
            const metadataDateLabel = metadata
              ? metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date"
              : "pending";
            const destinationLabel =
              movePlan?.destination_path ||
              (metadata?.metadata_status === "ready" ? "planning..." : "-");

            const statusLabel = resolveStatusLabel(row, metadata, movePlan);
            const issueLabel = resolveIssueLabel(metadata, movePlan);
            const isFallbackWarning = metadata?.date_kind === "fallback";

            return (
              <div
                key={row.id}
                className={`absolute left-0 top-0 grid w-full grid-cols-[14rem_minmax(0,1fr)_8rem_12rem_11rem_minmax(0,1fr)_14rem] border-t border-slate-800 text-sm ${isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`}
                style={{
                  transform: `translateY(${virtualRow.start}px)`,
                  height: `${virtualRow.size}px`
                }}
              >
                <div className="truncate px-3 py-2">{statusLabel}</div>
                <div className="truncate px-3 py-2 font-mono text-xs">{row.relative_path}</div>
                <div className="truncate px-3 py-2">{row.kind}</div>
                <div className="truncate px-3 py-2">{row.original_name}</div>
                <div className="truncate px-3 py-2">{metadataDateLabel}</div>
                <div className="truncate px-3 py-2 font-mono text-xs text-slate-300">
                  {destinationLabel}
                </div>
                <div className="truncate px-3 py-2">{issueLabel}</div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function resolveStatusLabel(
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined
): string {
  if (row.scan_status !== "discovered") {
    return STATUS_LABELS[row.scan_status];
  }

  if (metadata?.metadata_status === "error") {
    return "Metadata Error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Warning Fallback";
  }

  if (!movePlan) {
    return metadata?.metadata_status === "ready" ? "Planning" : "Pending Metadata";
  }

  switch (movePlan.plan_status) {
    case "ready":
      return movePlan.conflict_status === "suffix_applied" ? "Planned (Suffix)" : "Planned";
    case "blocked_no_date":
      return "Blocked No Date";
    case "blocked_conflict":
      return "Blocked Conflict";
    case "error":
      return "Plan Error";
    default:
      return "Pending Plan";
  }
}

function resolveIssueLabel(
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined
): string {
  if (metadata?.metadata_status === "error") {
    return metadata.error ?? "Metadata error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Using FileModifyDate fallback";
  }
  if (!metadata) {
    return "-";
  }

  if (movePlan?.plan_status === "blocked_no_date") {
    return "No usable metadata date";
  }
  if (movePlan?.conflict_status === "suffix_applied") {
    return "Suffix applied for conflict";
  }

  return metadata.chosen_date_source ?? "-";
}
