// Per-file values for folders of 100,000 files. A copy of a plain object per batch cost tens of
// milliseconds there, so updates write into one shared Map and return a new handle for React.
// An older handle reads the newer values too, so read the latest one from state.
export class FileIdMap<T> {
  private constructor(private readonly entries: Map<string, T>) {}

  static empty<T>(): FileIdMap<T> {
    return new FileIdMap(new Map());
  }

  static from<T>(values: Record<string, T>): FileIdMap<T> {
    return new FileIdMap(new Map(Object.entries(values)));
  }

  get size(): number {
    return this.entries.size;
  }

  get(fileId: string): T | undefined {
    return this.entries.get(fileId);
  }

  has(fileId: string): boolean {
    return this.entries.has(fileId);
  }

  values(): IterableIterator<T> {
    return this.entries.values();
  }

  // Returns this handle when the batch is empty, so React skips the render.
  with(values: Iterable<T>, fileIdOf: (value: T) => string): FileIdMap<T> {
    let changed = false;
    for (const value of values) {
      this.entries.set(fileIdOf(value), value);
      changed = true;
    }
    return changed ? new FileIdMap(this.entries) : this;
  }

  without(fileIds: Iterable<string>): FileIdMap<T> {
    let changed = false;
    for (const fileId of fileIds) {
      changed = this.entries.delete(fileId) || changed;
    }
    return changed ? new FileIdMap(this.entries) : this;
  }
}
