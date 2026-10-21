export const FILE_TABLE_COLUMNS = [
  { key: "source", label: "Source File" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" },
  { key: "kind", label: "Kind" },
  { key: "originalName", label: "Original Name" },
  { key: "metadataDate", label: "Metadata Date" },
  { key: "notes", label: "Notes" }
] as const;

export type FileColumnKey = (typeof FILE_TABLE_COLUMNS)[number]["key"];

export interface FileCell {
  value: string;
  title: string;
  className: string;
}

export type FileTableRow = { id: string } & Record<FileColumnKey, FileCell>;
