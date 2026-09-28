import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { cliInstall, isValidTarget } from "./ServersDialog";

describe("SSH target validation (mirrors peron_core::remote::is_valid_target)", () => {
  it("accepts normal targets", () => {
    for (const ok of ["web-01", "deploy@web-01.example.com", "root@10.0.0.5", "my_alias", "u@fe80::1"]) {
      expect(isValidTarget(ok), ok).toBe(true);
    }
  });

  it("rejects anything ssh could read as an option or a shell can expand", () => {
    for (const bad of ["", "-oProxyCommand=calc", "a b", "u@h;rm -rf /", "u@h$(x)", "a@b@c", "`x`", "h\n"]) {
      expect(isValidTarget(bad), JSON.stringify(bad)).toBe(false);
    }
  });

  it("pins the install command to the app version (betas aren't on /latest)", () => {
    expect(cliInstall("0.1.0-beta.1")).toContain("/releases/download/v0.1.0-beta.1/peron-cli-x86_64-linux");
  });
});
