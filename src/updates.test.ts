import { describe, expect, it } from "vitest";
import { compareVersions, pickUpdate, type Release } from "./updates";

const rel = (tag: string, prerelease = tag.includes("-"), draft = false): Release => ({
  tag_name: tag,
  html_url: `https://github.com/mburakaltiparmak/peron/releases/tag/${tag}`,
  draft,
  prerelease,
});

describe("compareVersions (SemVer precedence)", () => {
  it.each([
    ["0.1.0-beta.1", "0.1.0-beta.2", -1],
    ["0.1.0-beta.2", "0.1.0-beta.10", -1],
    ["0.1.0-beta.9", "0.1.0", -1],
    ["0.1.0", "0.1.1-beta.1", -1],
    ["v1.0.0", "1.0.0", 0],
    ["1.0.0-alpha", "1.0.0-alpha.1", -1],
    ["1.0.0-alpha.1", "1.0.0-alpha.beta", -1],
    ["2.0.0", "1.9.9", 1],
  ])("%s vs %s", (a, b, sign) => {
    expect(Math.sign(compareVersions(a, b))).toBe(sign);
  });
});

describe("pickUpdate", () => {
  it("offers the newest pre-release while running a beta", () => {
    const u = pickUpdate("0.1.0-beta.1", [rel("v0.1.0-beta.1"), rel("v0.1.0-beta.3"), rel("v0.1.0-beta.2")]);
    expect(u?.version).toBe("0.1.0-beta.3");
  });

  it("offers the stable release over betas of the same version", () => {
    expect(pickUpdate("0.1.0-beta.3", [rel("v0.1.0-beta.4"), rel("v0.1.0")])?.version).toBe("0.1.0");
  });

  it("ignores pre-releases once on stable, and drafts always", () => {
    expect(pickUpdate("0.1.0", [rel("v0.2.0-beta.1"), rel("v0.3.0", false, true)])).toBeNull();
  });

  it("returns null when already up to date", () => {
    expect(pickUpdate("0.1.0", [rel("v0.1.0"), rel("v0.0.9")])).toBeNull();
  });
});
