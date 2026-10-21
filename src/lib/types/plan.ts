type ConflictStatus = "none" | "suffix_applied" | "blocked";
export type PlanStatus =
  | "ready"
  | "blocked_no_date"
  | "blocked_conflict"
  | "already_named"
  | "error";

export interface MovePlan {
  file_id: string;
  original_name: string;
  destination_dir: string;
  destination_filename: string;
  destination_path: string;
  conflict_status: ConflictStatus;
  plan_status: PlanStatus;
}

export interface MoveConfirmationSummary {
  total_plans: number;
  ready_operations: number;
  skipped_operations: number;
  blocked_no_date: number;
  blocked_conflict: number;
  plan_errors: number;
  suffix_applied: number;
  fallback_warnings: number;
  photos_ready: number;
  videos_ready: number;
}

export interface PlanFileEvent {
  plan: MovePlan;
}

export interface PlanErrorEvent {
  file_id: string;
  error: string;
}

export interface PlanFinishedEvent {
  planned: number;
  errors: number;
}
