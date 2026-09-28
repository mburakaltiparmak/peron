import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppInfo } from "../types";

const fb = vi.hoisted(() => ({
  sendFeedback: vi.fn(),
  openInMailClient: vi.fn(),
}));
vi.mock("../feedback", async (orig) => ({
  ...(await orig<typeof import("../feedback")>()),
  ...fb,
  canSendDirect: true,
  validate: () => null, // skip the minimum fill time
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

import { ContactView } from "./ContactView";

const info = { version: "0.1.0-beta.1", os: "Windows 11" } as AppInfo;

async function submit(text = "Kapat düğmesi çalışmıyor.") {
  const onSent = vi.fn();
  render(<ContactView info={info} lang="tr" onSent={onSent} />);
  await userEvent.type(screen.getByRole("textbox", { name: /Mesaj/ }), text);
  await userEvent.click(screen.getByRole("button", { name: "Gönder" }));
  return onSent;
}

describe("ContactView send failures", () => {
  beforeEach(() => vi.clearAllMocks());

  it("offers the mail app when the service can't be reached, keeping the message", async () => {
    fb.sendFeedback.mockRejectedValue(new TypeError("Failed to fetch"));
    const onSent = await submit();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Gönderim hizmetine ulaşılamadı");
    expect(alert).not.toHaveTextContent("TypeError");
    expect(onSent).not.toHaveBeenCalled();
    expect(screen.getByRole("textbox", { name: /Mesaj/ })).toHaveValue("Kapat düğmesi çalışmıyor.");

    fb.openInMailClient.mockResolvedValue(undefined);
    await userEvent.click(screen.getByRole("button", { name: "Mail uygulamasında aç" }));
    expect(fb.openInMailClient).toHaveBeenCalledWith(expect.objectContaining({ message: "Kapat düğmesi çalışmıyor." }));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("shows the service's own error and still offers the mail app", async () => {
    fb.sendFeedback.mockRejectedValue(new Error("Invalid access key"));
    await submit();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Gönderilemedi: Invalid access key");
    expect(screen.getByRole("button", { name: "Mail uygulamasında aç" })).toBeInTheDocument();
  });

  it("opens the Buy Me a Coffee page in the browser", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    render(<ContactView info={info} lang="tr" onSent={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: /Bana bir kahve ısmarla/ }));
    expect(openUrl).toHaveBeenCalledWith("https://buymeacoffee.com/mburakaltiparmak");
  });

  it("clears the form after a successful send", async () => {
    fb.sendFeedback.mockResolvedValue(undefined);
    const onSent = await submit();
    expect(onSent).toHaveBeenCalledWith("Teşekkürler! Mesajınız iletildi.");
    expect(screen.getByRole("textbox", { name: /Mesaj/ })).toHaveValue("");
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
