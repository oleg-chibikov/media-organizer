import type { MoveCounts } from "../moves";

const files = (count: number) => (count === 1 ? "1 file" : `${count} files`);

export function finishSummary({ moved, copied, skipped, failed }: MoveCounts): string {
  const parts = ["Done."];
  if (moved > 0) {
    parts.push(`Moved ${files(moved)}.`);
  }
  if (copied > 0) {
    parts.push(`Copied ${files(copied)} and kept the originals.`);
  }
  if (skipped + failed > 0) {
    parts.push(`${files(skipped + failed)} didn't move. See Errors.`);
  }
  parts.push("You can undo this run in History.");
  return parts.join(" ");
}
