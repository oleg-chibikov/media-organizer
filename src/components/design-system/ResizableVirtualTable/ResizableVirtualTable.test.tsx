import type { ColumnDef } from "@tanstack/react-table";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ResizableVirtualTable } from "./ResizableVirtualTable";

type Row = { id: string; name: string };

const COLUMNS = [{ key: "name", label: "Name" }] as const;
const COLUMN_DEFS: ColumnDef<Row>[] = [{ id: "name", header: "Name", accessorKey: "name" }];
const SIZING = { name: 120 };

afterEach(cleanup);

describe("ResizableVirtualTable", () => {
  it("renders resizable headers and keyboard-selectable rows", () => {
    const onRowClick = vi.fn();
    render(
      <ResizableVirtualTable
        rows={[{ id: "a", name: "Alpha" }]}
        columns={COLUMNS}
        columnDefs={COLUMN_DEFS}
        defaultColumnSizing={SIZING}
        onAutoSizeColumns={() => SIZING}
        virtualized={false}
        selectable={true}
        onRowClick={onRowClick}
        getRowAriaLabel={(row) => `Open ${row.name}`}
      />
    );

    expect(screen.getByRole("button", { name: "Resize Name column" })).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole("button", { name: "Open Alpha" }), { key: "Enter" });
    expect(onRowClick).toHaveBeenCalledWith({ id: "a", name: "Alpha" }, 0);
  });
});
