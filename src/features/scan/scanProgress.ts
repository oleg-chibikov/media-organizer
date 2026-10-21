export interface ScanProgress {
  discovered: number;
  skipped: number;
  errors: number;
  completed: boolean;
  cancelled: boolean;
}

export const EMPTY_SCAN_PROGRESS: ScanProgress = {
  discovered: 0,
  skipped: 0,
  errors: 0,
  completed: false,
  cancelled: false
};
