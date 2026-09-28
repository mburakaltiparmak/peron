import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { type FeedbackInput, MIN_FILL_MS, validate } from "./feedback";

const base = (over: Partial<FeedbackInput> = {}): FeedbackInput => ({
  kind: "bug",
  kindLabel: "Bug",
  name: "",
  email: "",
  message: "The port list does not refresh after sleep.",
  systemInfo: null,
  website: "",
  openedAt: 0,
  ...over,
});

describe("feedback validation (client-side bot protection)", () => {
  const later = MIN_FILL_MS + 1;

  it("accepts a normal message", () => {
    expect(validate(base(), later)).toBeNull();
  });

  it("rejects forms submitted faster than a human can type", () => {
    expect(validate(base(), MIN_FILL_MS - 1)).toBe("tooFast");
  });

  it("rejects too short, too long and link-stuffed messages", () => {
    expect(validate(base({ message: "hi" }), later)).toBe("tooShort");
    expect(validate(base({ message: "x".repeat(5001) }), later)).toBe("tooLong");
    const links = "see https://a.io https://b.io https://c.io https://d.io";
    expect(validate(base({ message: links }), later)).toBe("tooManyLinks");
  });

  it("checks the optional reply address", () => {
    expect(validate(base({ email: "not-an-email" }), later)).toBe("badEmail");
    expect(validate(base({ email: "me@example.com" }), later)).toBeNull();
  });
});
