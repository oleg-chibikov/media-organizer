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
  it("renders a labelled table with resizable headers and clickable rows", () => {
    const onRowClick = vi.fn();
    render(
      <ResizableVirtualTable
        rows={[{ id: "a", name: "Alpha" }]}
        label="Names"
        columns={COLUMNS}
        columnDefs={COLUMN_DEFS}
        defaultColumnSizing={SIZING}
        onAutoSizeColumns={() => SIZING}
        virtualized={false}
        onRowClick={onRowClick}
      />
    );

    expect(screen.getByRole("table", { name: "Names" })).toHaveAttribute("aria-rowcount", "2");
    const resize = screen.getByRole("button", { name: "Resize Name column" });
    fireEvent.keyDown(resize, { key: "ArrowRight" });
    expect(screen.getByRole("row", { name: /Name/ })).toHaveStyle({ gridTemplateColumns: "136px" });
    expect(screen.getByRole("columnheader", { name: /Name/ })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("cell", { name: "Alpha" }));
    expect(onRowClick).toHaveBeenCalledWith({ id: "a", name: "Alpha" }, 0);
  });
});
