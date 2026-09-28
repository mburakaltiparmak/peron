import { useEffect, useRef, useState } from "react";
import { useT } from "../i18n";
import type { ExportFormat } from "../types";
import { DownloadIcon } from "./Icons";

interface Props {
  disabled?: boolean;
  onExport: (format: ExportFormat, save: boolean) => void;
}

/** "Export" dropdown: save TXT/CSV/JSON to Downloads, or copy the table to the clipboard. */
export function ExportMenu({ disabled, onExport }: Props) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !root.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);

  const pick = (format: ExportFormat, save: boolean) => {
    setOpen(false);
    onExport(format, save);
  };

  return (
    <div className="menu-root" ref={root}>
      <button aria-haspopup="menu" aria-expanded={open} disabled={disabled} onClick={() => setOpen((o) => !o)} title={t.exportTitle}>
        <DownloadIcon size={15} /> {t.export}
      </button>
      {open && (
        <div className="menu" role="menu">
          <button role="menuitem" onClick={() => pick("txt", true)}>{t.exportAs("TXT")}</button>
          <button role="menuitem" onClick={() => pick("csv", true)}>{t.exportAs("CSV")}</button>
          <button role="menuitem" onClick={() => pick("json", true)}>{t.exportAs("JSON")}</button>
          <hr />
          <button role="menuitem" onClick={() => pick("txt", false)}>{t.copyTable}</button>
        </div>
      )}
    </div>
  );
}
