function normalizePath(pathValue: string): string {
  return pathValue.replace(/\\/g, "/").replace(/\/+$/, "");
}

// Shows a destination relative to the root media folder when it sits inside it.
export function relativeDestinationPath(pathValue: string, basePath: string | null): string {
  if (!basePath || pathValue === "-" || pathValue === "planning...") {
    return pathValue;
  }

  const normalizedBase = normalizePath(basePath);
  const normalizedPath = normalizePath(pathValue);
  if (!normalizedPath.toLowerCase().startsWith(`${normalizedBase.toLowerCase()}/`)) {
    return pathValue;
  }

  return normalizedPath.slice(normalizedBase.length + 1);
}
