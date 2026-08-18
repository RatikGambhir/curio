# Curio desktop

The desktop product is no longer a separate frontend package. Its Tauri shell
lives at `web/src-tauri`, and it embeds the shared React application from
`web/src`.

Run it from `web` with `npm run dev:desktop` or build it with
`npm run build:desktop`.
