function normalizePath(pathValue: string): string {
  return pathValue.replace(/\\/g, "/").replace(/\/+$/, "");
}

// Shows a destination relative to the destination root shown in the toolbar.
export function relativeDestinationPath(pathValue: string, basePath: string | null): string {
  if (!basePath) {
    return pathValue;
  }

  const normalizedBase = normalizePath(basePath);
  const normalizedPath = normalizePath(pathValue);
  if (!normalizedPath.toLowerCase().startsWith(`${normalizedBase.toLowerCase()}/`)) {
    return pathValue;
  }

  return normalizedPath.slice(normalizedBase.length + 1);
}
