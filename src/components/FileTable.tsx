import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import type { FileRecord, MetadataResult } from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
}

const STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Discovered",
  skipped_processed: "Skipped Processed",
  skipped_unsupported: "Skipped Unsupported",
  scan_error: "Scan Error"
};

export function FileTable({ rows, metadataByFileId }: FileTableProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20
  });

  return (
    <div className="flex h-full flex-col">
      <div className="grid grid-cols-[14rem_minmax(0,1fr)_8rem_10rem_10rem_10rem_8rem] border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400">
        <div className="px-3 py-2">Status</div>
        <div className="px-3 py-2">Source File</div>
        <div className="px-3 py-2">Kind</div>
        <div className="px-3 py-2">Original Name</div>
        <div className="px-3 py-2">Chosen Date</div>
        <div className="px-3 py-2">Date Source</div>
        <div className="px-3 py-2">Date Kind / Issue</div>
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
            const metadataDateLabel = metadata
              ? metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date"
              : "pending";
            const dateSourceLabel = metadata
              ? metadata.chosen_date_source ?? "-"
              : "pending";
            const issueLabel = metadata
              ? metadata.metadata_status === "error"
                ? metadata.error ?? "Metadata error"
                : metadata.date_kind ?? "-"
              : "-";

            return (
              <div
                key={row.id}
                className={`absolute left-0 top-0 grid w-full grid-cols-[14rem_minmax(0,1fr)_8rem_10rem_10rem_10rem_8rem] border-t border-slate-800 text-sm ${
                  isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""
                }`}
                style={{
                  transform: `translateY(${virtualRow.start}px)`,
                  height: `${virtualRow.size}px`
                }}
              >
                <div className="truncate px-3 py-2">{STATUS_LABELS[row.scan_status]}</div>
                <div className="truncate px-3 py-2 font-mono text-xs">{row.relative_path}</div>
                <div className="truncate px-3 py-2">{row.kind}</div>
                <div className="truncate px-3 py-2">{row.original_name}</div>
                <div className="truncate px-3 py-2">{metadataDateLabel}</div>
                <div className="truncate px-3 py-2 text-slate-500">{dateSourceLabel}</div>
                <div className="truncate px-3 py-2">{issueLabel}</div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
