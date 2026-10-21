import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FileTable } from "./FileTable";
import type { FileRecord, MovePlan } from "../../lib/types";

vi.mock("../../lib/api", () => ({
  revealFileInExplorer: vi.fn()
}));

afterEach(() => {
  cleanup();
});

vi.mock("../../components/design-system/ResizableVirtualTable", () => ({
  ResizableVirtualTable: ({
    rows,
    columns
  }: {
    rows: Array<{ id: string; [key: string]: { value: string } | string }>;
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
          <tr key={row.id}>
            {columns.map((column) => (
              <td key={column.key}>{(row[column.key] as { value: string }).value}</td>
            ))}
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

type FileTableProps = Parameters<typeof FileTable>[0];

function renderTable(props: Partial<FileTableProps>) {
  return render(
    <FileTable
      rows={[]}
      metadataByFileId={{}}
      movePlansByFileId={{}}
      planErrorsByFileId={{}}
      moveResultsByFileId={{}}
      activeMoveFileId={null}
      destinationRoot={null}
      {...props}
    />
  );
}

describe("FileTable", () => {
  it("sorts displayed rows by name", () => {
    renderTable({ rows: [createRow("zeta"), createRow("Alpha"), createRow("beta")] });

    const renderedSourceOrder = screen
      .getAllByRole("row")
      .slice(1)
      .map((row) => row.querySelectorAll("td")[0]?.textContent);

    expect(renderedSourceOrder).toEqual(["Alpha.jpg", "beta.jpg", "zeta.jpg"]);
  });

  it("shows the destination inside the toolbar's destination folder", () => {
    const plan = { file_id: "a", destination_path: "D:\\Media\\2024\\5\\1\\a.jpg" } as MovePlan;
    renderTable({
      rows: [createRow("a")],
      movePlansByFileId: { a: plan },
      destinationRoot: "D:\\Media"
    });

    expect(screen.getByText("2024/5/1/a.jpg")).toBeInTheDocument();
    expect(screen.queryByText("Original Name")).not.toBeInTheDocument();
  });
});
