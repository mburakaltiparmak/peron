import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Settings } from "../types";

const win = vi.hoisted(() => ({ setTheme: vi.fn(() => Promise.resolve()) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => win }));

import { SettingsDialog } from "./SettingsDialog";

const settings: Settings = {
  language: "tr",
  theme: "system",
  viewMode: "listen",
  hideSystem: true,
  showUdp: false,
  autoRefresh: true,
  refreshIntervalSecs: 3,
  reminderEnabled: true,
  reminderThresholdMinutes: 240,
  reminderRepeatMinutes: 60,
  closeToTray: true,
  autostart: false,
  checkUpdates: true,
  alertExposed: true,
  remoteHosts: [],
};

const theme = () => document.documentElement.getAttribute("data-theme");

describe("SettingsDialog theme", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    win.setTheme.mockClear();
  });

  it("previews the theme immediately, title bar included, and restores it on cancel", async () => {
    const onCancel = vi.fn();
    render(<SettingsDialog settings={settings} language="tr" channel="direct" onSave={vi.fn()} onCancel={onCancel} />);

    await userEvent.click(screen.getByRole("button", { name: "Açık" }));
    expect(theme()).toBe("light");
    expect(win.setTheme).toHaveBeenLastCalledWith("light");

    await userEvent.click(screen.getByRole("button", { name: "İptal" }));
    expect(theme()).toBeNull();
    expect(win.setTheme).toHaveBeenLastCalledWith(null);
    expect(onCancel).toHaveBeenCalled();
  });

  it("shows the reminder limits instead of changing values silently on save", async () => {
    const onSave = vi.fn(() => Promise.resolve());
    render(<SettingsDialog settings={settings} language="tr" channel="direct" onSave={onSave} onCancel={vi.fn()} />);
    const threshold = screen.getByRole("spinbutton", { name: /Şu süreden uzun açık kalınca/ });
    const repeat = screen.getByRole("spinbutton", { name: /Tekrar hatırlatma aralığı \(dakika, 5–1440\)/ });

    await userEvent.clear(threshold);
    await userEvent.type(threshold, "1");
    await userEvent.clear(repeat);
    await userEvent.type(repeat, "1");
    await userEvent.tab();
    expect(threshold).toHaveValue(1); // 1 minute is allowed
    expect(repeat).toHaveValue(5); // snapped to the minimum, visibly

    await userEvent.click(screen.getByRole("button", { name: "Kaydet" }));
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ reminderThresholdMinutes: 1, reminderRepeatMinutes: 5 }));
  });

  it("saves the picked theme", async () => {
    const onSave = vi.fn(() => Promise.resolve());
    render(<SettingsDialog settings={settings} language="tr" channel="direct" onSave={onSave} onCancel={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "Koyu" }));
    expect(theme()).toBe("dark");
    await userEvent.click(screen.getByRole("button", { name: "Kaydet" }));
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ theme: "dark" }));
  });
});
