import type { Virtualizer } from "@tanstack/react-virtual";
import { TableRow } from "./TableRow";
import type { TableRowModel } from "./tableFeatures";
import type { RowBehavior } from "./types";

interface TableBodyProps<TRow extends object> {
  rows: TRow[];
  tableRows: TableRowModel<TRow>[];
  gridTemplateColumns: string;
  rowBehavior: RowBehavior<TRow>;
  // Renders only the rows in view; without it every row renders.
  rowVirtualizer: Virtualizer<HTMLDivElement, Element> | null;
}

export function TableBody<TRow extends object>({
  rows,
  tableRows,
  gridTemplateColumns,
  rowBehavior,
  rowVirtualizer
}: TableBodyProps<TRow>) {
  if (!rowVirtualizer) {
    return (
      <div className="min-h-full" role="rowgroup">
        {tableRows.map((row, rowIndex) => (
          <TableRow
            key={row.id}
            {...rowBehavior}
            row={row}
            sourceRow={rows[rowIndex]}
            rowIndex={rowIndex}
            layoutClassName="grid w-full"
            style={{ gridTemplateColumns }}
          />
        ))}
      </div>
    );
  }

  return (
    <div
      className="relative"
      role="rowgroup"
      style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
    >
      {rowVirtualizer.getVirtualItems().map((virtualRow) => (
        <TableRow
          key={tableRows[virtualRow.index].id}
          {...rowBehavior}
          row={tableRows[virtualRow.index]}
          sourceRow={rows[virtualRow.index]}
          rowIndex={virtualRow.index}
          layoutClassName="absolute left-0 top-0 grid w-full"
          style={{
            gridTemplateColumns,
            transform: `translateY(${virtualRow.start}px)`,
            height: `${virtualRow.size}px`
          }}
        />
      ))}
    </div>
  );
}
