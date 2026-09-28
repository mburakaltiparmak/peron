import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { groupRows } from "../filter";
import { port } from "../test/fixtures";
import { PortTable } from "./PortTable";

const rows = groupRows(
  [
    port({ localPort: 3000, processName: "node.exe" }),
    port({ localPort: 135, processName: "svchost.exe", isProtected: true, isSystem: true, isDev: false }),
  ],
  true,
);

function setup() {
  const onSelect = vi.fn();
  const onKill = vi.fn();
  render(
    <PortTable
      ports={rows}
      now={1_700_000_000_000 + 60_000}
      longOpenMs={4 * 3600_000}
      selectedId={null}
      onSelect={onSelect}
      onKill={onKill}
      onClearFilters={vi.fn()}
    />,
  );
  const bodyRows = screen.getAllByRole("row").slice(1); // skip header
  return { onSelect, onKill, bodyRows };
}

describe("PortTable", () => {
  it("rows are keyboard reachable: Enter/Space select, arrows move", async () => {
    const { onSelect, bodyRows } = setup();
    const byPort = (p: string) => bodyRows.find((r) => within(r).queryByText(p))!;
    byPort("135").focus();
    fireEvent.keyDown(byPort("135"), { key: "Enter" });
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ localPort: 135 }));
    fireEvent.keyDown(byPort("135"), { key: " " });
    expect(onSelect).toHaveBeenCalledTimes(2);
    fireEvent.keyDown(byPort("135"), { key: "ArrowDown" });
    expect(document.activeElement).toBe(byPort("3000"));
  });

  it("protected processes can't be closed; others ask via onKill (never kill directly)", async () => {
    const { onKill, bodyRows } = setup();
    const svchost = bodyRows.find((r) => within(r).queryByText("svchost.exe"))!;
    const node = bodyRows.find((r) => within(r).queryByText("node.exe"))!;
    expect(within(svchost).getByRole("button", { name: /Kapat/ })).toBeDisabled();
    await userEvent.click(within(node).getByRole("button", { name: /Kapat/ }));
    expect(onKill).toHaveBeenCalledWith(expect.objectContaining({ localPort: 3000 }));
  });

  it("column headers sort via buttons (keyboard accessible)", async () => {
    setup();
    const header = screen.getByRole("button", { name: /Port/ });
    await userEvent.click(header);
    expect(header.closest("th")).toHaveAttribute("aria-sort", "descending");
  });
});
