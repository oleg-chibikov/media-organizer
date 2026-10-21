import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";

export interface FileRowState {
  record: FileRecord;
  metadata: MetadataResult | undefined;
  movePlan: MovePlan | undefined;
  moveResult: MoveResult | undefined;
  planError: string | undefined;
}
