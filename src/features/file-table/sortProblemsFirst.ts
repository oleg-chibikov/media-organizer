import { fileRowCategory } from "./fileRowCategory";
import type { FileRowState } from "./fileRowState";

// Keeps the name order inside each group, so only the problem rows move.
export function sortProblemsFirst(states: FileRowState[]): FileRowState[] {
  const errors: FileRowState[] = [];
  const review: FileRowState[] = [];
  const rest: FileRowState[] = [];
  for (const state of states) {
    const category = fileRowCategory(state);
    (category === "error" ? errors : category === "review" ? review : rest).push(state);
  }
  return errors.length + review.length === 0 ? states : [...errors, ...review, ...rest];
}
