import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { badLinks, inline, markdown } from "./markdown.mjs";

describe("site markdown", () => {
  it.each([
    ["policy: https://web3forms.com/privacy.", "https://web3forms.com/privacy", "."],
    ["see https://a.dev/x, then", "https://a.dev/x", ","],
    ["(https://a.dev/y)", "https://a.dev/y", ")"],
    ["https://a.dev/z:", "https://a.dev/z", ":"],
  ])("bare URL in %j keeps trailing punctuation as text", (text, url, tail) => {
    const html = inline(text);
    expect(html).toContain(`<a href="${url}">${url}</a>${tail}`);
    expect(badLinks(html)).toEqual([]);
  });

  it("keeps dots inside URLs", () => {
    expect(inline("https://docs.github.com/site-policy/a.html")).toContain(
      'href="https://docs.github.com/site-policy/a.html"',
    );
  });

  it("flags broken hrefs", () => {
    expect(badLinks('<a href="https://x.dev/privacy.">')).toEqual(["https://x.dev/privacy."]);
    expect(badLinks('<a href="javascript:alert(1)">')).toHaveLength(1);
    expect(badLinks('<a href="/peron/support/"><a href="mailto:a@b.c">')).toEqual([]);
  });

  it.each(["PRIVACY.md", "SUPPORT.md"])("%s renders with valid links", (name) => {
    const html = markdown(readFileSync(join(process.cwd(), name), "utf8"));
    expect(html).toContain("<a href=");
    expect(badLinks(html)).toEqual([]);
  });
});
