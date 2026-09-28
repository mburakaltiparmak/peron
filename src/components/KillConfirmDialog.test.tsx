import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { port } from "../test/fixtures";

const api = vi.hoisted(() => ({
  killProcess: vi.fn(),
  remoteKill: vi.fn(),
  getProcessDetails: vi.fn(),
  relaunchAsAdmin: vi.fn(),
}));
vi.mock("../api", async (orig) => ({ ...(await orig<typeof import("../api")>()), ...api }));

import { KillConfirmDialog } from "./KillConfirmDialog";

describe("KillConfirmDialog — killing is always user-confirmed", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getProcessDetails.mockResolvedValue({ pid: 1, parentChain: [], children: [] });
    api.killProcess.mockResolvedValue({ killed: [1], failed: [] });
  });

  const entry = port({ pid: 4242, localPort: 3000, processName: "node.exe", processStartMs: 111 });

  it("does not end anything just by opening, and Cancel ends nothing", async () => {
    const onCancel = vi.fn();
    render(<KillConfirmDialog entry={entry} samePid={[entry]} onCancel={onCancel} onDone={vi.fn()} />);
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /İptal/ }));
    expect(onCancel).toHaveBeenCalled();
    expect(api.killProcess).not.toHaveBeenCalled();
  });

  it("Escape cancels without ending the process", async () => {
    const onCancel = vi.fn();
    render(<KillConfirmDialog entry={entry} samePid={[entry]} onCancel={onCancel} onDone={vi.fn()} />);
    await userEvent.keyboard("{Escape}");
    expect(onCancel).toHaveBeenCalled();
    expect(api.killProcess).not.toHaveBeenCalled();
  });

  it("ends exactly the shown process (PID + start time guard) after confirmation", async () => {
    const onDone = vi.fn();
    render(<KillConfirmDialog entry={entry} samePid={[entry]} onCancel={vi.fn()} onDone={onDone} />);
    await userEvent.click(screen.getByRole("button", { name: /Süreci sonlandır/ }));
    expect(api.killProcess).toHaveBeenCalledTimes(1);
    expect(api.killProcess).toHaveBeenCalledWith(4242, 111, false);
    await waitFor(() => expect(onDone).toHaveBeenCalled());
  });

  it("offers the admin relaunch when access is denied", async () => {
    api.killProcess.mockRejectedValue({ kind: "accessDenied", message: "x" });
    render(<KillConfirmDialog entry={entry} samePid={[entry]} onCancel={vi.fn()} onDone={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: /Süreci sonlandır/ }));
    expect(await screen.findByRole("button", { name: /Yönetici olarak yeniden başlat/ })).toBeInTheDocument();
  });

  it("remote rows end the process on the server, never locally, and skip the local tree lookup", async () => {
    api.remoteKill.mockResolvedValue({ killed: [4242], failed: [] });
    render(
      <KillConfirmDialog entry={entry} samePid={[entry]} remote={{ id: "h1", name: "web-01" }} onCancel={vi.fn()} onDone={vi.fn()} />,
    );
    expect(screen.getByText(/web-01/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Süreci sonlandır/ }));
    expect(api.remoteKill).toHaveBeenCalledWith("h1", 4242, 111, false);
    expect(api.killProcess).not.toHaveBeenCalled();
    expect(api.getProcessDetails).not.toHaveBeenCalled();
  });

  it("keeps keyboard focus inside the dialog", async () => {
    render(<KillConfirmDialog entry={entry} samePid={[entry]} onCancel={vi.fn()} onDone={vi.fn()} />);
    const dialog = screen.getByRole("alertdialog");
    for (let i = 0; i < 5; i++) {
      await userEvent.tab();
      expect(dialog).toContainElement(document.activeElement as HTMLElement);
    }
  });
});
