// "New version available" notice for direct-download builds (site / GitHub). The Microsoft Store
// build never checks: the Store updates it. Checks at most once a day, only downloads a small
// JSON list of releases, and never downloads or installs anything itself.

export const RELEASES_API = "https://api.github.com/repos/mburakaltiparmak/peron/releases?per_page=10";
export const RELEASES_PAGE = "https://github.com/mburakaltiparmak/peron/releases";
const CHECK_EVERY_MS = 24 * 60 * 60 * 1000;
const KEY_CHECKED = "peron.updateCheckedAt";
const KEY_DISMISSED = "peron.updateDismissed";

interface Parsed {
  core: number[];
  pre: (string | number)[];
}

function parse(v: string): Parsed | null {
  const m = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?/.exec(v.trim());
  if (!m) return null;
  const pre = m[4] ? m[4].split(".").map((p) => (/^\d+$/.test(p) ? Number(p) : p)) : [];
  return { core: [Number(m[1]), Number(m[2]), Number(m[3])], pre };
}

/** SemVer 2.0 precedence: <0 if a<b, 0 if equal, >0 if a>b. Unparsable versions sort lowest. */
export function compareVersions(a: string, b: string): number {
  const pa = parse(a);
  const pb = parse(b);
  if (!pa || !pb) return pa ? 1 : pb ? -1 : 0;
  for (let i = 0; i < 3; i++) if (pa.core[i] !== pb.core[i]) return pa.core[i] - pb.core[i];
  if (!pa.pre.length || !pb.pre.length) return pb.pre.length - pa.pre.length; // release > pre-release
  for (let i = 0; i < Math.max(pa.pre.length, pb.pre.length); i++) {
    const x = pa.pre[i];
    const y = pb.pre[i];
    if (x === undefined) return -1;
    if (y === undefined) return 1;
    if (x === y) continue;
    if (typeof x === "number" && typeof y === "number") return x - y;
    if (typeof x === "number") return -1; // numeric identifiers sort before alphanumeric
    if (typeof y === "number") return 1;
    return x < y ? -1 : 1;
  }
  return 0;
}

export interface Release {
  tag_name: string;
  html_url: string;
  draft: boolean;
  prerelease: boolean;
}

export interface Update {
  version: string;
  url: string;
}

/** Newest release above `current`. Pre-releases count only while `current` is a pre-release. */
export function pickUpdate(current: string, releases: Release[]): Update | null {
  const onPre = (parse(current)?.pre.length ?? 0) > 0;
  const candidates = releases.filter((r) => !r.draft && (onPre || !r.prerelease) && parse(r.tag_name));
  candidates.sort((a, b) => compareVersions(b.tag_name, a.tag_name));
  const best = candidates[0];
  if (!best || compareVersions(best.tag_name, current) <= 0) return null;
  return { version: best.tag_name.replace(/^v/, ""), url: best.html_url };
}

function storageGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function storageSet(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* storage unavailable: we'll just check again next time */
  }
}

/** Returns an update to announce, or null (not due yet, none newer, dismissed, or offline). */
export async function checkForUpdate(current: string, now = Date.now()): Promise<Update | null> {
  const last = Number(storageGet(KEY_CHECKED) ?? 0);
  if (now - last < CHECK_EVERY_MS) return null;
  try {
    const res = await fetch(RELEASES_API, { headers: { Accept: "application/vnd.github+json" } });
    if (!res.ok) return null;
    storageSet(KEY_CHECKED, String(now));
    const update = pickUpdate(current, (await res.json()) as Release[]);
    return update && storageGet(KEY_DISMISSED) !== update.version ? update : null;
  } catch {
    return null; // offline or GitHub unreachable: the app keeps working, try again tomorrow
  }
}

export function dismissUpdate(version: string) {
  storageSet(KEY_DISMISSED, version);
}
