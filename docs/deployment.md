# Deployment notes

## Curio service

`curio-service` is the Axum backend both clients target. Run it with
`cargo run` from `curio-service`; it binds `CURIO_SERVICE_ADDR` (default
`127.0.0.1:3000`) and requires `OPENAI_API_KEY` and `OPENAI_MODEL`.
`CURIO_DATABASE_URL` selects the SQLite database (default `sqlite://curio.db`;
migrations run automatically at startup).

Set `CURIO_CORS_ALLOWED_ORIGINS` to a comma-separated list of exact deployed web
origins, for example `https://curio.example.com`. When the variable is absent,
only the common local Vite development origins (ports 5173 and 1420) are
allowed. Desktop builds do not need a CORS entry because their requests
originate from the Rust process, not a browser origin. Never put
`OPENAI_API_KEY` in a Vite variable; it belongs only in the service
environment.

## Web

Build the static SPA with `npm run build:web` from `web` and publish
`web/dist`. The host must rewrite unknown application paths such as
`/chat`, `/vault`, and `/settings` to `index.html`; asset requests should still
return normal 404 responses.

`VITE_CURIO_SERVICE_URL` is public build-time configuration pointing at the
deployed Curio service. Production builds fail unless it is an explicit HTTPS
URL, so a packaged artifact cannot silently point at localhost. A template is
provided in `.env.production.example`.

For a deliberately localhost-targeted debug package, set
`CURIO_ALLOW_INSECURE_LOCAL_BUILD=1` while running `npm run build:desktop --
--debug`. This opt-in must not be used for release artifacts.

## Desktop

`npm run build:desktop` builds the same React source in desktop mode and invokes
the Tauri bundler. The service URL is compiled into the Rust bridge; the desktop
renderer passes only relative service paths, JSON payloads, and an optional
bearer token. Users do not need `VITE_CURIO_SERVICE_URL` in their shell.

Release distribution still requires platform-specific credentials and policy:

- macOS: configure an Apple Developer signing identity, notarization credentials,
  hardened runtime/entitlements, and staple the notarization result.
- Windows: configure an Authenticode certificate and timestamp service.
- Linux: build the desired deb/AppImage/RPM targets in compatible build images.
- Auto-update should be introduced as a separate capability with signed update
  metadata; it is not enabled by this consolidation.

The renderer has no direct opener capability. An app-owned Rust command validates
HTTPS links before opening them, and the CSP permits application assets, Tauri
IPC, and local development HMR. Review both whenever a native plugin or remote
resource is added.
