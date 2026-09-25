---
name: code-verification
description: Verify Curio changes and report evidence across the React/Vite client, Tauri Rust shell, Axum service, PostgreSQL migrations, legacy Workers, documentation, or project skills. Use for implementation completion, regression checks, PR readiness, reviews, or when deciding the smallest trustworthy test matrix. Do not use to claim deployment or production health without an explicit live check.
---

# Curio code verification

Build a verification matrix from the actual diff and architecture boundaries.
Read [`AGENTS.md`](../../../AGENTS.md), the relevant section of the canonical
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md), package manifests, CI,
and nearby tests before choosing commands. The repository has no root npm or
Cargo workspace, so always run a command from its owning package root.

## Establish scope and baseline

Start with read-only evidence:

```bash
git status --short --branch
git diff --check
git diff --name-only
```

Treat pre-existing working-tree changes as user work. Do not revert, reformat, stage, or include unrelated files. For behavior-preserving refactors or suspected regressions, run the narrowest relevant check before editing when practical so later failures can be attributed honestly.

Map changed paths to affected contracts, not only to their language:

| Changed area | Verify at minimum |
| --- | --- |
| `web/src` ordinary UI/domain code | focused Vitest, lint; add architecture gate when imports/network boundaries could be affected |
| routes, aliases, platform, API, chat, config | architecture gate, focused tests, lint, both web and desktop UI builds |
| styling or interaction | relevant automated checks plus browser/runtime inspection at useful viewport and keyboard states |
| `web/src-tauri` | Rust format, clippy, unit tests; rebuild desktop UI/package when contract or packaging changes |
| `curio-service` pure logic | focused Rust test, format, clippy |
| service router/auth/contracts | unit/full-router tests plus relevant client contract tests |
| repositories or PostgreSQL migrations | database-backed tests |
| `curio-workers` | that Worker's typecheck and Vitest; remember root CI does not cover it |
| docs or `.agents/skills` only | link/path/frontmatter review, diff check, and skill validation when available |
| cross-stack or CI/config changes | every affected package plus the workflow-equivalent command set |

Expand the matrix when a shared boundary has more consumers. A green leaf test does not prove a changed protocol, migration, target alias, or build configuration.

## Frontend checks

Run from `web` with npm and the committed `package-lock.json`:

```bash
npm run check:architecture
npm run lint
npm test
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:web
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:desktop-ui
```

For fast feedback, pass a focused test path through the existing script before the full suite, for example `npm test -- tests/features/chat/chat-stream.test.ts`.

The build scripts run strict project TypeScript builds before Vite. Production builds require an explicit HTTPS `VITE_CURIO_SERVICE_URL`; the example host is suitable only for an offline artifact check and is not evidence that a deployed service is reachable. Do not treat `vite preview` as production hosting proof.

For observable UI changes, run the appropriate web or desktop development target and inspect:

- success plus the most relevant loading, empty, error, disabled, cancellation, and retry state;
- console warnings and request failures;
- keyboard order, focus, labels, menus/dialogs, and escape behavior;
- narrow and wide layouts, zoom/reflow, and reduced motion when relevant;
- direct web deep-link reloads or desktop hash routing when routes change;
- both targets when build-selected behavior or shared transport code changes.

Stop temporary servers after verification.

## Tauri Rust checks

Run from `web/src-tauri`:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Unit tests prove core URL/path/token/cancellation behavior but do not launch a real WebView. Changes to commands, channels, CSP, capabilities, service URL compilation, or packaging need a desktop runtime or package smoke test in addition to unit tests. `npm run build:desktop` runs from `web` and may require platform build tools and signing decisions; report when it was not practical.

## Axum service checks

Run from `curio-service`:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

`CURIO_TEST_DATABASE_URL` must identify a non-production test-runner role that can create and drop only `curio_test_*` schemas. Database-backed tests return early when this variable is absent, so a successful `cargo test` without the variable does not prove PostgreSQL behavior. Record whether the variable was present without printing its value.

Run migration/readiness tools only against an explicitly safe target:

```bash
cargo run --bin curio_db -- verify
```

Migration application is a state-changing operation. Do not
run it merely as verification against an existing database. If the task
explicitly requires it, resolve the exact schema and credential role first,
use an isolated disposable target or approved environment, run a dry run where
supported, and follow
[`docs/deployment.md`](../../../docs/deployment.md).

For an HTTP/SSE contract change, verify status, headers, JSON/event shape, fragmented streaming behavior, terminal state, persistence order, sanitized failures, cancellation/disconnect, and the corresponding TypeScript client/parser.

## Legacy Worker checks

The two Cloudflare Worker packages are independent and outside root CI. From the changed Worker package:

```bash
npm run typecheck
npm test -- --run
```

Use Wrangler/local bindings only when the task requires runtime behavior. Do not assume the active React or Axum system calls these Workers; they are retained migration/reference code.

## Documentation and skill checks

For Markdown and project skills:

- verify every documented path, command, environment variable, endpoint, and architectural claim against source;
- search for stale links after moves or renames;
- ensure there is only one canonical architecture document at `docs/ARCHITECTURE.md`;
- require every feature/fix handoff to include an architecture-impact result;
  verify that the canonical document changed when current behavior, boundaries,
  contracts, persistence, configuration, limitations, or checks changed;
- ensure each `SKILL.md` has valid YAML with matching lowercase-hyphen folder/name and a discriminating `description`;
- run the Codex skill `quick_validate.py` helper when it is available, once per skill folder;
- inspect the rendered structure enough to catch broken fences, tables, and relative links.

Do not add generated documentation, copied dependency manuals, or tests that merely assert exact prose.

## Finish with evidence

Re-run `git diff --check`, inspect the final diff and `git status`, and confirm no generated outputs or unrelated files entered the change. Report:

- exact commands and meaningful manual interactions;
- pass, fail, skip, or partial outcome for each;
- required environment that was absent, without exposing secrets;
- failures known to predate the change only when a baseline or other concrete evidence proves it;
- residual risk that automated checks cannot cover.
- `Architecture impact: updated` with the affected sections, or
  `Architecture impact: none` when all existing claims remain accurate.

A skipped, canceled, truncated, environment-gated, or unrun check is not a pass. Build success does not by itself prove runtime behavior, accessibility, production routing, deployment, or database compatibility.
