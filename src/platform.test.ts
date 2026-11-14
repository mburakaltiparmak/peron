import { describe, expect, it } from "vitest";
import { isMac } from "./platform";

describe("isMac", () => {
  it("detects macOS from navigator.platform, falling back to the user agent", () => {
    expect(isMac({ platform: "MacIntel", userAgent: "" })).toBe(true);
    expect(isMac({ platform: "", userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit" })).toBe(true);
    expect(isMac({ platform: "Win32", userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" })).toBe(false);
    expect(isMac({ platform: "Linux x86_64", userAgent: "Mozilla/5.0 (X11; Linux x86_64)" })).toBe(false);
  });
});
