// Returns the same object when no key goes, so state setters skip the render.
export function withoutFileIds<T>(byFileId: Record<string, T>, fileIds: string[]) {
  if (!fileIds.some((fileId) => fileId in byFileId)) {
    return byFileId;
  }
  const next = { ...byFileId };
  for (const fileId of fileIds) {
    delete next[fileId];
  }
  return next;
}
