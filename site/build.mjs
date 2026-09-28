// Generates the public pages for https://burakaltiparmak.dev/products/peron from the repo's single sources:
//   PRIVACY.md → site/peron/privacy/index.html
//   SUPPORT.md → site/peron/support/index.html
//   package.json version → site/peron/index.html (download page, links to immutable GitHub
//   release assets). Upload the whole site/peron folder to the web server.
// Run: node site/build.mjs     (no dependencies; fails on broken links — see markdown.mjs)
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "site", "peron");
const version = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version;
const repo = "https://github.com/mburakaltiparmak/peron";

import { badLinks, esc, markdown } from "./markdown.mjs";

function page(title, body) {
  return `<!doctype html>
<html lang="tr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<link rel="icon" href="/peron/logo.png">
<style>
  :root { color-scheme: light dark; --fg:#1c2025; --muted:#69717c; --bg:#fff; --accent:#2563eb; --card:#f5f6f8; }
  @media (prefers-color-scheme: dark) { :root { --fg:#e6e8eb; --muted:#979fa9; --bg:#16181b; --accent:#6392f0; --card:#1e2125; } }
  body { margin:0; font:16px/1.6 "Segoe UI", system-ui, sans-serif; color:var(--fg); background:var(--bg); }
  main { max-width: 820px; margin: 0 auto; padding: 32px 20px 64px; }
  header { display:flex; align-items:center; gap:12px; margin-bottom:24px; }
  header a { color:inherit; text-decoration:none; font-weight:700; font-size:20px; display:flex; gap:10px; align-items:center; }
  nav { margin-left:auto; display:flex; gap:16px; font-size:14px; }
  a { color: var(--accent); }
  h1 { font-size: 28px; line-height:1.2; } h2 { margin-top: 36px; } h3 { margin-top: 24px; }
  code { background: var(--card); padding: 1px 5px; border-radius: 4px; }
  hr { border: none; border-top: 1px solid var(--card); margin: 36px 0; }
  .card { background: var(--card); border-radius: 12px; padding: 18px 20px; margin: 16px 0; }
  .btn { display:inline-block; background: var(--accent); color:#fff; padding:10px 16px; border-radius:8px; text-decoration:none; margin:4px 8px 4px 0; }
  .muted { color: var(--muted); font-size: 14px; }
  footer { margin-top: 48px; color: var(--muted); font-size: 13px; }
</style>
</head>
<body><main>
<header><a href="/peron/"><img src="/peron/logo.png" width="32" height="32" alt="">Peron</a>
<nav><a href="/peron/">İndir / Download</a><a href="/peron/support/">Destek / Support</a><a href="/peron/privacy/">Gizlilik / Privacy</a></nav></header>
${body}
<footer>© 2026 <a href="https://burakaltiparmak.dev">M. Burak Altıparmak</a> · <a href="${repo}">GitHub</a> · <a href="${repo}/blob/main/LICENSE">GPL-3.0</a> · <a href="${repo}/blob/main/EULA.md">Store EULA</a> · <a href="https://buymeacoffee.com/mburakaltiparmak">☕ Buy me a coffee</a></footer>
</main></body></html>
`;
}

function write(rel, html) {
  const bad = badLinks(html);
  if (bad.length) throw new Error(`${rel}: broken links: ${bad.join(", ")}`);
  const file = join(out, rel);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, html);
  console.log("wrote", file);
}

const tag = `v${version}`;
const asset = (name) => `${repo}/releases/download/${tag}/${name}`;
const beta = version.includes("-") ? " (beta)" : "";

write("privacy/index.html", page("Peron — Gizlilik / Privacy", markdown(readFileSync(join(root, "PRIVACY.md"), "utf8"))));
write("support/index.html", page("Peron — Destek / Support", markdown(readFileSync(join(root, "SUPPORT.md"), "utf8"))));
write(
  "index.html",
  page(
    "Peron — Localhost port manager",
    `<h1>Peron ${esc(version)}${beta}</h1>
<p>Localhost'ta açık kalan portları, onları kimin açtığını ve ne kadar kaynak tükettiklerini gösterir; unutulanları tek tıkla kapatır.<br>
<span class="muted">Shows the ports left open on localhost, who opened them and what they consume — and closes the forgotten ones in one click.</span></p>

<div class="card"><h2>Windows 10/11</h2>
<a class="btn" href="https://apps.microsoft.com/detail/9MVV5PRQ9J6D">Microsoft Store</a>
<a class="btn" href="${asset(`Peron_${version}_x64-setup.exe`)}">Peron_${esc(version)}_x64-setup.exe</a>
<p class="muted">Store sürümü Microsoft tarafından imzalanır, otomatik güncellenir ve <a href="${repo}/blob/main/EULA.md">EULA</a> ile ücretsizdir. Doğrudan indirilen kurulum açık kaynaklıdır (<a href="${repo}/blob/main/LICENSE">GPL-3.0</a>) ve henüz imzalı değildir; SmartScreen uyarısında "Ek bilgi → Yine de çalıştır".<br>
The Store version is signed by Microsoft, updates automatically and is free under the <a href="${repo}/blob/main/EULA.md">EULA</a>. The direct installer is open source (<a href="${repo}/blob/main/LICENSE">GPL-3.0</a>) and isn't code-signed yet; on the SmartScreen prompt choose "More info → Run anyway".</p></div>

<div class="card"><h2>Linux x64</h2>
<a class="btn" href="${asset(`Peron_${version}_amd64.deb`)}">.deb</a>
<a class="btn" href="${asset(`Peron-${version}-1.x86_64.rpm`)}">.rpm</a>
<a class="btn" href="${asset(`Peron_${version}_amd64.AppImage`)}">.AppImage</a></div>

<p class="muted">Tüm sürümler / All releases: <a href="${repo}/releases">${repo}/releases</a> · <a href="${repo}/blob/main/CHANGELOG.md">Changelog</a></p>`,
  ),
);
