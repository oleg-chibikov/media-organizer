export const FILE_TABLE_COLUMNS = [
  { key: "source", label: "Source File" },
  { key: "status", label: "Status" },
  { key: "destination", label: "Destination" },
  { key: "date", label: "Date" },
  { key: "country", label: "Country" },
  { key: "place", label: "Place" },
  { key: "kind", label: "Kind" },
  { key: "notes", label: "Notes" }
] as const;

export type FileColumnKey = (typeof FILE_TABLE_COLUMNS)[number]["key"];

export interface FileCell {
  value: string;
  title: string;
  className: string;
  // Shows a spinner before the text while the value is still on its way.
  loading?: boolean;
  // A house or a plane before a place, by whether it matches home.
  icon?: "home" | "away";
}

export type FileTableRow = { id: string } & Record<FileColumnKey, FileCell>;
