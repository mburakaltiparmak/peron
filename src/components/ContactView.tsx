import { useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { toAppError } from "../api";
import {
  canSendDirect,
  DEVELOPER,
  FEEDBACK_TO,
  type FeedbackInput,
  type FeedbackKind,
  MESSAGE_MAX,
  MESSAGE_MIN,
  openInMailClient,
  PRIVACY_URL,
  SUPPORT_URL,
  sendFeedback,
  validate,
  type ValidationError,
} from "../feedback";
import { prereleaseLabel } from "../format";
import { CoffeeIcon } from "./Icons";
import { errorText, useT } from "../i18n";
import type { AppInfo } from "../types";

interface Props {
  info: AppInfo;
  lang: string;
  onSent: (text: string) => void;
}

export function ContactView({ info, lang, onSent }: Props) {
  const t = useT();
  const openedAt = useRef(Date.now());
  const [kind, setKind] = useState<FeedbackKind>("bug");
  const [message, setMessage] = useState("");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [website, setWebsite] = useState(""); // honeypot
  const [sysInfo, setSysInfo] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Direct send failed (offline, service down, limit reached): offer the mail app instead.
  const [offerMail, setOfferMail] = useState(false);

  const kinds: { value: FeedbackKind; label: string }[] = [
    { value: "bug", label: t.kindBug },
    { value: "idea", label: t.kindIdea },
    { value: "other", label: t.kindOther },
  ];

  const validationText = (v: ValidationError) =>
    ({
      tooFast: t.valTooFast,
      tooShort: t.valTooShort(MESSAGE_MIN),
      tooLong: t.valTooLong(MESSAGE_MAX),
      tooManyLinks: t.valTooManyLinks,
      badEmail: t.valBadEmail,
    })[v];

  const input = (): FeedbackInput => ({
    kind,
    kindLabel: kinds.find((k) => k.value === kind)!.label,
    name,
    email,
    message,
    website,
    openedAt: openedAt.current,
    systemInfo: sysInfo ? `Peron ${info.version} · ${info.os} · ${lang}` : null,
  });

  const submit = async () => {
    const f = input();
    const invalid = validate(f);
    if (invalid) {
      setError(validationText(invalid));
      return;
    }
    setBusy(true);
    setError(null);
    setOfferMail(false);
    try {
      if (canSendDirect) {
        await sendFeedback(f);
        onSent(t.sent);
      } else {
        await openInMailClient(f);
      }
      setMessage("");
      openedAt.current = Date.now();
    } catch (e) {
      const appErr = toAppError(e);
      // fetch() rejects with a TypeError when the service can't be reached at all.
      setError(
        appErr.kind === "rateLimited"
          ? errorText(t, appErr)
          : e instanceof TypeError
            ? t.sendUnreachable
            : t.sendFailed(errorText(t, appErr)),
      );
      setOfferMail(canSendDirect);
    } finally {
      setBusy(false);
    }
  };

  const sendByMail = async () => {
    try {
      await openInMailClient(input());
      setError(null);
      setOfferMail(false);
    } catch (e) {
      setError(t.sendFailed(errorText(t, toAppError(e))));
    }
  };

  return (
    <div className="contact">
      <section className="card contact-form">
        <h2>{t.contactTitle}</h2>
        <p className="muted">{t.contactIntro}</p>

        <div className="field-col">
          <span className="label">{t.fieldKind}</span>
          <div className="segmented">
            {kinds.map((k) => (
              <button key={k.value} className={kind === k.value ? "active" : ""} aria-pressed={kind === k.value} onClick={() => setKind(k.value)}>
                {k.label}
              </button>
            ))}
          </div>
        </div>

        <label className="field-col">
          <span className="label">{t.fieldMessage}</span>
          <textarea
            rows={8}
            maxLength={MESSAGE_MAX}
            placeholder={t.messagePlaceholder}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
          />
          <span className="muted small counter">{t.charCount(message.length, MESSAGE_MAX)}</span>
        </label>

        <div className="row2">
          <label className="field-col">
            <span className="label">{t.fieldName}</span>
            <input type="text" value={name} maxLength={80} onChange={(e) => setName(e.target.value)} />
          </label>
          <label className="field-col">
            <span className="label">{t.fieldEmail}</span>
            <input type="email" value={email} maxLength={120} onChange={(e) => setEmail(e.target.value)} />
          </label>
        </div>

        {/* Honeypot: invisible to people and screen readers; bots tend to fill every field. */}
        <div className="hp" aria-hidden="true">
          <label>
            Website
            <input type="text" tabIndex={-1} autoComplete="off" value={website} onChange={(e) => setWebsite(e.target.value)} />
          </label>
        </div>

        <label className="check">
          <input type="checkbox" checked={sysInfo} onChange={(e) => setSysInfo(e.target.checked)} />
          {t.includeSysInfo}
          {sysInfo && <span className="muted small">({info.version} · {info.os})</span>}
        </label>

        {!canSendDirect && <p className="note">{t.noKeyNote}</p>}
        {error && (
          <div className="error" role="alert">
            {error}
            {offerMail && (
              <>
                {" "}
                <button className="link inline" onClick={sendByMail}>{t.sendViaMail}</button>
              </>
            )}
          </div>
        )}

        <div className="form-actions">
          <span className="muted small">
            {t.privacyNote}{" "}
            <button className="link inline" onClick={() => openUrl(PRIVACY_URL)}>{t.privacyPolicy}</button>
          </span>
          <button className="primary" onClick={submit} disabled={busy || message.trim().length === 0}>
            {busy ? t.sending : canSendDirect ? t.send : t.sendViaMail}
          </button>
        </div>
      </section>

      <aside className="card about">
        <img src="/logo.png" alt="" width={56} height={56} />
        <h2>
          Peron {prereleaseLabel(info.version) && <span className="beta-badge">{prereleaseLabel(info.version)}</span>}
        </h2>
        <p className="muted small">{t.version(info.version)}</p>
        <p>{t.aboutText}</p>
        <dl>
          <dt>{t.developer}</dt>
          <dd>
            <button className="link" onClick={() => openUrl(DEVELOPER.url)}>{DEVELOPER.name}</button>
          </dd>
          <dt>{t.website}</dt>
          <dd>
            <button className="link" onClick={() => openUrl(DEVELOPER.url)}>{DEVELOPER.url.replace("https://", "")}</button>
          </dd>
          <dt>GitHub</dt>
          <dd>
            <button className="link" onClick={() => openUrl(DEVELOPER.github)}>
              {DEVELOPER.github.replace("https://www.", "")}
            </button>
          </dd>
          <dt>{t.email}</dt>
          <dd>
            <button className="link" onClick={() => openUrl(`mailto:${FEEDBACK_TO}`)}>{FEEDBACK_TO}</button>
          </dd>
        </dl>
        <button className="coffee" onClick={() => openUrl(DEVELOPER.coffee)}>
          <CoffeeIcon /> {t.buyMeACoffee}
        </button>
        <p className="muted small">{t.buyMeACoffeeNote}</p>
        <p className="about-links">
          <button className="link" onClick={() => openUrl(SUPPORT_URL)}>{t.support}</button>
          <span aria-hidden="true"> · </span>
          <button className="link" onClick={() => openUrl(PRIVACY_URL)}>{t.privacyPolicy}</button>
        </p>
      </aside>
    </div>
  );
}
