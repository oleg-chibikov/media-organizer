export const FILE_TABLE_COLUMNS = [
  { key: "source", label: "Source File" },
  { key: "status", label: "Status" },
  { key: "destination", label: "Destination" },
  { key: "date", label: "Date" },
  { key: "kind", label: "Kind" },
  { key: "notes", label: "Notes" }
] as const;

export type FileColumnKey = (typeof FILE_TABLE_COLUMNS)[number]["key"];

export interface FileCell {
  value: string;
  title: string;
  className: string;
}

export type FileTableRow = { id: string } & Record<FileColumnKey, FileCell>;
