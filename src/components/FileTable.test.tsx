import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ColumnDef } from "@tanstack/react-table";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FileTable } from "./FileTable";
import type { FileRecord, MetadataResult, MovePlan } from "../lib/types";

vi.mock("../lib/api", () => ({
  revealFileInExplorer: vi.fn(),
  testPlaceLookup: vi.fn()
}));

import { testPlaceLookup } from "../lib/api";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

vi.mock("./design-system/ResizableVirtualTable", () => ({
  ResizableVirtualTable: ({
    rows,
    columns,
    columnDefs
  }: {
    rows: Array<{ id: string; [key: string]: { value: string; title: string } | string }>;
    columns: Array<{ key: string; label: string }>;
    columnDefs: Array<ColumnDef<{ id: string }>>;
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
            {columns.map((column, columnIndex) => {
              const cell = row[column.key] as { value: string; title: string };
              const columnDef = columnDefs[columnIndex];
              const rendered =
                typeof columnDef.cell === "function"
                  ? columnDef.cell({
                      getValue: () => cell
                    } as never)
                  : cell.value;
              return (
                <td key={column.key} title={cell.title}>
                  {rendered}
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

function createMetadata(fileId: string, overrides: Partial<MetadataResult> = {}): MetadataResult {
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
  const mockedTestPlaceLookup = vi.mocked(testPlaceLookup);

  it("renders coordinates in the grid and includes reasons for unavailable places", () => {
    const rows = ["pending", "resolved", "unavailable", "failed"].map(createRow);
    const metadataByFileId = {
      pending: createMetadata("pending", {
        gps_coordinate: { latitude: 51.5074, longitude: -0.1278 },
        place_status: "pending"
      }),
      resolved: createMetadata("resolved", {
        gps_coordinate: { latitude: 37.7749, longitude: -122.4194 },
        place_status: "resolved",
        place: "San Francisco, California, United States"
      }),
      unavailable: createMetadata("unavailable", {
        gps_coordinate: { latitude: -33.8688, longitude: 151.2093 },
        place_status: "not_available"
      }),
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
    expect(screen.getByText("37.7749, -122.4194")).toBeInTheDocument();
    expect(screen.getByText("-33.8688, 151.2093")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Test" }).length).toBeGreaterThan(0);
    expect(
      screen.getAllByTitle(
        "Not available: GPS coordinates are present but place lookup did not produce a place. (Coordinates: -33.8688, 151.2093)"
      ).length
    ).toBeGreaterThan(0);
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

  it("runs a diagnostic place lookup for a GPS row", async () => {
    mockedTestPlaceLookup.mockResolvedValueOnce({
      coordinate: { latitude: 37.723511, longitude: -119.718558 },
      bucket_id: "v1:377:-1198",
      provider: "nominatim",
      request_url: "https://nominatim.openstreetmap.org/reverse?lat=37.723511",
      http_status: 200,
      display_place: "Yosemite Valley, California, United States",
      components: {
        city: "Yosemite Valley",
        region: "California",
        country: "United States"
      },
      raw_response: {
        display_name: "Yosemite Valley, Mariposa County, California, United States"
      },
      error: null
    });

    render(
      <FileTable
        rows={[createRow("yosemite")]}
        metadataByFileId={{
          yosemite: createMetadata("yosemite", {
            gps_coordinate: { latitude: 37.723511, longitude: -119.718558 },
            place_status: "failed",
            place_error: "Place lookup returned no address result."
          })
        }}
        movePlansByFileId={{}}
        moveResultsByFileId={{}}
        activeMoveFileId={null}
        rootMediaFolder={null}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Test" }));

    expect(mockedTestPlaceLookup).toHaveBeenCalledWith({
      latitude: 37.723511,
      longitude: -119.718558
    });
    await waitFor(() => {
      expect(
        screen.getByText("200: Yosemite Valley, California, United States")
      ).toBeInTheDocument();
    });
    expect(screen.getByText("Yosemite Valley, California, United States")).toBeInTheDocument();
    expect(
      screen.getAllByTitle(
        "Diagnostic lookup: Yosemite Valley, California, United States | HTTP: 200 | Provider: nominatim | Coordinates: 37.723511, -119.718558"
      ).length
    ).toBeGreaterThan(0);
  });
});
