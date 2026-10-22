import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { EMPTY_SCAN_PROGRESS } from "../scan";
import { StatusCounts } from "./StatusCounts";

const NO_MOVES = { moved: 0, copied: 0, skipped: 0, failed: 0 };
const NO_PLANS = {
  total_plans: 0,
  ready_operations: 0,
  skipped_operations: 0,
  blocked_no_date: 0,
  blocked_conflict: 0,
  plan_errors: 0,
  suffix_applied: 0,
  fallback_warnings: 0,
  photos_ready: 0,
  videos_ready: 0,
  gather_library: true,
  library_folders: []
};

function renderCounts(read: number, total: number) {
  render(
    <StatusCounts
      scan={{ ...EMPTY_SCAN_PROGRESS, discovered: total }}
      metadata={{ read, total }}
      planErrors={0}
      summary={NO_PLANS}
      showReady
      moveCounts={NO_MOVES}
    />
  );
}

describe("StatusCounts", () => {
  afterEach(cleanup);

  it("counts the dates read while reading", () => {
    renderCounts(3, 10);
    expect(screen.getByText("Files: 10")).toBeInTheDocument();
    expect(screen.getByText("Metadata read: 3/10")).toBeInTheDocument();
  });

  it("drops the dates count once every date is read", () => {
    renderCounts(10, 10);
    expect(screen.queryByText(/Metadata read/)).not.toBeInTheDocument();
  });
});
