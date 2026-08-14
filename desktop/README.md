# Curio desktop

The desktop product is no longer a separate frontend package. Its Tauri shell
lives at `web/curio/src-tauri`, and it embeds the shared React application from
`web/curio/src`.

Run it from `web/curio` with `npm run dev:desktop` or build it with
`npm run build:desktop`.
