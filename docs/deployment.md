# Deployment notes

## Web

Build the static SPA with `npm run build:web` from `web` and publish
`web/dist`. The host must rewrite unknown application paths such as
`/chat`, `/vault`, and `/settings` to `index.html`; asset requests should still
return normal 404 responses.

`VITE_CURIO_CHAT_WORKER_URL` is public build-time configuration. Production
builds fail unless it is an explicit HTTPS URL, so a packaged desktop app cannot
silently point at localhost or a guessed Workers hostname. A template is
provided in `.env.production.example`.

For a deliberately localhost-targeted debug package, set
`CURIO_ALLOW_INSECURE_LOCAL_BUILD=1` while running `npm run build:desktop --
--debug`. This opt-in must not be used for release artifacts.

Set the chat worker's `CURIO_ALLOWED_ORIGINS` variable to a comma-separated list
of exact deployed web origins, for example `https://curio.example.com`. When the
variable is absent, only the common local Vite/Tauri development origins are
allowed. Never put `GEMINI_API_KEY` in a Vite variable; it belongs only in
Cloudflare Worker secrets.

## Desktop

`npm run build:desktop` builds the same React source in desktop mode and invokes
the Tauri bundler. The worker URL is compiled into the frontend artifact; users
do not need `CURIO_CHAT_WORKER_URL` in their shell.

Release distribution still requires platform-specific credentials and policy:

- macOS: configure an Apple Developer signing identity, notarization credentials,
  hardened runtime/entitlements, and staple the notarization result.
- Windows: configure an Authenticode certificate and timestamp service.
- Linux: build the desired deb/AppImage/RPM targets in compatible build images.
- Auto-update should be introduced as a separate capability with signed update
  metadata; it is not enabled by this consolidation.

The current Tauri capability permits only scoped HTTPS external URL opening, and
the CSP permits application assets, Tauri IPC, and local development HMR. Review
both whenever a native plugin or remote resource is added.
