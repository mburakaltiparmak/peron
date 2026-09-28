/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Web3Forms access key (from .env at build time; see .env.example). */
  readonly WEB3FORMS_FORM_ACCESS_KEY?: string;
  /** Feedback recipient shown in the app and used for the mailto fallback. */
  readonly WEB3FORMS_FORM_TO_MAIL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
