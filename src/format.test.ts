import { describe, expect, it } from "vitest";
import { formatBytes, prereleaseLabel } from "./format";

// Project names are computed in Rust (peron-core `model::project_of`, tested there).

describe("prereleaseLabel", () => {
  it("labels pre-releases only", () => {
    expect(prereleaseLabel("0.1.0-beta.1")).toBe("BETA");
    expect(prereleaseLabel("1.2.0-rc.2")).toBe("RC");
    expect(prereleaseLabel("0.1.0")).toBeNull();
  });
});

describe("formatBytes", () => {
  it("scales units", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(50 * 1024 * 1024)).toBe("50 MB");
  });
});
