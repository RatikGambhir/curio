# NATS embedding producer in curio-service and Rust consumer in curio-workers

Status: proposed. This plan does not describe a deployed feature.

## Decision

Keep the producer in `curio-service/`: the Axum chat completion transaction creates durable embedding jobs, and an in-process background dispatcher publishes their notifications to NATS JetStream. Replace the disconnected Gemini/D1/Cloudflare Queue implementation in `curio-workers/` with an independent Rust consumer package, deployed as a separate, always-on Railway service named `curio-embed-worker`. The consumer accepts each notification, loads the saved message, generates its embedding, and persists the result using the processing rules below.

Rust is required for the consumer. Use a Railway background service for this plan: Railway's product named *Functions* currently runs single-file TypeScript on Bun, so it does not provide the requested Rust runtime. A future Rust Function deployment is an option only after verifying Rust support, private NATS/PostgreSQL connectivity, and execution semantics that preserve durable consumption, leases, acknowledgements, and recovery scans. See [Railway Functions](https://docs.railway.com/functions).

These are alternative broker implementations: choose this plan or [the RabbitMQ alternative](rabbitmq-embedding-railway-worker-plan.md), rather than deploying both for the same jobs.

Scope for the first release: embed nonempty, completed **user and assistant message text** from the active PostgreSQL `messages` table, save vectors in PostgreSQL, and leave the public chat HTTP/SSE contract unchanged. The current Axum service remains the only chat streaming backend. Chat attachment embedding remains outside this release. The existing documents domain already stores file versions, blobs, chunks, and vectors; connecting chat attachments to it requires an explicit identity, ownership, and job-source design.

## Current state and gaps

- `curio-service` inserts a completed user message and a pending assistant before calling OpenAI. It commits completed assistant content and `response_id` before emitting the Curio `done` SSE event. Failed/interrupted assistants are stored with separate states.
- The disconnected chat Worker used Gemini streaming, persisted to D1, and optionally sent `userId`, `threadId`, `userMessageId`, `assistantMessageId`, and `assetPath[]` to a Cloudflare Queue. The processor loaded both messages and assets, called Gemini embeddings, and stored a JSON response in D1.
- The existing Railway PostgreSQL database is the target; do not provision a replacement database. The active documents domain already persists file blobs, chunks, and embeddings, but the chat path has no completed message-embedding pipeline. Partial local work from the interrupted MQTT implementation must be inspected and adapted or explicitly removed during implementation, not assumed complete or deployed.
- Chat rows have no owner. The active chat request has no trusted `userId`, so the legacy `userId` cannot safely be carried over as an ownership claim.
- D1 records and Gemini vectors do not automatically exist in PostgreSQL. Inventory them before deleting the old deployment or source.

## Target flow

```text
curio-service/ (Rust Axum API + producer)
  POST /v1/chat/stream
  -> transaction inserts completed user + pending assistant
  -> OpenAI tokens stream to the client
  -> one transaction completes assistant and inserts two embedding jobs
  -> Curio `done` SSE event after that transaction commits

curio-service/ background job dispatcher (producer)
  -> JetStream publish to private NATS + durable publish acknowledgement
  -> curio-workers/ Rust service: curio-embed-worker durable pull consumer
  -> claim job and fetch saved content from PostgreSQL
  -> OpenAI embeddings API
  -> one transaction upserts vector and completes job
  -> explicitly acknowledge JetStream delivery
```

PostgreSQL is the durable work queue; NATS JetStream carries durable notifications. The worker also scans due jobs on an interval so a missed notification or broker outage cannot strand work. Embedding and NATS latency never hold up the chat response. Broker publication happens only after the completion transaction commits.

### Package and runtime ownership

| Responsibility | Owner |
| --- | --- |
| Chat HTTP/SSE, completed message persistence, atomic job creation | `curio-service/` |
| Background job dispatcher, publication leases, JetStream publish confirmations | `curio-service/` |
| NATS consumption, validation, content lookup, processing leases, OpenAI message embeddings, vector persistence, retries, reconciliation, and replay | New Rust package at `curio-workers/` |
| PostgreSQL schema migrations and administrative database tools | `curio-service/migrations/postgres` and `curio_db` |
| Durable notifications | Private Railway NATS JetStream service |

The API and consumer build, deploy, scale, and restart independently. The consumer does not serve chat or import the Axum application. Their integration boundaries are the versioned NATS contract and the shared PostgreSQL schema. The existing synchronous document-embedding path in `curio-service` remains unchanged.

### NATS JetStream v1 contract

Use subject `curio.v1.embeddings.messages`, file-backed stream `CURIO_EMBEDDINGS_V1`, and durable pull consumer `curio-embed-worker-v1` with explicit acknowledgements. Provision stream/consumer topology before enabling publication. Use JetStream publish and await its publish acknowledgement before recording acceptance; plain Core NATS publish/flush is insufficient. Set JSON content type and `Nats-Msg-Id=<jobId>:<publishGeneration>`. See [JetStream publishing](https://docs.nats.io/learn/jetstream/publishing).

One notification per message job:

```json
{
  "version": 1,
  "jobId": "<server-generated UUID>",
  "messageId": "<saved message ID>",
  "conversationId": "<saved conversation ID>",
  "role": "assistant",
  "responseId": "<provider response ID or null>",
  "completedAt": "<UTC timestamp>",
  "model": "text-embedding-3-small"
}
```

The user-message event has `role: "user"` and `responseId: null`. `conversationId` replaces legacy `threadId`; the former `userMessageId` and `assistantMessageId` become independent jobs. No prompt, response text, vector, API key, or attachment bytes enter NATS. Validate subject, version, payload size, field lengths, and shape. Treat IDs and metadata as hints: fetch the job and row, then verify message ID, conversation, role, status, model, and content hash against PostgreSQL before using the content. Log mismatches without sensitive content.

Do not send `userId` until chat has authenticated, persisted ownership; do not copy an unverified client value. Do not send legacy `assetPath` values: chat messages currently have no authoritative link to the active document/file records. When chat attachments are integrated, use immutable database file/version IDs and a versioned contract.

## Database and code changes

Inspect any unapplied local migration from the interrupted implementation, then add or complete an append-only migration under `curio-service/migrations/postgres`. Never edit an applied migration:

1. `embedding_jobs`: UUID primary key; `message_id` FK with cascade delete; model; SHA-256 of the exact saved UTF-8 content; state (`pending`, `processing`, `retry`, `completed`, `dead`, `superseded`); attempts; `available_at`, lease expiry and fencing token, separate publication lease/token, last confirmed publish time, publish-generation counter, timestamps, and sanitized error code. Unique `(message_id, model, content_sha256)`; index due jobs and expired leases.
2. `message_embeddings`: `(message_id, model)` primary key and FK to `messages` with cascade delete; source SHA-256; dimension count; `real[]` vector; timestamps; array shape/dimension constraint. Save numeric floats, not the full provider response. Validate finite values, model, and dimensions in Rust. Upsert vector and mark job complete in the same transaction.

Use OpenAI `text-embedding-3-small` at the documented default 1,536 dimensions initially, with explicit model/dimension configuration; see the [OpenAI embedding guide](https://developers.openai.com/api/docs/guides/embeddings). Store model and source hash to support deliberate re-embedding. Start with PostgreSQL arrays to reuse the existing Railway PostgreSQL database without requiring an extension or database-image change; see [Railway extension guidance](https://docs.railway.com/databases/postgresql#extensions). If indexed message similarity search is later required, assess pgvector support on the existing database or plan an explicit database migration, then move to `vector` plus an appropriate index, following [pgvector](https://github.com/pgvector/pgvector). Array storage here does not promise indexed vector search.

Insert both jobs in the same transaction that marks the assistant completed. Skip empty content. Do not enqueue pending, failed, or interrupted assistants. If job insertion fails, roll back completion and emit the existing sanitized storage error, retaining persistence-before-`done`. Backfill policy for a user message whose assistant failed must be decided explicitly; the proposed live path waits for a completed assistant turn. Update the expected catalog in `curio-service/src/bin/curio_db.rs` and checks in `curio-service/ops/postgres/verify.sql`; verify application-role grants cover the new tables. The SQLite importer has already been retired. Continue to apply DDL only through `curio_db migrate` with migrator credentials.

### Producer implementation: curio-service

Extend Chat's repository/service boundary to create jobs inside its completion transaction. Put producer-side job contracts, publication repository, and dispatcher under `curio-service/src/domains/embeddings/`; put the NATS publishing client under `curio-service/src/adapters/`, with no adapter-to-domain imports. Compose and supervise the dispatcher through `curio-service/src/app/` alongside the API, including cancellation and bounded shutdown. Preserve shared SQL-builder conventions, adding narrowly tested support for composite conflicts and `FOR UPDATE SKIP LOCKED` if needed. This package owns job creation and publication; the new message-embedding processor runs in the consumer package.

### Consumer implementation: curio-workers

Create an independent Cargo package at `curio-workers/` with its own `Cargo.toml`, committed `Cargo.lock`, `src/main.rs`, tests, and `railway.toml`. Name its executable `curio-embed-worker`. Use a thin entrypoint, `src/app/` for configuration/composition/lifecycle, `src/domains/embeddings/` for the notification contract, processing service and repositories, and `src/adapters/` for NATS, PostgreSQL, and OpenAI clients. Keep infrastructure adapters independent of domain modules. The Rust consumer owns the full receive/validate/claim/load/embed/commit/acknowledge cycle, plus bounded retries, recovery scans, and operator replay.

Keep the packages independent: do not add a root Cargo workspace or a dependency on the full `curio-service` application. Mirror the small versioned notification DTO in both crates and test producer serialization against consumer decoding with common fixtures. Treat the job/vector schema and state transitions as a shared contract verified by integration tests. Migrations remain solely under `curio-service`; the worker verifies the required migration state and schema compatibility at startup and never applies DDL. Coordinate schema changes with both consumers of the database contract.

During migration, the Rust package may coexist with the legacy `curio-chat-worker/`, `curio-processor-worker/`, and D1 `migrations/` subdirectories. Build only the Rust package for Railway. Remove those legacy contents after the retirement gates pass; retain `curio-workers/` as the new Rust consumer's home.

### Dispatch and processing lifecycle

The service-side dispatcher claims due unpublished jobs with a separate, fenced publication lease across API replicas. Record publication only after broker confirmation; a socket write or local client enqueue is insufficient. Uncertain outcomes leave publication retryable, including a crash after broker acceptance but before the database marker. Each intentional retry/replay advances a durable publication generation; transport retries of that generation retain its publication identity. The worker starts with one replica, reconnects with bounded backoff, binds to a stable durable consumer, limits pull batches and processing concurrency, and leases jobs with `FOR UPDATE SKIP LOCKED` or equivalent. Hold no database lock during the provider call. Bound provider timeout below the processing lease duration and fence every result/retry write by lease token and expiry; an expired worker cannot commit over a new claimant. Broker acknowledgements never substitute for database fencing. On receipt or periodic scan, claim a job, fetch the completed row, check nonempty content and hash, call `/v1/embeddings` over HTTPS, then recheck source status/hash while committing. A stale result must not overwrite an embedding for edited content; edits supersede old jobs and enqueue a new hash, while deletes cascade. Current chat exposes no message editing, but retries still need this guard.

Retry transport failures, 429, and provider 5xx with exponential backoff and jitter. Classify invalid inputs/permanent failures as `dead`, cap attempts, and provide operator replay. Duplicate JetStream deliveries converge through the unique job key, lease, and idempotent `(message_id, model)` upsert. Expired leases and a bounded reconciliation scan recover from crashes and find eligible completed messages missing a job or vector. Reconciliation must honor the explicit failed-turn user backfill policy and must not silently revive `dead` jobs. Operator replay is scoped by job ID, resets bounded attempts, and revalidates saved content. Log only IDs, error class, attempts, lag, and duration; never log message content, provider bodies, secrets, or full broker payloads.

### JetStream delivery and retry rules

Configure `WorkQueuePolicy` retention on the exact subject and share one durable pull consumer across worker replicas; do not create overlapping work-queue consumers per replica. Set explicit storage, message-size, age, and byte limits. Use `DiscardNew` for capacity overflow so publication fails and the PostgreSQL dispatcher can retry. A notification removed by retention must still be recoverable through the database scan. See [retention policies](https://docs.nats.io/learn/jetstream/retention-policies).

Bound pull batch size and `MaxAckPending` to processing capacity. Set `AckWait` above the normal bounded provider-plus-commit duration; any progress acknowledgements must not extend the PostgreSQL lease implicitly. Acknowledge, preferably with server-confirmed acknowledgement, after committing `completed`, scheduled `retry`, or a terminal state. For a duplicate, acknowledge only after authoritative lookup establishes completion, an active lease, or future availability. If database lookup/commit fails, leave unacknowledged or use a delayed negative acknowledgement. The database scanner remains the recovery path. See [pull consumers](https://docs.nats.io/learn/jetstream/pull-consumers).

PostgreSQL owns provider retry timing and attempt limits. Commit retry state, clear publication status, and acknowledge the old notification; the dispatcher publishes a new generation when due. A transport retry reuses its generation's `Nats-Msg-Id`, while an intentional retry/replay uses a new generation so JetStream's duplicate window cannot suppress needed work. Database uniqueness and lease fencing remain necessary beyond that window; do not claim exactly-once OpenAI calls. See [advanced publishing](https://docs.nats.io/learn/jetstream/advanced-publishing).

Configure bounded redelivery and alerts for exhausted broker deliveries. Do not assume `MaxDeliver` automatically creates a dead-letter queue. Malformed or mismatched notifications need a bounded quarantine stream: publish a sanitized diagnostic with acknowledgement before terminating delivery; if that publication fails, retain/retry handling with backoff. For unknown or forged job IDs, never update a database job based only on the envelope. Operator replay uses authoritative PostgreSQL `dead` jobs; document how to inspect and remove exhausted/quarantined broker records without deleting valid database work.

## Railway deployment

1. Reuse the existing PostgreSQL service. Add a separate private NATS service in the same Railway environment with a pinned supported server image, JetStream enabled, and its `store_dir` on a persistent Railway volume. Start with one server and stream replica count 1; this is durable storage, not HA. A later three-node JetStream cluster requires explicit route configuration, distinct server identities and volumes, and stream replica count 3. Keep client, routing, and monitoring ports private with no public TCP proxy/domain. Bind listeners for Railway's private-network address family. See [Railway private networking](https://docs.railway.com/networking/private-networking).
2. Keep the API/producer service rooted at `/curio-service` with `curio-service/railway.toml`. Deploy the consumer as a separate service rooted at `/curio-workers` with `curio-workers/railway.toml`; build with `cargo build --release --locked --bin curio-embed-worker` and start `./target/release/curio-embed-worker` from that package root. These worker files/commands are implementation targets, not existing artifacts. Set independent deployment watch paths and verify each service selects its own Cargo manifest, lockfile, and Railway configuration. Both processes must verify the required schema migration; only the existing service migration tooling applies it. See Railway's [monorepo guidance](https://docs.railway.com/deployments/monorepo).
3. Set private DB URL, validated `CURIO_DB_SCHEMA`, `CURIO_NATS_URL`, separate producer/consumer NATS credentials, stream/subject/durable-consumer settings, `OPENAI_API_KEY`, model/dimensions, pool/concurrency limits, and retry bounds through Railway secrets/config. The worker gets a least-privilege application role, never the migrator URL or `VITE_*` variables. Budget the combined SQLx pools across replicas.
4. If Railway uses an HTTP healthcheck, give the worker a private liveness/readiness listener. Readiness requires DB migration verification and a usable JetStream stream and bound durable consumer. Monitor pending/dead jobs, oldest job age, retries, broker connectivity, provider errors, and processing time.

Provision topology with a separate administrative identity. Producer permissions should cover only the embedding subject and required acknowledgement inboxes; consumer permissions should cover the named consumer's required JetStream API, delivery inbox, acknowledgement, and quarantine subjects. Do not give runtime identities unrestricted `$JS.API.>` or stream-deletion access. Validate these permissions with the real client during deployment. Use the Tokio-compatible `async-nats` client in both packages, pin its resolved version in each package's Cargo lockfile, and keep transport code in its adapter.

## Implementation and retirement gates

| Phase | Work | Exit criterion |
| --- | --- | --- |
| 0. Inventory | Check live Railway/Cloudflare deployments, bindings, D1 row/assets counts, queue backlog, and retention needs. Confirm no client uses either Worker. | Decide whether D1 data/assets need import, export, or archival. |
| 1. Persistence | Add migration, DB grants, verification SQL, job/vector repositories, and provider validation. | Disposable PostgreSQL tests prove constraints, transaction boundaries, duplicates, cascade, and stale-result guards. |
| 2. Runtime | Add atomic completion/job insertion and the dispatcher in `curio-service`; create the independent Rust consumer in `curio-workers`, with the NATS JetStream v1 contract and replay/reconciliation. | Both packages build independently. End-to-end tests show `done` after DB commit, DB content lookup by ID, eventual vector with duplicate/lost JetStream notifications, and chat success despite later embedding failure. |
| 3. Deployment | Migrate schema first; deploy broker/worker privately; enable producer with a server-side flag. Backfill nonempty completed PostgreSQL messages using bounded, resumable job insertion. | The live synthetic-message test passes against staging on Railway; staging and production backlog drains, vectors have expected model/dimension, and pause/replay is exercised. |
| 4. Retirement | Retain or migrate verified D1 data, then remove the legacy TypeScript/Wrangler packages and D1 migrations from `curio-workers/`, plus unused Cloudflare deployments, bindings, and secrets. Preserve the new Rust package at `curio-workers/`. Update its README, `AGENTS.md`, the root `README.md`, `docs/ARCHITECTURE.md`, `docs/deployment.md`, and CI/verification references. Keep older `plans/` as history. | No live integration or current guide depends on the Cloudflare Workers; the Rust consumer replaces their processing role and recovery data is retained for the agreed window. |

If D1 contains attachments that must remain usable, or attachment embedding is required for parity, add an attachment import/integration phase before Phase 4: assess reuse of the existing PostgreSQL document storage, define ownership and stable file/version IDs, extraction per supported MIME type, size/security limits, an attachment job source, and tests. The old processor base64-encoded arbitrary bytes into a text embedding request; do not repeat that behavior. Record the no-assets finding if this phase is unnecessary.

## Verification and rollback

Run unit tests for the NATS payload schema, source-hash validation, embedding shape, retry classification, and sanitized errors. Use disposable `curio_test_*` PostgreSQL schemas for migration, atomic completion, lease competition, idempotent upsert, crash/retry, delete cascade, and backfill tests. A green `cargo test` without safe `CURIO_TEST_DATABASE_URL` is partial evidence. Separately exercise a real local broker and a fault-injection provider through disconnects before/after publish, duplicate/redelivered JetStream messages, lost publish/consumer acknowledgements, deduplication-window retry collisions, exhausted delivery limits, broker outage, timeout, 429/5xx, and worker restart. Confirm `done` never waits for embedding.

### Required live integration test: synthetic input, real downstream calls

Provide an explicitly opt-in `embedding_live` integration target under `curio-service/tests/` that coordinates the real producer with the separately built `curio-workers` binary. Put consumer unit and broker integration tests under `curio-workers/tests/`. The only fabricated part of the acceptance scenario is the incoming message content and its fixture IDs. After fixture setup, use production paths and real services: no fake broker, mocked OpenAI response, in-memory repository, precomputed vector, or direct processor invocation that bypasses delivery.

1. Require an explicit live-test flag, real `OPENAI_API_KEY`, safe `CURIO_TEST_DATABASE_URL`, and isolated broker credentials/resources. Fail clearly when the explicitly requested live target lacks prerequisites; do not silently report a skipped live test as passing. Document that this test makes billable provider calls using synthetic, nonsensitive text.
2. Identify the existing Railway PostgreSQL instance and provision a dedicated test role with no access to application schemas. Create and migrate only a disposable `curio_test_*` schema. Use unique per-run broker resource names and fixture IDs. Never run this test with the production application's schema, queue/stream, or administrator credentials.
3. Build `curio-embed-worker` from `curio-workers/Cargo.toml` and pass its executable path explicitly to the live-test harness. Start that binary against the isolated schema and broker resources, then await database-and-consumer readiness. For the delivery assertion, use a test configuration with the fallback scan delayed/disabled so a scan cannot mask broken broker consumption.
4. Seed a synthetic completed turn and its jobs through the real Chat repository completion transaction. Publish the v1 notification using the real broker adapter and wait for its broker confirmation. From receipt onward, the real worker must decode/validate, claim the job, fetch saved content, call OpenAI over HTTPS, persist the returned vector, complete the job, and acknowledge delivery.
5. Assert the job is `completed`, the source hash matches exact saved UTF-8 text, and exactly one `(message_id, model)` embedding exists with the configured model, finite values, and 1,536 dimensions by default. Assert database completion and broker acknowledgement; record only redacted identifiers/counts/timing. Exercise both user and assistant messages.
6. Send duplicate deliveries and restart the worker; verify the completed vector remains unique and unchanged. Run a separate missed-notification scenario with scans enabled and no publish, proving recovery performs real OpenAI and PostgreSQL calls. A crash during an external call may repeat a billable call; do not claim exactly-once provider execution.
7. Add an end-to-end producer case that sends a synthetic prompt to the actual chat HTTP route, uses the real OpenAI Responses API, and observes real dispatcher/broker/worker/embedding persistence. Assert jobs exist when `done` arrives and completion does not await the embedding path. This verifies the producer in addition to the incoming-notification scenario.
8. Stop only test-owned processes and delete only the per-run schema and broker resources. Execute cleanup on assertion failure too; document a targeted recovery command for cleanup interrupted by process termination.

Run locally with real broker/PostgreSQL dependencies and again with broker and worker inside a Railway staging environment using the existing database instance and isolated test schema. Keep provider fault-injection tests for deterministic 429/5xx/timeouts separate from this live acceptance test; they do not satisfy the requirement that every downstream call is real. Document the exact commands and pass/fail/skip results for each layer.

From each of `curio-service` and the new `curio-workers` Cargo package, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, database-backed tests with a safe test URL, and the applicable broker integration targets. Add CI coverage for the new Rust package and cross-package notification/schema contracts. Run `curio_db verify` from `curio-service` only against an explicitly identified safe schema. Run frontend chat contract checks if the public HTTP/SSE contract changes. Check `git diff --check` and the final diff. As each phase lands, compare implementation with canonical `docs/ARCHITECTURE.md` and update current-state sections and `docs/deployment.md` when behavior or operations actually change.

Ship schema and worker before enabling the producer. Stop broker intake on shutdown and drain in-flight work within a deadline; unfinished claims recover through lease expiry. To stop a bad rollout, disable producer/worker while retaining jobs and vectors, then fix and replay. Do not edit an applied migration to roll back. Delete Cloudflare resources only after the Rust path and D1 retention decision pass the gates.
