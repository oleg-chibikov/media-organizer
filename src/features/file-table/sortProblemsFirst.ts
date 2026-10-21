import { fileRowCategory } from "./fileRowCategory";
import type { FileRowState } from "./fileRowState";

export interface ProblemsFirst {
  states: FileRowState[];
  errorCount: number;
  reviewCount: number;
}

// Keeps the name order inside each group, so errors are the first rows and review rows follow.
export function sortProblemsFirst(states: FileRowState[]): ProblemsFirst {
  const errors: FileRowState[] = [];
  const review: FileRowState[] = [];
  const rest: FileRowState[] = [];
  for (const state of states) {
    const category = fileRowCategory(state);
    (category === "error" ? errors : category === "review" ? review : rest).push(state);
  }
  const ordered = errors.length + review.length === 0 ? states : [...errors, ...review, ...rest];
  return { states: ordered, errorCount: errors.length, reviewCount: review.length };
}
