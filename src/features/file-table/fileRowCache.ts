import type { FileIdMap } from "../../lib/fileIdMap";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { buildFileTableRow } from "./buildFileTableRow";
import { fileRowCategory, type FileRowCategory } from "./fileRowCategory";
import type { FileRowState } from "./fileRowState";
import type { FileTableRow } from "./fileTableColumns";

interface FileRowInputs {
  metadataByFileId: FileIdMap<MetadataResult>;
  movePlansByFileId: FileIdMap<MovePlan>;
  moveResultsByFileId: FileIdMap<MoveResult>;
  planErrorsByFileId: Record<string, string>;
}

interface RowContext {
  activeMoveFileId: string | null;
  destinationRoot: string | null;
  filePlaces: Record<string, string>;
  locatingPlaces: boolean;
}

interface Entry {
  state: FileRowState;
  category: FileRowCategory;
  row?: FileTableRow;
  rowPlace?: string;
  rowActive?: boolean;
  rowGeneration?: number;
}

// A batch of new dates or plans touches a few rows of thousands. The cache keeps every other
// row's state and table cells, so a render rebuilds only the rows that changed.
export function createFileRowCache() {
  const entries = new Map<string, Entry>();
  let rowGeneration = 0;
  let rowContextKey = "";

  function states(records: FileRecord[], inputs: FileRowInputs): FileRowState[] {
    const result = records.map((record) => {
      const fileId = record.id;
      const metadata = inputs.metadataByFileId.get(fileId);
      const movePlan = inputs.movePlansByFileId.get(fileId);
      const moveResult = inputs.moveResultsByFileId.get(fileId);
      const planError = inputs.planErrorsByFileId[fileId];
      const state = entries.get(fileId)?.state;
      if (
        state?.record === record &&
        state.metadata === metadata &&
        state.movePlan === movePlan &&
        state.moveResult === moveResult &&
        state.planError === planError
      ) {
        return state;
      }
      const next = { record, metadata, movePlan, moveResult, planError };
      entries.set(fileId, { state: next, category: fileRowCategory(next) });
      return next;
    });
    if (entries.size > records.length) {
      const current = new Set(records.map((record) => record.id));
      entries.forEach((_, fileId) => current.has(fileId) || entries.delete(fileId));
    }
    return result;
  }

  const categoryOf = (state: FileRowState): FileRowCategory =>
    entries.get(state.record.id)?.category ?? fileRowCategory(state);

  function rows(rowStates: FileRowState[], context: RowContext): FileTableRow[] {
    const contextKey = `${context.destinationRoot ?? ""}\n${context.locatingPlaces}`;
    if (contextKey !== rowContextKey) {
      rowContextKey = contextKey;
      rowGeneration += 1;
    }
    return rowStates.map((state) => {
      const fileId = state.record.id;
      const place = context.filePlaces[fileId];
      const active = context.activeMoveFileId === fileId;
      const entry = entries.get(fileId);
      const cached = entry?.state === state ? entry : undefined;
      if (
        cached?.row &&
        cached.rowPlace === place &&
        cached.rowActive === active &&
        cached.rowGeneration === rowGeneration
      ) {
        return cached.row;
      }
      const row = buildFileTableRow(
        state,
        context.activeMoveFileId,
        context.destinationRoot,
        place,
        context.locatingPlaces
      );
      if (cached) {
        Object.assign(cached, { row, rowPlace: place, rowActive: active, rowGeneration });
      }
      return row;
    });
  }

  return { states, categoryOf, rows };
}
