/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CURIO_CHAT_WORKER_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare const __CURIO_CHAT_WORKER_URL__: string;
