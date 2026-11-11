/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_USUYO_API?: string;
  /** "1" 时不找内核，只用样例库。 */
  readonly VITE_USUYO_MOCK?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
