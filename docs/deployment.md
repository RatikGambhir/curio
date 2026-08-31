# Deployment notes

See the canonical [architecture guide](ARCHITECTURE.md) for runtime boundaries,
contracts, and current limitations.

## Curio service

`curio-service` is the Axum backend both clients target. PostgreSQL is its only
runtime database; version 18 is the current Railway/CI target, not a major
version enforced by the service connection. The service does not apply schema
changes at startup: run the migration command before starting a new application
release.

Database credentials are deliberately separated:

| Variable | Used by | Required access |
| --- | --- | --- |
| `DATABASE_URL` | Running service and verification | Application role with DML access only |
| `CURIO_MIGRATOR_DATABASE_URL` | `curio_db migrate` and the SQLite importer | Migrator role that owns the selected schema |
| `CURIO_DB_SCHEMA` | Both database clients | `curio_dev`, `curio_prod`, or a disposable `curio_test_*` schema |
| `CURIO_TEST_DATABASE_URL` | Integration-test fixture | Test runner allowed to create/drop only `curio_test_*` schemas |

URLs contain credentials and must stay in local or deployment secret stores.
Percent-encode passwords when constructing a URL. Never expose any database URL
as a `VITE_*` variable, log it, or give an application process the Railway
administrator credential.

For local development, copy `.env.example` to `.env`, use the development app
and migrator roles, and select `curio_dev`. Railway's private hostname resolves
only inside the project. A local or externally hosted service therefore needs
Railway Public Access/TCP Proxy enabled and a TLS URL ending in
`?sslmode=require` (or an approved private tunnel). Apply migrations and start
the service separately:

```bash
cd curio-service
cargo run --bin curio_db -- migrate
cargo run --bin curio-service
```

The service binds `CURIO_SERVICE_ADDR` (default `127.0.0.1:3000`) and requires
`OPENAI_API_KEY` and `OPENAI_MODEL`. Pool sizing is controlled by
`CURIO_DB_MAX_CONNECTIONS` and `CURIO_DB_ACQUIRE_TIMEOUT_SECONDS`; keep the sum
of every replica's pool below the database connection limit.

Set `CURIO_CORS_ALLOWED_ORIGINS` to a comma-separated list of exact deployed web
origins, for example `https://curio.example.com`. When the variable is absent,
only the common local Vite development origins (ports 5173 and 1420) are
allowed. Desktop builds do not need a CORS entry because their requests
originate from the Rust process, not a browser origin. Never put
`OPENAI_API_KEY` in a Vite variable; it belongs only in the service
environment.

### Railway application service

The repository includes `curio-service/railway.toml`. Configure the Railway
service with repository root directory `/curio-service` so Railpack finds the
crate and that configuration file. The configuration:

- runs `cargo run --release --bin curio_db -- migrate` before deployment;
- starts the Axum service on `0.0.0.0:$PORT`;
- gates rollout on the database-backed `/ready` endpoint; and
- restarts the process only after failures.

Set `DATABASE_URL` to a private-network DSN for `curio_prod_app`, set
`CURIO_MIGRATOR_DATABASE_URL` to the corresponding private-network migrator
DSN, and set `CURIO_DB_SCHEMA=curio_prod`. The built-in
`${{Postgres.DATABASE_URL}}` points at Railway's administrative database user;
do not pass it directly to the application. Construct the two role-specific
DSNs from the private Postgres host/database values and passwords stored as
Railway secrets.

The pre-deploy command must fail the release when a migration fails. `/health`
remains a process-liveness check; `/ready` must also confirm database access and
the expected migration version. Keep the migrator credential out of the
running application's code paths even though Railway makes the variable
available to the pre-deploy container.

Before a production migration or import, verify the target database/user/schema,
take a restorable backup, and run the redacted checks in
`curio-service/ops/postgres/verify.sql`. Do not import into a non-empty or
unknown schema and do not remove the final SQLite snapshot during the rollback
window.

## Web

Build the static SPA with `npm run build:web` from `web` and publish
`web/dist`. The host must rewrite unknown application paths such as
`/chat`, `/vault`, and `/settings` to `index.html`; asset requests should still
return normal 404 responses.

`VITE_CURIO_SERVICE_URL` is public build-time configuration pointing at the
deployed Curio service. Production builds fail unless it is an explicit HTTPS
URL, so a packaged artifact cannot silently point at localhost. `build:web`
uses Vite mode `web`, so provide the value through the build process environment
or a common/`.env.web*` file. The existing `.env.production.example` is a value
template; Vite does not load it automatically for this script.

For a deliberately localhost-targeted debug package, set
`CURIO_ALLOW_INSECURE_LOCAL_BUILD=1` while running `npm run build:desktop --
--debug`. This opt-in must not be used for release artifacts.

## Desktop

`npm run build:desktop` builds the same React source in desktop mode and invokes
the Tauri bundler. The service URL is compiled into the Rust bridge; the desktop
renderer passes only relative service paths, JSON payloads, and an optional
bearer token. Release CI must export `VITE_CURIO_SERVICE_URL` to the full Tauri
build process so both Vite and Cargo see it; a `.env.desktop` value loaded by
Vite is not automatically exported to Rust compilation. Users do not need the
variable in their shell after installation.

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
