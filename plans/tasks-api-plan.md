# User-owned Tasks API plan

Status: proposal. This document describes a future `curio-service/src/domains/tasks/` implementation. The independent [Spaces API plan](spaces-api-plan.md) is a prerequisite; neither API exists yet.

## Goal and boundaries

Make a Space a first-class, user-owned container shared across Curio. A user's documents, tasks, calendar events, conversations, and later notes can each belong to one Space or remain standalone. User ownership is always the outer boundary; optional Space membership is the next scope. A Space is not a special type of task or a separate account.

Create an independent task resource that can be created, read, edited, completed, reopened, and deleted by its owner. A task may have one optional Space, many tags, comments, web links, Curio note links, and subtasks. Support due dates, priority, flags, ordering, and useful list filters. New Tasks and Spaces endpoints take the owner from `CurrentUser.id`, never from a body or query `userId`.

This is a service/API plan. The current Calendar “Tasks” list and kanban are projections of `calendar_events`; this proposal does **not** silently replace, migrate, or synchronize them. A later client integration must decide whether to display task records alongside calendar events and avoid treating a task due date as a calendar event without an explicit mapping.

The existing bearer middleware only interprets a nonblank token as a development user ID. Owner-scoped SQL prevents accidental cross-user access under that model, but does not provide production authentication. A real auth rollout must replace token verification across the service. Chat needs an additional ownership migration before it can safely participate in Spaces: conversations currently have no owner and chat/history routes are public.

## Reminders feature inventory and choices

Apple Reminders currently exposes notes, URLs, due date/time, repeat, early reminders, location and Messages triggers, flag, priority, photos/scans, subtasks, lists/sections, tags, Smart Lists, and shared-list assignment. The first API slice below implements the parts that are meaningful as user-owned service data: notes/description, dates, flags, priority, subtasks, Spaces, tags, comments, and links. Smart List views can initially be assembled from task query filters; saved Smart Lists come later. Repeat, alert delivery, location, attachments, templates, and sharing have explicit follow-up designs below; storing an alert rule alone would not mean a notification was sent.

Sources checked September 25, 2026: [Apple: Add details in Reminders](https://support.apple.com/guide/iphone/add-details-iphec7a1de82/27/ios/27), [Apple: Edit and organize a list](https://support.apple.com/guide/ipad/edit-and-organize-a-list-ipadaeb26d92/27/ipados/27), [Apple: Smart Lists](https://support.apple.com/guide/reminders/create-custom-smart-lists-remnfec66479/7.0/mac/27), and [Apple: Organize reminders](https://support.apple.com/en-gb/119953).

## Shared Spaces dependency

The [Spaces API plan](spaces-api-plan.md) owns the separate `spaces` table and its create, list, and delete routes. This Tasks plan adds nullable `tasks.space_id` only after that table exists. A task remains owned by its user whether `space_id` is set or null. Assigning a task requires an owner/Space match; deleting a Space leaves its tasks standalone and advances their task versions.

Other resources may also gain optional Space membership in their own domain migrations. Documents attach at `document_files`, calendar events at `calendar_events`, and chats at `conversations` only after chat has user ownership and protected routes. Notes need a persisted service resource first. Each owner/Space composite FK should clear only `space_id` on Space deletion; the owning user ID must remain intact. [PostgreSQL 18 documents column-specific `ON DELETE SET NULL`](https://www.postgresql.org/docs/18/ddl-constraints.html), so verify deployment compatibility before writing those migrations.

## First-release Tasks data model

Use append-only SQLx migrations under `curio-service/migrations/postgres/`: establish `spaces` before adding any member FKs, then create Tasks tables and migrate existing member domains in ordered steps. All new IDs use server-generated UUID strings (`text`) to match the service's existing public IDs. All audit instants are `timestamptz(3)` in UTC. Domain validation trims bounded text, parses dates/URLs, and rejects invalid combinations; database constraints defend the same invariants where practical. The table outline below is precise enough to implement but is not a migration to run as-is.

| Table | Columns and constraints | Purpose |
| --- | --- | --- |
| `tasks` | `id text PK`, `owner_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE`, `space_id text NULL`, `parent_id text NULL`, `title text NOT NULL`, `description text NULL`, `status text NOT NULL DEFAULT 'todo'`, `priority text NULL`, `flagged boolean NOT NULL DEFAULT false`, `due_on date NULL`, `due_at timestamptz(3) NULL`, `due_timezone text NULL`, `completed_at timestamptz(3) NULL`, `sort_order bigint NOT NULL DEFAULT 0`, `version bigint NOT NULL DEFAULT 1`, `created_at`, `updated_at`; `UNIQUE(owner_id,id)`; composite FKs `(owner_id,space_id) -> spaces(owner_id,id) ON DELETE SET NULL (space_id)` and `(owner_id,parent_id) -> tasks(owner_id,id)`; checks below | Task and optional same-owner Space/parent. Child deletion follows the parent via service transaction; avoid a self-referential cascade surprise. |
| `task_tags` | `id text PK`, `owner_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE`, `name text NOT NULL`, `name_key text NOT NULL`, `created_at`; `UNIQUE(owner_id,id)`, `UNIQUE(owner_id,name_key)` | Reusable per-user tag vocabulary. |
| `task_tag_assignments` | `owner_id text NOT NULL`, `task_id text NOT NULL`, `tag_id text NOT NULL`, `created_at`; `PRIMARY KEY(task_id,tag_id)`, composite FKs `(owner_id,task_id) -> tasks(owner_id,id) ON DELETE CASCADE` and `(owner_id,tag_id) -> task_tags(owner_id,id) ON DELETE CASCADE` | Many-to-many association with database-enforced common ownership. |
| `task_comments` | `id text PK`, `owner_id text NOT NULL`, `task_id text NOT NULL`, `author_id text NOT NULL`, `body text NOT NULL`, `created_at`, `edited_at timestamptz(3) NULL`; composite task FK on `(owner_id,task_id)` with cascade; `author_id` FK to `users(id)` | Append-only discussion by the current owner in v1; edit/delete own comment can be added without changing the task row. No collaboration is implied by this table. |
| `task_links` | `id text PK`, `owner_id text NOT NULL`, `task_id text NOT NULL`, `kind text NOT NULL`, `label text NULL`, `url text NULL`, `note_id text NULL`, `sort_order bigint NOT NULL DEFAULT 0`, `created_at`, `updated_at`; composite task FK with cascade; `kind IN ('web','curio_note')`; check exactly one of `url` or `note_id` is set according to `kind` | Ordered web and Curio note references. No remote URL fetching or note content is stored here. |

`tasks` checks: trimmed nonempty title; `status IN ('todo','in_progress','blocked','done','cancelled')`; `priority IS NULL OR IN ('low','medium','high')`; exactly one of `due_on` and `due_at` may be non-null; `due_timezone` is required only with `due_at` (validated as an IANA name by the service); `completed_at IS NOT NULL` exactly when `status = 'done'`; `version > 0`; `parent_id <> id`. The service rejects parent cycles and requires a child to remain in its parent's Space (or both unspaced); moving a parent moves its subtree transactionally. Date-only `dueOn` remains a local calendar date and is never coerced to midnight UTC. Timed `dueAt` is an RFC 3339 instant plus `dueTimezone` for display and future recurrence.

Indexes: `tasks(owner_id,created_at DESC,id DESC)` for default keyset paging; `(owner_id,space_id,sort_order,id)` and `(owner_id,parent_id,sort_order,id)` for list/tree order; partial `(owner_id,due_on,id) WHERE due_on IS NOT NULL` and `(owner_id,due_at,id) WHERE due_at IS NOT NULL`; `(owner_id,status,created_at DESC,id DESC)`; partial `(owner_id,created_at DESC,id DESC) WHERE flagged`; assignment `(owner_id,tag_id,task_id)`; comments `(owner_id,task_id,created_at,id)`; links `(owner_id,task_id,sort_order,id)`. Add a search index only after choosing and testing PostgreSQL full-text behavior; do not start with an unindexed arbitrary substring filter.

Deleting a Space follows the shared cross-domain detach rule above, not a Tasks-only deletion path. A task with children is deleted with its subtree in one transaction; associated tags are retained, while assignments/comments/links cascade. Task delete is hard delete in v1; recently deleted/recovery would need a separate soft-delete design across descendants and filters.

There is no persisted Notes table today. `curio_note` links store an opaque note ID as a **reference only**: no ownership claim, note preview, note existence claim, or resolution from Tasks. A client must not turn this into an unchecked cross-user note fetch. When Notes gains persistence, add an owner-scoped validation/resolution API and preferably a composite `(owner_id,note_id)` FK in a later migration, after planning how note deletion affects links.

## HTTP contract

Nest protected Tasks routes under `/v1/tasks` in `app/bootstrap.rs`; the independent Spaces plan owns `/v1/spaces`. JSON uses camelCase and the same millisecond UTC audit timestamps as existing service records. New Space/Task requests do not accept `ownerId`/`userId`; the owner is always the bearer-derived principal. A missing or foreign task/space/comment/link returns the same `404` to avoid resource enumeration. A missing current-user profile returns the existing calendar-style `422` until user provisioning is redesigned. Validation is `422`, malformed JSON/query is `400`, stale version is `409`, and unexpected storage failures are sanitized `500`. Keep feature-local `{ "error": "..." }` responses; do not claim a service-wide error envelope.

| Method/path | Behavior |
| --- | --- |
| `POST /v1/tasks` | Create a task. Required `title`; optional `description`, `spaceId`, `parentId`, `status`, `priority`, `flagged`, `dueOn` **or** `dueAt` plus `dueTimezone`, `tagIds`, `links`. Transactionally validate all same-owner references and write associations; return `201` and the full task. Comments begin through their own endpoint. |
| `GET /v1/tasks/{id}` | Return one owned task with tags, links, comment count, child count, and `version`. Page comment bodies through the separate comments route. No unbounded recursive subtree in one response. |
| `PATCH /v1/tasks/{id}` | Partial update of title, description, status, priority, flag, due fields, Space, parent, order, and full `tagIds` replacement. Require `If-Match: "<version>"`; increment version and return committed record. Omitted means unchanged; explicit `null` clears nullable fields; `tagIds: []` clears all tags. Reject empty patch and unknown keys. Revalidate merged due fields, parent cycles, and same-owner references in the transaction. |
| `DELETE /v1/tasks/{id}` | Require `If-Match`; delete task/subtree and return `204`. Missing/foreign `404`, stale version `409`. |
| `GET /v1/tasks` | Filter, sort, and keyset paginate owned tasks; return `{tasks,nextCursor}`. Default excludes completed/cancelled unless `status=all` is requested. |
| `POST /v1/task-tags`, `GET /v1/task-tags`, `PATCH /v1/task-tags/{id}`, `DELETE /v1/task-tags/{id}` | Manage per-user tags. Rename keeps associations; delete removes assignments. Task PATCH handles association replacement. |
| `POST /v1/tasks/{id}/comments`, `GET /v1/tasks/{id}/comments` | Append a comment and page comments newest-first. The author is the current user. |
| `PATCH /v1/tasks/{id}/comments/{commentId}`, `DELETE /v1/tasks/{id}/comments/{commentId}` | Edit/delete a comment by its author; owner-scoped in SQL, `204` on delete. |
| `POST /v1/tasks/{id}/links`, `PATCH /v1/tasks/{id}/links/{linkId}`, `DELETE /v1/tasks/{id}/links/{linkId}` | Add/edit/remove web or note links; return committed link on create/update, `204` on delete. |

`GET /v1/tasks` accepts bounded, combinable query parameters: `spaceId` (a Space ID, `none`, or omitted for all owned tasks), `status` (repeatable or `all`), `priority`, `flagged`, `tagId` (repeatable with `tagMode=all|any`), `parentId` (including `none`), `hasDueDate`, `dueKind=date|timed`, `dueFrom`/`dueBefore` (half-open date or instant windows; require `dueKind` and matching types), and `completedFrom`/`completedBefore`. Support `sort=created|due|priority|manual` with a fixed secondary `id` tie-breaker; `sort=due` requires `dueKind` so local dates and instants are never conflated, and `manual` requires a Space or parent filter. `limit` defaults to 50 and is capped at 100. Cursor encodes sort/filter values and the last row's sort key; reject mismatched cursor/filter combinations. User ownership is the first predicate for every list and subresource query. `GET /v1/tasks?status=done` is the Completed view; Today/Overdue are composed from date-only and timed queries using the caller's explicit timezone, not the service host timezone. Add `q` only with a tested full-text index and clear search semantics. No offset pagination.

Existing domains gain the same optional Space contract in their own routes: document upload/jobs accept `spaceId`, document list/search accept `spaceId`, and an owner-scoped document PATCH can move a file; calendar create/list accept `spaceId`, with a later owner-scoped event PATCH for moving existing events. After conversation ownership is fixed, chat creation accepts `spaceId`, conversation list filters by it, and an owner-scoped conversation PATCH moves a chat. Omitted filter means all of the user's resources; `spaceId=none` means standalone. A request with a foreign Space ID must not reveal or attach that Space. Keep current paths flat; `/v1/spaces/{id}` is the Space resource, not a bypass around each domain's owner checks.

Example create payload:

```json
{
  "title": "Review the draft",
  "description": "Check the references",
  "spaceId": "space-uuid",
  "priority": "high",
  "flagged": true,
  "dueOn": "2026-10-02",
  "tagIds": ["tag-uuid"],
  "links": [
    {"kind": "web", "label": "Brief", "url": "https://example.com/brief"},
    {"kind": "curio_note", "label": "Draft note", "noteId": "note-uuid"}
  ]
}
```

Validate web URLs as `http` or `https`, reject credentials and control characters, cap length, and never fetch them server-side. The client must use its platform external-link capability when opening them. Bound titles/descriptions/comments/tag names/link labels, association counts, request body size, query count, and nesting depth before allocating or querying. Use a transaction for aggregate writes and parameterized SQL for every value; IDs in a payload are untrusted even when syntactically valid.

## Implementation sequence

1. **Shared Spaces foundation:** implement the separate [Spaces API plan](spaces-api-plan.md) first. It provides the `spaces` table and protected create/list/delete routes. Its first slice has no resource memberships or update route.
2. **Tasks schema and domain:** add the five Tasks tables, constraints, and indexes in a later append-only migration; update catalog verification. Create `src/domains/tasks/{mod,model,route,handler,service,repository}.rs`. Define validated create/patch/filter types and feature errors in `model.rs`; `service.rs` owns transitions, cycles, limits, and same-owner validation; `repository.rs` owns bound SQL, transactions, and hydration. Add `tasks` to `domains/mod.rs` and construct/protect its routes in `app/bootstrap.rs`. Keep feature SQL out of `Database`.
3. **Core Tasks routes:** implement tags and task create/get/list/patch/delete with owner-scoped SQL and version checks. Complete/reopen are status PATCH transitions: setting `done` stamps `completedAt`; reopening clears it. Preserve `createdAt`, advance `updatedAt`, and return the committed record.
4. **Subresources:** add comment and link routes. Ensure parent task ownership is checked inside each SQL operation or transaction, not through a prior unscoped read. Each subresource mutation increments task `version` and returns the new `taskVersion` (including delete responses as a header), so task detail caches can invalidate predictably. These append/edit operations do not require `If-Match`; task PATCH/DELETE do.
5. **Existing owned domains:** migrate `calendar_events` and `document_files` with nullable `space_id` and composite owner/Space FKs. Add Space assignment and filters to calendar and document routes while preserving their existing owner/date/search contracts. Document search must join or otherwise validate the current owned file before filtering by Space. Existing records remain standalone. Check the SQLite importer against the migrated schema: imported old records should have null `space_id` and should not change the archived source model.
6. **Chats:** first introduce conversation ownership and protect chat/history routes, updating the client and tests together. Legacy conversations have no trustworthy owner: leave them inaccessible until an explicit, verified reconciliation, never assign them to the first requester. Only then add optional `space_id`, composite FK, create/list/move behavior, and Space detachment for owned conversations. Do not publish Space-scoped chat APIs while global reads remain possible.
7. **Client contract when consumed:** add `web/src/api/tasks.ts` with independent wire types and narrow response validation; use a React Query owner for task keys/invalidation. The Spaces client belongs to its separate plan. Keep current calendar projections until a deliberate UI migration. Add a Notes resolver only after Notes has a real owned API. Do not expand Tauri's platform contract for ordinary JSON routes.
8. **Documentation and rollout:** update `docs/ARCHITECTURE.md` only when behavior is implemented; update deployment notes if migration ordering or operational steps change. Apply DDL through `curio_db migrate` with migrator credentials before starting a service build that expects each new migration. Do not edit applied migrations or run migrations on a persistent target as an experiment.

## Verification and acceptance

- Pure domain tests: field bounds, status/completion transition, due date versus timed instant, null/omitted PATCH semantics, tag normalization, URL protocol, parent cycle/depth, and filter parsing.
- Full-router tests: every route's bearer requirement, status/body shape, version conflict, bad input, empty page, pagination stability, and same `404` for missing/foreign IDs. Specifically test forged body/query owner IDs are rejected/ignored.
- Cross-domain router tests when Space support is added: standalone versus Space-filtered document and calendar results; foreign Space rejection on create/move; protected chat/history and owner-scoped conversation/message reads before chat Space assignment; legacy unowned conversations remain inaccessible.
- PostgreSQL fixture tests in disposable `curio_test_*` schema: same-owner composite FK enforcement, cross-user Space/tag/parent/link/comment rejection, association replacement atomicity, Space deletion leaving tasks/documents/events/conversations standalone, subtree deletion, reconnect durability, unique tag/Space names, and index-backed filtering. Verify transactions roll back on partial invalid payloads.
- Contract consumer tests when the web API is added: exact paths, headers, JSON casing, date-only preservation, desktop raw-byte bridge behavior, and cache invalidation across filters.
- Run from `curio-service`: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` with a safe `CURIO_TEST_DATABASE_URL`; importer tests if the migration changes compatibility expectations. Run `curio_db verify` only against an explicitly safe migrated schema. Run `git diff --check` and compare the implementation with `docs/ARCHITECTURE.md` before handoff. A green test run without the database variable is partial evidence.

Acceptance: two distinct users can create identically named Spaces/tags and cannot read or mutate one another's Spaces, tasks, or subresources; unspaced tasks work; tags/comments/links survive a reconnect; due and status filters return deterministic bounded pages; every mutation has defined concurrency and deletion behavior. Documents, calendar events, and securely owned conversations can each be placed in or left outside a Space, filtered by Space, and retained when their Space is deleted. Calendar date behavior remains unchanged.

## Follow-up capabilities, with explicit gates

- **Repeat and reminders:** add a structured recurrence table/rule with timezone, start anchor, interval, weekday/month constraints, and end condition. Define whether completion generates the next occurrence or marks one occurrence of a series; test DST and missed occurrences. Add a `task_alerts` schedule/outbox with idempotent delivery and a worker before promising notifications. Early reminders and urgent alarms depend on that delivery system.
- **Location/Messages triggers:** require client permission, a provider/device integration, and privacy rules. Do not imply that storing coordinates or contact identifiers causes a trigger.
- **Attachments:** link to owned `document_files` or a dedicated blob resource after deciding size, quota, download authorization, deletion, and malware/media policy. Avoid a raw external-media URL shortcut.
- **Saved Smart Lists, sections, templates, and grocery categorization:** start with the query filters above; add saved filter definitions and Space sections/order only after client needs are clear. Grocery auto-categorization and AI suggestions are separate classifiers, not task CRUD.
- **Sharing and assignment:** require verified identity, membership/role schema, owner-versus-assignee policy, audit behavior, and every read/write query updated for membership. Do not infer collaborators from `task_comments.author_id`.
- **Recently Deleted/recovery:** add soft deletion and retention for a task subtree and its associations as a separate, tested lifecycle; v1 hard deletion is intentional.
