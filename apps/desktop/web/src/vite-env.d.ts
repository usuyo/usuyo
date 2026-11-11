/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_USUYO_API?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
