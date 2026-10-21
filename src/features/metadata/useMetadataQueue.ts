import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { readMetadataForFiles } from "../../lib/api";
import type { FileRecord, MetadataResult } from "../../lib/types";
import {
  createMetadataQueue,
  enqueueRowsNeedingMetadata,
  takeMetadataBatch,
  toMetadataRequest,
  type MetadataQueue
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

  const changeActiveRequests = useCallback((queue: MetadataQueue, delta: number) => {
    queue.activeRequests += delta;
    setActiveMetadataRequests(queue.activeRequests);
  }, []);

  const readBatch = useCallback(
    (batchRows: FileRecord[]) => {
      // A reset swaps the queue; a batch from the old scan must not touch the new one.
      const queue = queueRef.current;
      const isCurrent = () => queueRef.current === queue;
      changeActiveRequests(queue, 1);
      void readMetadataForFiles(batchRows.map(toMetadataRequest))
        .then((results) => {
          if (!isCurrent()) {
            return;
          }
          const byFileId = Object.fromEntries(results.map((result) => [result.file_id, result]));
          setMetadataByFileId((prev) => ({ ...prev, ...byFileId }));
          for (const result of results) {
            if (result.metadata_status === "error" && result.error) {
              reportError(result.error);
            }
          }
          onMetadataRead(batchRows, results);
        })
        .catch((error: unknown) => {
          if (isCurrent()) {
            reportError(`Failed to read metadata batch: ${String(error)}`);
          }
        })
        .finally(() => {
          if (isCurrent()) {
            changeActiveRequests(queue, -1);
            pumpRef.current();
          }
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
