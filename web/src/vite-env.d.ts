/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CURIO_SERVICE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare const __CURIO_SERVICE_URL__: string;
