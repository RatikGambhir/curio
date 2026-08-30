# Email + password auth plan (JWT)

Linear: [RAT-31](https://linear.app/ratik-gambhir/issue/RAT-31/email-password-auth-register-login-signup-form-ui)
Signup component: <https://ui.ilinxa.com/components/signup-form>

Replace Curio's mock authentication with real email + password auth: a
`POST /v1/auth/register` endpoint that hashes passwords with argon2id, a
`POST /v1/auth/login` endpoint that issues a **JWT**, and a rebuilt signup/login
UI on the ilinxa `signup-form` component — including a Google button that is
present but not yet wired.

## What exists today

**Authentication is entirely fake.** This is not a "wire up the real backend"
task — there is no auth to wire up, and the current code actively pretends there
is.

`web/src/lib/providers/auth-provider.tsx`:

```tsx
const loginUser = useCallback((email: string) => {
  const mockUser = buildMockUser(email.trim().toLowerCase())
  window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(mockUser))
  setUser(mockUser)
}, [])
```

- `loginUser` takes **only an email**. There is no password anywhere in the
  frontend.
- `buildMockUser` (`web/src/features/auth/storage.ts`) synthesises a user from
  the email's local part. The storage key is literally `curio-mock-auth-user-v1`.
- No network request is made. Typing any email into `/login` grants access.
- `RequireAuth` in `web/src/app/router.tsx` gates every authenticated route on
  this localStorage value, so clearing or forging one key is the entire access
  control model.

The `users` table (`curio-service/migrations/0002_users.sql`) has `id`, `name`,
`email`, `avatar_url`, and timestamps — **no password column**.

## Security defects this work must close

These are live in `main` today. They are in scope because the auth work
necessarily rewrites the code containing them.

**1. Any non-empty bearer token authenticates.** `curio-service/src/lib.rs`:

```rust
async fn authorize_current_user(auth_header: &str) -> Option<CurrentUser> {
    auth_header
        .strip_prefix("Bearer ")
        .filter(|token| !token.trim().is_empty())
        .map(|_| CurrentUser)
}
```

`Bearer x` passes. The service's own tests use `Bearer development-token`.

**2. `CurrentUser` carries no identity.** It is a unit struct
(`pub struct CurrentUser;`). Every handler that takes it binds it as
`Extension(_current_user)` and ignores it, because there is nothing to read.

**3. `save_user` trusts the request body for identity.**
`curio-service/src/user/commands.rs` reads `id` from `SaveUserRequest` and
upserts on it. Combined with (1) and (2), any caller with a syntactically valid
bearer header can overwrite any user row by sending that user's id. Fixing this
means the handler must take the id from the authenticated principal, not the
payload.

**4. Chat routes are entirely unauthenticated.** In `app_with_config`,
`chat_routes` is merged onto `base_router()`, outside the `protected_routes`
group that carries `route_layer(middleware::from_fn(auth))`. Only `/user/*` and
`/v1/users` are behind the middleware. The existing test
`chat_route_normalizes_openai_streams` sends no `Authorization` header and
asserts a 200 — so this is intentional today, not an oversight, and closing it
will require updating that test.

Whether to put chat behind auth in this ticket is a scoping decision. The
recommendation is yes: once real auth exists, an unauthenticated endpoint that
spends OpenAI credits is a bill waiting to happen. If it is deferred, it should
be deferred explicitly rather than silently.

## Token strategy: JWT

Login returns a signed **JWT** that the client sends as `Authorization: Bearer
<token>` on every authenticated request. Use the `jsonwebtoken` crate with
**HS256** — one symmetric secret, no key distribution, appropriate for a single
service signing and verifying its own tokens.

**Claims.** Keep them minimal:

- `sub` — the user id. This is what `CurrentUser` will carry.
- `exp` — expiry, as a Unix timestamp. **Required.** A JWT without `exp` is a
  permanent credential.
- `iat` — issued-at.

Do not put the email, name, or anything else mutable in the claims. A JWT is a
snapshot: whatever is baked in stays stale until the token expires. Look mutable
data up from the database using `sub`.

**Never put the password or its hash in the token.** A JWT is signed, not
encrypted — the payload is base64, readable by anyone holding the token.

**Pin the algorithm on verification.** Construct
`Validation::new(Algorithm::HS256)` explicitly and verify with it. Accepting
whatever `alg` the token header claims is the classic JWT vulnerability class
(`alg: none`, or HS256-signed-with-the-public-key confusion). The `jsonwebtoken`
crate's defaults are sane here — the point is to set it deliberately rather than
inherit it and hope.

**Expiry.** Something on the order of 24 hours is reasonable for this app. The
tradeoff is the whole story of stateless tokens: short expiry means users
re-login often; long expiry means a stolen token stays useful for that long. No
refresh-token flow in this ticket — a deliberate scope choice for "basic auth,"
and the thing to add later if the expiry window proves annoying.

### The revocation tradeoff — read this before choosing JWT

**A JWT cannot be revoked.** Once signed, it is valid until `exp`, and the server
has no record of it to delete. Concretely, for this app:

- **Logout is client-side only.** `POST /v1/auth/logout` cannot invalidate
  anything; it exists so the client can clear its stored token. If someone copied
  the token before logout, it keeps working until it expires.
- **Password change does not lock out existing tokens.**
- **There is no "sign out all devices."**

The escape hatches, none of which are in scope here: a denylist table (which
reintroduces the database lookup JWT was avoiding), a per-user `token_version`
claim checked against the users row (cheap, and the usual first upgrade), or
short expiry plus refresh tokens.

This is a normal tradeoff for basic auth and matches the request — but it should
be a decision, not a surprise the first time someone asks why logout did not
actually log anyone out. If revocable logout matters more than statelessness,
opaque session tokens in a `sessions` table are the alternative.

## The signing secret

Add `jwt_secret` to `ServiceConfig` in `curio-service/src/config.rs`, loaded with
the existing `required()` helper:

```rust
jwt_secret: required("CURIO_JWT_SECRET")?,
```

**Use `required()`, not `env::var().unwrap_or_else(...)`.** `database_url`,
`openai_base_url`, and `cors_allowed_origins` all have development fallbacks; a
signing secret must not. A default secret that reaches production means anyone
who has read the source can mint valid tokens for any `sub` — the auth system
becomes decorative. `required()` already rejects unset *and* blank values and
returns a `ConfigError` that `main.rs` turns into a startup failure. That is
exactly the behaviour wanted: refuse to boot rather than boot insecure.

Generate a long random secret (32+ bytes). Add `CURIO_JWT_SECRET=` to
`curio-service/.env.example` under the "Required" heading alongside
`OPENAI_API_KEY`, with no value filled in.

Two follow-on details:

- `ServiceConfig` derives `Debug`. Any `{:?}` of it would print the secret (and
  the OpenAI key, which has the same exposure today). Either implement `Debug`
  manually to redact both, or make it a rule that config is never debug-printed.
- `test_config()` in `curio-service/src/lib.rs` constructs `ServiceConfig` with a
  struct literal, so adding a field breaks compilation there until it is updated.
  That is a feature — it forces every test path to supply a secret.

## Where the token lives on the client

`ApiRequestOptions` in `web/src/api/client.ts` already accepts `bearerToken` and
threads it into `ServiceRequestInit` — the seam exists and is currently unused.

The honest tradeoff: **a token in `localStorage` is readable by any XSS on the
origin.** An httpOnly cookie avoids that, but this app ships a Tauri desktop
target whose transport is an app-owned Rust `service_request` command that
forwards an explicit bearer token — cookies do not fit that path without separate
work. Given one shared `src/` for both targets, bearer-in-storage is the
consistent choice.

With JWT this matters slightly more than it would with revocable sessions,
because a stolen token cannot be invalidated. Keep the token out of URLs, logs,
and error payloads, and keep `exp` short enough that theft has a bounded window.

## Data model

New migration `curio-service/migrations/0003_auth.sql`. With JWT there is **no
sessions table** — that is the point of stateless tokens. The migration is just:

- Add `password_hash TEXT` to `users`. It must be **nullable**, because
  `0002_users.sql` may already have rows, and because SQLite's `ALTER TABLE ADD
  COLUMN` cannot add a `NOT NULL` column without a default. A NULL hash means
  "this account cannot log in with a password" — the login handler must reject
  those rather than treating NULL as a match. (This is also the shape a future
  Google-only account would take, so the NULL case is worth getting right now.)

`users_email_idx` already exists as a `UNIQUE INDEX` — registration's duplicate
detection can rely on it, and `save_user` already maps `is_unique_violation()` to
409.

Store emails normalized (trimmed, lowercased) on write. The existing unique index
is case-sensitive, so `A@b.com` and `a@b.com` would otherwise both register.

## Endpoints

**`POST /v1/auth/register`** — public (must NOT sit behind the auth middleware).

- Accepts email, password, and whatever name field the form is configured to
  collect.
- Validates: email shape, password minimum length. Server-side, independent of
  the client policy.
- Hashes with **argon2id** via the `argon2` crate, using its documented default
  parameters (OWASP-aligned) rather than hand-picked numbers. Never a fast hash
  (MD5/SHA/bcrypt-with-low-cost).
- Inserts the user; maps a unique violation to 409.
- Returns a JWT (auto-login) — fewer round trips, and it matches the component's
  `status: "success"` flow.

**`POST /v1/auth/login`** — public.

- Looks up the user by normalized email, verifies the password against the stored
  hash.
- **Uniform failure response.** Wrong email and wrong password must return the
  same status and the same message. Distinct errors turn the endpoint into a
  user-enumeration oracle.
- Also worth doing a dummy hash verification when the user is not found, so the
  response time does not leak existence.
- Signs and returns a JWT plus the user record.

**Note the enumeration tension:** register *has* to signal "email already taken"
for the form to be usable, which leaks existence regardless of how careful login
is. That is a normal, accepted tradeoff — but make it a decision, not an
accident. Rate-limit registration if it matters.

**`POST /v1/auth/logout`** — see the revocation section above. With stateless
JWT this endpoint cannot invalidate the token. Either omit it entirely and have
the client simply drop its stored token, or keep it as a no-op for symmetry. If
it is kept, **do not let its existence imply revocation** — comment it, so the
next reader does not assume tokens are being invalidated server-side.

**Middleware rewrite** — replace `authorize_current_user` so it decodes and
validates the JWT (signature, `exp`, pinned algorithm) and populates
`CurrentUser` with the `sub` claim. `CurrentUser` becomes
`struct CurrentUser { pub id: String }`.

The middleware needs the decoding key, which a bare `middleware::from_fn` closure
cannot capture — convert to `middleware::from_fn_with_state` (or an extractor)
carrying the key. Note that unlike a session-table design, it needs **no database
access**, which keeps the hot path cheap.

Every handler currently binding `Extension(_current_user)` should be revisited;
`save_user` in particular must use `current_user.id` instead of `request.id`.

## Frontend work

This is a visible rework of the pre-auth surface, not just plumbing. Today
`/login` is a single email box with the subtitle "Enter your email to continue to
your mock workspace." That copy and that flow both go away.

### New: `/register`

`web/src/pages/Register.tsx` renders `<SignupForm>`. Route wiring — `RouteId` in
`web/src/app/route-manifest.ts` is a closed union consumed by an exhaustive
`switch` in `router.tsx`, so this needs the same five coordinated edits as any new
page:

1. `route-manifest.ts:12` — add `| "register"` to `RouteId`
2. `route-manifest.ts:43` — add
   `{ id: "register", path: "/register", access: "anonymous", targets: allTargets }`
   — **`anonymous`**, so `RedirectIfAuthenticated` bounces signed-in users to
   `/home`, matching `/login`
3. `router.tsx:18` — `const Register = lazy(() => import("@/pages/Register"))`
4. `router.tsx:64` — `case "register": return <Register />`
5. No `app-sidebar.tsx` entry — this is a pre-auth page

There is also a `verify-email` route already in the manifest that currently
renders `<Navigate replace to="/login" />`. Email verification is out of scope;
leave it as is.

### Signup form configuration

```typescript
type SignupSubmitPayload = {
  stepCompleted: "single" | "step1" | "step2"
  values: {
    email: string
    password?: string   // optional because magic-link omits it
    firstName?: string
    lastName?: string
    displayName?: string
    phone?: string
    company?: string
  }
  isHoneypotTripped: boolean
}
```

- `onSubmit: (payload) => Promise<void>` — the only required prop.
- `flow="single-step"` (the default). **Do not use two-step.** It lets the user
  skip step 2, submitting `stepCompleted: "step1"`, which forces the handler to
  switch on the discriminant for no benefit here.
- `passwordStrategy` stays the default `password`. Note `values.password` is
  typed optional only because magic-link omits it — narrow it before use.
- `fields` — the `users` table has a **non-null `name`**, so registration must
  produce one. Enable `displayName: { required: true }` (or first/last), or
  derive a name server-side from the email. Pick one and be explicit.
- `passwordPolicy={{ minLength, requireUppercase, requireNumber, requireSymbol,
  showStrengthMeter }}` — **client-side only, UX not a control.** The server
  enforces its own minimum independently. Keep the two in sync so the form does
  not accept what the API then rejects.
- `consent` — Curio has no Terms or Privacy page. Either point `href` at
  something real or leave consent off rather than linking a 404.
- Controlled status: pass `status` + `errorMessage` so server errors ("email
  already registered") surface in the form's own alert region.
  **Mutual-exclusion contract:** passing `status` makes the component read-only
  for internal state; omitting it means the component owns transitions. Do not
  mix.
- `signInHref="/login"`.
- Honeypot: the payload carries `isHoneypotTripped` and the component does not
  auto-reject. The docs recommend a silent success — return as if registration
  worked, without creating a user.

Accessibility is handled by the component (`aria-invalid`, `aria-describedby`,
`role="alert"`, `aria-busy`, reduced-motion). Do not undo it.

### Google button — present now, wired later

Enable it on the signup form:

```tsx
oauthProviders={["google"]}
oauthIcons={{ google: <GoogleIcon className="size-4" /> }}
onOAuthClick={({ provider }) => { /* not yet implemented */ }}
```

Three things to get right:

**The component renders the button but does not perform the handshake.**
`onOAuthClick({ provider })` is a bare callback. There is no Google OAuth client
id, no redirect URI, no callback route, and no `google_id` column — so nothing
happens on click until that work lands.

**A button that silently does nothing is a bug report waiting to happen.** Since
the handshake is deliberately deferred, make the deferral visible: render it
`disabled` with a "Coming soon" affordance, or have `onOAuthClick` surface a
brief "Google sign-in isn't available yet" message. Either is fine. A live-looking
button that swallows the click is not.

**Lucide has no Google icon.** The component docs note: *"Lucide-react v1+ dropped
branded icons (Google / GitHub / Apple) to dodge licensing."* So `oauthIcons`
needs a hand-supplied SVG. Follow Google's branding guidelines for the mark if
this ships to real users.

Also plan the same treatment on the **login** page so the two pre-auth screens
match — a user who sees "Continue with Google" on signup will look for it on
login.

When the real integration lands it needs, at minimum: an OAuth client id and
secret in config, a `/auth/google/callback` route, a `google_id` column on
`users`, and a decision about account linking when a Google email matches an
existing password account. Worth its own ticket.

### Login page

**The `signup-form` component is signup-only.** The docs are unambiguous: *"This
is a signup-only component. It does NOT handle sign-in/login modes,"* and the
summary table lists `Login/Sign-in mode ✗`. Do not try to coerce it — its payload
shape (`stepCompleted`, `isHoneypotTripped`, consent) is signup-specific.

Extend `web/src/components/auth/email-login-form.tsx` instead. It already renders
a Card with an email `Input`, a `Label`, inline `role="alert"` errors, and an
`isSubmitting` prop that is currently accepted but never set. Add:

- A password field, `type="password"`, `autoComplete="current-password"` (and
  `autoComplete="email"` is already on the email input).
- A show/hide toggle if you want parity with the signup form's password field.
- The Google button, matching the signup treatment above.
- A "Create an account" link to `/register`.
- Wire the existing `isSubmitting` prop so the button disables during the request.

Update the card copy — "Enter your email to continue to your mock workspace" is
wrong on both counts once this ships.

`Login.tsx` currently calls `loginUser(email)` and navigates synchronously on the
next line. It must `await`, catch rejection, and render the failure instead of
navigating.

### Auth context is a breaking change

Today (`web/src/types/LoginRegisterTypes.ts`):

```typescript
loginUser: (email: string) => void
```

It becomes async, takes a password, and can fail:

```typescript
loginUser: (email: string, password: string) => Promise<void>
registerUser: (input: RegisterInput) => Promise<void>
```

Callers to update: `Login.tsx`, `nav-user.tsx` (logout), and `RequireAuth`.

**Add a loading state.** Today `isAuthenticated` is derived synchronously from
localStorage. With a real token there is a window on startup where the session is
being restored, and `RequireAuth` must not redirect to `/login` during it — or
every refresh bounces the user out.

Since the JWT carries `exp`, the provider can check expiry locally on startup and
drop an expired token without a network round trip. Treat that as a fast path,
not as verification: the client cannot validate the signature, so the server is
still the authority.

### Storage and validators

`features/auth/storage.ts` is mock-specific — `buildMockUser`, `parseStoredUser`,
the `curio-mock-auth-user-v1` key and its two legacy keys. It stores a synthesised
user, not a token. Rewrite it to persist the JWT and the server-returned user,
and pick a **new storage key** so existing mock sessions are dropped rather than
misread as valid. `storage.test.ts` covers the mock behaviour and needs rewriting
alongside.

`lib/validators/auth.ts` exports a hand-rolled `EMAIL_REGEX` and `validateEmail`.
Once zod is a dependency the signup form validates internally; keep this for the
login form only, or migrate the login form to zod too for one validation story.

### Network boundary

All calls go through `src/api/` — add `web/src/api/auth.ts` following
`src/api/conversations.ts`. Page and component code must never call `fetch`
directly: `web/eslint.config.js` bans the `fetch` global across `**/*.{ts,tsx}`,
and `web/scripts/check-frontend-boundaries.mjs` hard-fails the build on it. The
component's docs example calls `await api.signUp(...)` inside `onSubmit`, which is
the right shape — route it through the typed API layer.

## Dependency gap

| Requirement | Repo state | Action |
| --- | --- | --- |
| `@ilinxa` registry | Not in `web/components.json` | Add `"@ilinxa": "https://ui.ilinxa.com/r/{name}.json"` |
| shadcn `input`, `label`, `button` | Present | — |
| shadcn `checkbox` | **Missing** | `npx shadcn@latest add checkbox` |
| `react-hook-form@^7.75.0` | Not installed | Add |
| `@hookform/resolvers@^5.2.2` | Not installed | Add |
| `zod@^4.4.3` | Not installed | Add — note **v4**, not v3; the APIs differ |
| `lucide-react@^1.11.0` peer | Repo pins `^0.525.0` | Same conflict as the calendar/notes plans |
| Rust password hashing | No crypto crates in `Cargo.toml` | Add `argon2` + `password-hash` |
| Rust JWT | None | Add `jsonwebtoken` |
| Google icon | Lucide v1+ dropped branded icons | Hand-supplied SVG via `oauthIcons` |

## Risks

**1. Password handling is unforgiving code.** Argon2id with library defaults,
never a fast hash, constant-time verification, and no password or hash in any log
line or error body. Confirm the hash never appears in a `UserRecord` — it derives
`Serialize` and is returned directly from `save_user`, so a new field would be
serialized to clients by default.

**2. JWT-specific footguns.** Pin the algorithm; always set and validate `exp`;
never put secrets or mutable data in claims; remember the payload is readable by
anyone holding the token. And the signing secret must have no development
fallback — see "The signing secret" above.

**3. `lucide-react` peer conflict.** The component declares `^1.11.0`; the repo
pins `^0.525.0`. Same conflict as the calendar and notes plans — whichever lands
first sets the resolution and the others inherit it.

**4. zod v4, not v3.** `zod@^4.4.3` is a major-version jump from the v3 API most
examples assume. If other work later adds zod expecting v3, they will conflict.

**5. Vendored code vs. the architecture gates.** The `signup-form` is ~1,400 LOC
landing inside `web/src/`. Both gates scan that tree — run
`npm run check:architecture` and `npm run lint` immediately after install, before
building on top of it. `tsconfig.app.json` also runs `strict`, `noUnusedLocals`,
`noUnusedParameters`, and `verbatimModuleSyntax`, which third-party code
frequently trips.

**6. Both build targets.** Web (`BrowserRouter`) and desktop (`HashRouter`) share
`src/`. Auth must work on both, and nothing in the pages may import
`@tauri-apps/*` or `@curio/platform-runtime` — ESLint blocks both outside their
designated modules.

**7. Existing service tests will fail and that is correct.** `Bearer
development-token` stops working once tokens are real.
`authenticated_routes_deserialize_json_requests`,
`saving_a_user_upserts_the_profile`, and `saving_a_user_rejects_blank_profiles`
all rely on it; `chat_route_normalizes_openai_streams` sends no header at all;
and `test_config()` will not compile until it supplies a `jwt_secret`. Update
them to register and log in a real user. Do not weaken the middleware to keep the
old tests green.

## Phases

### Phase 1 — Service: schema, hashing, config

Add `argon2`, `password-hash`, and `jsonwebtoken` to `curio-service/Cargo.toml`.
Write `migrations/0003_auth.sql` (nullable `password_hash` on `users`). Add
`jwt_secret` to `ServiceConfig` via `required()`, update `.env.example` and
`test_config()`.

Tests: a correct password verifies, a wrong one does not, two hashes of the same
password differ (salting), and `ServiceConfig::from_env()` fails when
`CURIO_JWT_SECRET` is unset or blank.

### Phase 2 — Service: JWT issue and verify

Add `curio-service/src/auth/` following the `src/user/` module shape. Sign with
HS256, claims `sub`/`exp`/`iat`. Verify with an explicit
`Validation::new(Algorithm::HS256)`.

Tests: a signed token round-trips to the right `sub`; an expired token is
rejected; a token signed with a different secret is rejected; a tampered payload
is rejected.

### Phase 3 — Service: register and login

Register and login are **public** routes — merged outside the
`route_layer(middleware::from_fn(auth))` group.

Tests: register creates a user and returns a token; duplicate email returns 409;
login with the right password returns a token; wrong password and unknown email
return **identical** status and body; a NULL `password_hash` cannot log in.

### Phase 4 — Service: real auth middleware

Rewrite `authorize_current_user` to decode and validate the JWT and populate
`CurrentUser { id }`. Convert the middleware to carry the decoding key. Change
`save_user` to take the id from `CurrentUser`, not the body. Decide and act on
whether chat routes move behind auth. Update the tests that relied on
`Bearer development-token`.

Verify: `Bearer x` now returns 401 — the defect in finding (1) is closed.

### Phase 5 — Frontend: dependencies and API layer

Add the `@ilinxa` registry, `npx shadcn@latest add @ilinxa/signup-form`, add the
`checkbox` primitive, install `react-hook-form`, `@hookform/resolvers`, and
`zod@^4`. Resolve the `lucide-react` peer. Run the gates immediately.

Write `web/src/api/auth.ts` with typed `register` and `login` over `api.post`.

### Phase 6 — Frontend: auth context and token storage

Rewrite `features/auth/storage.ts` for JWTs (new storage key), update
`LoginRegisterTypes.ts`, and rewrite `auth-provider.tsx` to call the API, persist
the token, expose async `loginUser` / `registerUser` / `logoutUser`, and carry a
loading state. Check `exp` locally on startup as a fast path. Thread the stored
token into `ApiRequestOptions.bearerToken`. Rewrite `storage.test.ts`.

Update `RequireAuth` to wait on the loading state instead of redirecting during
session restoration.

### Phase 7 — Frontend: register and login UI

Build `Register.tsx` with `<SignupForm>` in controlled-status mode, including the
disabled Google button. Wire the five route edits.

Extend `email-login-form.tsx` with the password field, the matching Google
button, a link to `/register`, and the `isSubmitting` wiring. Update the card
copy. Update `Login.tsx` to await `loginUser` and render failures.

Match the existing pre-auth visual treatment — centered column, `bg-muted/30`,
the Curio logo above the card.

### Phase 8 — End-to-end verification

Register → auto-login → land on `/home`; log out; log back in with the same
credentials; wrong password shows a uniform error; duplicate registration shows
"already registered"; refresh preserves the session; an expired or tampered token
lands on `/login`; the Google button communicates that it is not yet available.
Confirm both web and desktop.

## Definition of done

- `POST /v1/auth/register` hashes with argon2id, rejects duplicate emails, and
  returns a JWT
- `POST /v1/auth/login` returns a JWT and fails uniformly for unknown email and
  wrong password
- JWTs are HS256, carry `sub`/`exp`/`iat`, and are verified with a pinned
  algorithm
- The service refuses to start without `CURIO_JWT_SECRET`
- `Bearer x` no longer authenticates; `CurrentUser` carries a real user id
- `save_user` derives identity from the JWT, not the request body
- The chat-route auth decision is made and recorded (implemented, or explicitly
  deferred)
- The logout/revocation limitation is documented where a reader will find it
- `/register` renders the signup form; `/login` accepts email + password; both
  redirect authenticated users away and link to each other
- A Google button appears on both screens and visibly communicates "not yet
  available" rather than failing silently
- No password or password hash appears in any response body or log line
- `cargo test` passes with the updated service tests
- `npm run lint`, `npm run check:architecture`, `npm run build:web`,
  `npm run build:desktop-ui`, and `npm test` all pass
