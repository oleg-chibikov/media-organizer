import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileTable } from "./FileTable";
import type { FileRecord, MetadataResult, MovePlan } from "../lib/types";

vi.mock("./design-system/ResizableVirtualTable", () => ({
  ResizableVirtualTable: ({
    rows,
    columns
  }: {
    rows: Array<{ id: string; [key: string]: { value: string; title: string } | string }>;
    columns: Array<{ key: string; label: string }>;
  }) => (
    <table>
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column.key}>{column.label}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={String(row.id)}>
            {columns.map((column) => {
              const cell = row[column.key] as { value: string; title: string };
              return (
                <td key={column.key} title={cell.title}>
                  {cell.value}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  )
}));

function createRow(id: string): FileRecord {
  return {
    id,
    source_path: `C:\\in\\${id}.jpg`,
    relative_path: `${id}.jpg`,
    extension: "jpg",
    original_name: `${id}.jpg`,
    kind: "photo",
    scan_status: "discovered"
  };
}

function createMetadata(
  fileId: string,
  overrides: Partial<MetadataResult> = {}
): MetadataResult {
  return {
    file_id: fileId,
    chosen_date: "2026-05-25T00:00:00Z",
    chosen_date_source: "DateTimeOriginal",
    date_kind: "timezone_aware",
    raw_metadata_date: "2026:05:25 00:00:00",
    device_name: "Camera",
    gps_coordinate: null,
    gps_source: null,
    place_bucket_id: null,
    place_status: "not_available",
    place: null,
    place_error: null,
    metadata_status: "ready",
    error: null,
    ...overrides
  };
}

describe("FileTable place display", () => {
  it("renders pending, resolved, unavailable, and failed place states without raw coordinates", () => {
    const rows = ["pending", "resolved", "unavailable", "failed"].map(createRow);
    const metadataByFileId = {
      pending: createMetadata("pending", { place_status: "pending" }),
      resolved: createMetadata("resolved", {
        gps_coordinate: { latitude: 37.7749, longitude: -122.4194 },
        place_status: "resolved",
        place: "San Francisco, California, United States"
      }),
      unavailable: createMetadata("unavailable", { place_status: "not_available" }),
      failed: createMetadata("failed", {
        place_status: "failed",
        place_error: "provider timeout"
      })
    };

    render(
      <FileTable
        rows={rows}
        metadataByFileId={metadataByFileId}
        movePlansByFileId={{}}
        moveResultsByFileId={{}}
        activeMoveFileId={null}
        rootMediaFolder={null}
      />
    );

    expect(screen.getByText("Pending lookup")).toBeInTheDocument();
    expect(screen.getByText("San Francisco, California, United States")).toBeInTheDocument();
    expect(screen.getByText("Not available")).toBeInTheDocument();
    expect(screen.getByText("Lookup failed")).toBeInTheDocument();
    expect(screen.queryByText("37.7749")).not.toBeInTheDocument();
    expect(screen.queryByText("-122.4194")).not.toBeInTheDocument();
  });

  it("keeps move flow status ready when place lookup fails", () => {
    const row = createRow("failed-ready");
    const metadata = createMetadata("failed-ready", {
      place_status: "failed",
      place_error: "provider timeout"
    });
    const movePlan: MovePlan = {
      file_id: "failed-ready",
      original_name: "failed-ready.jpg",
      destination_dir: "C:\\out\\2026",
      destination_filename: "failed-ready.jpg",
      destination_path: "C:\\out\\2026\\failed-ready.jpg",
      conflict_status: "none",
      plan_status: "ready"
    };

    render(
      <FileTable
        rows={[row]}
        metadataByFileId={{ "failed-ready": metadata }}
        movePlansByFileId={{ "failed-ready": movePlan }}
        moveResultsByFileId={{}}
        activeMoveFileId={null}
        rootMediaFolder={null}
      />
    );

    expect(screen.getAllByText("Lookup failed").length).toBeGreaterThan(0);
    expect(screen.getByText("Ready")).toBeInTheDocument();
  });
});
