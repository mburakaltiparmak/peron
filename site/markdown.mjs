// Minimal Markdown → HTML for the site pages (no dependencies). Used by site/build.mjs.

export const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// A bare URL never ends in sentence punctuation: "…policy: https://x.com/privacy." links to
// https://x.com/privacy, and the period stays text.
const BARE_URL = /(^|[\s(])(https?:\/\/[^\s)<]*[^\s)<.,;:!?'"])/g;

export function inline(s) {
  return esc(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
    .replace(BARE_URL, '$1<a href="$2">$2</a>');
}

/** Headings, paragraphs, lists, bold/italic/code/links, rules. */
export function markdown(md) {
  const html = [];
  let list = false;
  let para = [];
  const flush = () => {
    if (para.length) html.push(`<p>${inline(para.join(" "))}</p>`);
    para = [];
  };
  const closeList = () => {
    if (list) html.push("</ul>");
    list = false;
  };
  for (const raw of md.split(/\r?\n/)) {
    const line = raw.trimEnd();
    const h = /^(#{1,3}) (.*)$/.exec(line);
    if (h) {
      flush();
      closeList();
      const id = h[2].toLowerCase().replace(/[^a-z0-9ğüşıöç]+/g, "-");
      html.push(`<h${h[1].length} id="${id}">${inline(h[2])}</h${h[1].length}>`);
    } else if (/^[-*] /.test(line)) {
      flush();
      if (!list) html.push("<ul>");
      list = true;
      html.push(`<li>${inline(line.slice(2))}</li>`);
    } else if (/^---+$/.test(line)) {
      flush();
      closeList();
      html.push("<hr>");
    } else if (!line.trim()) {
      flush();
      closeList();
    } else {
      closeList();
      para.push(line.trim());
    }
  }
  flush();
  closeList();
  return html.join("\n");
}

/**
 * Hrefs in `html` that look broken: not an absolute http(s)/mailto URL or site path, unparsable, or
 * ending in punctuation that belonged to the sentence. Empty = all good.
 */
export function badLinks(html) {
  const bad = [];
  for (const [, href] of html.matchAll(/href="([^"]*)"/g)) {
    const ok =
      (/^(https?:\/\/|mailto:)/.test(href) && URL.canParse(href) && !/[.,;:!?)]$/.test(href)) ||
      /^\/[^\s]*$/.test(href);
    if (!ok) bad.push(href);
  }
  return bad;
}
