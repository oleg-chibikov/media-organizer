import type { FileRecord } from "../../lib/types";

const FILE_NAME_COLLATOR = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

export function sortFileRecords(rows: FileRecord[]): FileRecord[] {
  return [...rows].sort(
    (left, right) =>
      FILE_NAME_COLLATOR.compare(left.original_name, right.original_name) ||
      FILE_NAME_COLLATOR.compare(left.relative_path, right.relative_path) ||
      FILE_NAME_COLLATOR.compare(left.id, right.id)
  );
}
