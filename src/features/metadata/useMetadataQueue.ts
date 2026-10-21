import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { readMetadataForFiles } from "../../lib/api";
import type { FileRecord, MetadataResult } from "../../lib/types";
import {
  createMetadataQueue,
  enqueueRowsNeedingMetadata,
  takeMetadataBatch
} from "./metadataQueue";

interface MetadataQueueOptions {
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  concurrency: number;
  queueCapacity: number;
  reportError: (message: string) => void;
  onMetadataRead: (rows: FileRecord[], results: MetadataResult[]) => void;
}

export function useMetadataQueue({
  rows,
  rowByIdRef,
  concurrency,
  queueCapacity,
  reportError,
  onMetadataRead
}: MetadataQueueOptions) {
  const [metadataByFileId, setMetadataByFileId] = useState<Record<string, MetadataResult>>({});
  const [activeMetadataRequests, setActiveMetadataRequests] = useState(0);
  const queueRef = useRef(createMetadataQueue());
  const pumpRef = useRef<() => void>(() => undefined);

  const changeActiveRequests = useCallback((delta: number) => {
    queueRef.current.activeRequests += delta;
    setActiveMetadataRequests(queueRef.current.activeRequests);
  }, []);

  const readBatch = useCallback(
    (batchRows: FileRecord[]) => {
      changeActiveRequests(1);
      void readMetadataForFiles(
        batchRows.map((row) => ({
          file_id: row.id,
          source_path: row.source_path,
          file_kind: row.kind
        }))
      )
        .then((results) => {
          setMetadataByFileId((prev) => {
            const next = { ...prev };
            for (const result of results) {
              next[result.file_id] = result;
            }
            return next;
          });
          for (const result of results) {
            if (result.metadata_status === "error" && result.error) {
              reportError(result.error);
            }
          }
          onMetadataRead(batchRows, results);
        })
        .catch((error: unknown) => {
          reportError(`Failed to read metadata batch: ${String(error)}`);
        })
        .finally(() => {
          changeActiveRequests(-1);
          pumpRef.current();
        });
    },
    [changeActiveRequests, onMetadataRead, reportError]
  );

  const pump = useCallback(() => {
    const queue = queueRef.current;
    while (queue.activeRequests < concurrency && queue.pending.length > 0) {
      const batchRows = takeMetadataBatch(queue, rowByIdRef.current);
      if (batchRows.length > 0) {
        readBatch(batchRows);
      }
    }
  }, [concurrency, readBatch, rowByIdRef]);

  useEffect(() => {
    pumpRef.current = pump;
  }, [pump]);

  useEffect(() => {
    enqueueRowsNeedingMetadata(queueRef.current, rows, metadataByFileId, queueCapacity);
    pump();
  }, [metadataByFileId, queueCapacity, pump, rows]);

  const resetMetadata = useCallback(() => {
    queueRef.current = createMetadataQueue();
    setMetadataByFileId({});
    setActiveMetadataRequests(0);
  }, []);

  return { metadataByFileId, activeMetadataRequests, resetMetadata };
}
