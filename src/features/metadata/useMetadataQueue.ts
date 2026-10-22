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
import { useMetadataResults } from "./useMetadataResults";

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
  const results = useMetadataResults(onMetadataRead);
  const { metadataByFileId, addRead, flushReads, clearResults } = results;
  const [activeMetadataRequests, setActiveMetadataRequests] = useState(0);
  const queueRef = useRef(createMetadataQueue());
  const pumpRef = useRef<() => void>(() => undefined);
  const pausedRef = useRef(false);
  const [paused, setPausedState] = useState(false);

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
          addRead({ rows: batchRows, results });
        })
        .catch((error: unknown) => {
          if (isCurrent()) {
            reportError(`Failed to read metadata batch: ${String(error)}`);
          }
        })
        .finally(() => {
          if (isCurrent()) {
            // Planning hears of the last results before the reads count as done.
            if (queue.activeRequests === 1) {
              flushReads();
            }
            changeActiveRequests(queue, -1);
            pumpRef.current();
          }
        });
    },
    [addRead, changeActiveRequests, flushReads, reportError]
  );

  const pump = useCallback(() => {
    const queue = queueRef.current;
    while (!pausedRef.current && queue.activeRequests < concurrency && queue.pending.length > 0) {
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

  // Reads already running finish; the next ones wait for the resume.
  const setPaused = useCallback((nextPaused: boolean) => {
    pausedRef.current = nextPaused;
    setPausedState(nextPaused);
    if (!nextPaused) {
      pumpRef.current();
    }
  }, []);

  const resetMetadata = useCallback(() => {
    clearResults();
    queueRef.current = createMetadataQueue();
    setActiveMetadataRequests(0);
  }, [clearResults]);

  return {
    metadataByFileId,
    activeMetadataRequests,
    paused,
    addResults: results.addResults,
    removeResults: results.removeResults,
    setPaused,
    resetMetadata
  };
}
