// Why files that aren't checked yet don't move, while the scan or its dates are still read.
export function moveWaitHint(groupByPlace: boolean): string {
  return groupByPlace
    ? "Files move once every date is read. A place folder depends on all the files of the folder, so the destinations shown now can still change."
    : "Each file moves as soon as its date is read. Files still being checked join the run when they are ready.";
}
