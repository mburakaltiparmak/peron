// Feedback delivery via Web3Forms. No SMTP credentials in the app: the access key can only
// deliver mail to the account owner, so shipping it in the bundle is acceptable.
// Bot protection (client side): honeypot, minimum fill time, content checks, and a persistent
// send limit enforced in Rust (`feedback_reserve`). Server side: Web3Forms spam filter + botcheck.
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";

const ACCESS_KEY = import.meta.env.WEB3FORMS_FORM_ACCESS_KEY?.trim() || "";
export const FEEDBACK_TO = import.meta.env.WEB3FORMS_FORM_TO_MAIL?.trim() || "mburakaltiparmak@gmail.com";
export const DEVELOPER = {
  name: "M. Burak Altıparmak",
  url: "https://burakaltiparmak.dev",
  github: "https://www.github.com/mburakaltiparmak",
  /** Voluntary tips; nothing in the app is unlocked by them (Microsoft Store policy 10.8.2). */
  coffee: "https://buymeacoffee.com/mburakaltiparmak",
};

/** Public pages (also mirrored in the GitHub repo as PRIVACY.md / SUPPORT.md). */
export const PRIVACY_URL = "https://burakaltiparmak.dev/products/peron/privacy";
export const SUPPORT_URL = "https://burakaltiparmak.dev/contact";

/** Without a key (e.g. a fork built without .env) the form falls back to the mail client. */
export const canSendDirect = ACCESS_KEY.length > 0;

export const MIN_FILL_MS = 3000;
export const MESSAGE_MIN = 10;
export const MESSAGE_MAX = 5000;
const MAX_LINKS = 3;

export type FeedbackKind = "bug" | "idea" | "other";

export interface FeedbackInput {
  kind: FeedbackKind;
  kindLabel: string;
  name: string;
  email: string;
  message: string;
  systemInfo: string | null;
  /** Honeypot: hidden from humans; any value means a bot filled the form. */
  website: string;
  openedAt: number;
}

export type ValidationError = "tooFast" | "tooShort" | "tooLong" | "tooManyLinks" | "badEmail";

export function validate(f: FeedbackInput, now = Date.now()): ValidationError | null {
  if (now - f.openedAt < MIN_FILL_MS) return "tooFast";
  const msg = f.message.trim();
  if (msg.length < MESSAGE_MIN) return "tooShort";
  if (msg.length > MESSAGE_MAX) return "tooLong";
  if ((msg.match(/https?:\/\//gi) ?? []).length > MAX_LINKS) return "tooManyLinks";
  if (f.email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(f.email.trim())) return "badEmail";
  return null;
}

function compose(f: FeedbackInput) {
  const firstLine = f.message.trim().split("\n")[0].slice(0, 60);
  const subject = `[Peron] ${f.kindLabel}: ${firstLine}`;
  const body = [
    f.message.trim(),
    "",
    "---",
    f.name.trim() && `Ad / Name: ${f.name.trim()}`,
    f.email.trim() && `Yanıt / Reply-to: ${f.email.trim()}`,
    f.systemInfo,
  ]
    .filter(Boolean)
    .join("\n");
  return { subject, body };
}

/** Sends directly via Web3Forms. Honeypot hits resolve silently without sending. */
export async function sendFeedback(f: FeedbackInput): Promise<void> {
  if (f.website) return; // bot: pretend success, send nothing
  await invoke("feedback_reserve"); // throws { kind: "rateLimited" } when over the limit
  const { subject, body } = compose(f);
  const res = await fetch("https://api.web3forms.com/submit", {
    method: "POST",
    headers: { "Content-Type": "application/json", Accept: "application/json" },
    body: JSON.stringify({
      access_key: ACCESS_KEY,
      subject,
      from_name: f.name.trim() || "Peron",
      email: f.email.trim() || undefined,
      message: body,
      botcheck: false,
    }),
  });
  const data = (await res.json().catch(() => null)) as { success?: boolean; message?: string } | null;
  if (!res.ok || !data?.success) throw new Error(data?.message || `HTTP ${res.status}`);
}

/** Fallback: open the user's mail client with the message prefilled. */
export function openInMailClient(f: FeedbackInput) {
  const { subject, body } = compose(f);
  return openUrl(`mailto:${FEEDBACK_TO}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`);
}
