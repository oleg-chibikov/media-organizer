import type { PlaceIconKind } from "../../components/design-system/PlaceIcon";

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
  // A house, car or plane before a place, by how far it is from home.
  icon?: PlaceIconKind;
}

export type FileTableRow = { id: string } & Record<FileColumnKey, FileCell>;
