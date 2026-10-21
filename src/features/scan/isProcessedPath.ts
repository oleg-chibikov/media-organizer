export function isProcessedPath(folder: string): boolean {
  return folder
    .toLowerCase()
    .split(/[/\\]+/)
    .some((segment) => segment === "processed");
}
