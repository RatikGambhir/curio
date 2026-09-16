use crate::core::sql::Sql;

#[test]
fn numbers_placeholders_in_bind_order() {
    let statement = Sql::insert_into("tasks")
        .set("id", "task-a")
        .set("user_id", "user-a")
        .set("active", true)
        .returning("id, user_id, active");

    assert_eq!(
        statement.sql(),
        "INSERT INTO tasks (id, user_id, active) VALUES ($1, $2, $3) \
         RETURNING id, user_id, active"
    );
}

#[test]
fn keeps_literal_columns_out_of_the_placeholder_sequence() {
    let statement = Sql::insert_into("messages")
        .set("id", "message-a")
        .set_literal("role", "'user'")
        .set("content", "hello")
        .statement();

    assert_eq!(
        statement.sql(),
        "INSERT INTO messages (id, role, content) VALUES ($1, 'user', $2)"
    );
}

#[test]
fn refreshes_every_non_key_column_on_conflict() {
    let statement = Sql::insert_into("users")
        .set("id", "user-a")
        .set("name", "Ada")
        .set("email", "ada@example.com")
        .upsert_on("id")
        .returning("id, name, email");

    assert_eq!(
        statement.sql(),
        "INSERT INTO users (id, name, email) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email, \
         updated_at = CURRENT_TIMESTAMP RETURNING id, name, email"
    );
}

#[test]
fn stamps_updated_at_when_an_upsert_carries_only_the_key() {
    let statement = Sql::insert_into("conversations")
        .set("id", "conversation-a")
        .upsert_on("id")
        .statement();

    assert_eq!(
        statement.sql(),
        "INSERT INTO conversations (id) VALUES ($1) \
         ON CONFLICT (id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP"
    );
}

#[test]
fn joins_filters_in_written_order() {
    let statement = Sql::select("id, starts_at")
        .from("calendar_events")
        .filter("user_id = ?", "user-a")
        .filter("starts_at < ?", "2026-09-02")
        .filter("ends_at > ?", "2026-09-01")
        .order_by("starts_at, id")
        .statement();

    assert_eq!(
        statement.sql(),
        "SELECT id, starts_at FROM calendar_events \
         WHERE user_id = $1 AND starts_at < $2 AND ends_at > $3 \
         ORDER BY starts_at, id"
    );
}

#[test]
fn numbers_update_assignments_before_filters() {
    let statement = Sql::update("messages")
        .set("content", "done")
        .set("status", "completed")
        .set_now("updated_at")
        .filter("id = ?", "message-a")
        .filter_literal("role = 'assistant'")
        .statement();

    assert_eq!(
        statement.sql(),
        "UPDATE messages SET content = $1, status = $2, updated_at = CURRENT_TIMESTAMP \
         WHERE id = $3 AND role = 'assistant'"
    );
}

#[test]
fn omits_the_where_clause_when_nothing_is_filtered() {
    let statement = Sql::select("id")
        .from("conversations")
        .order_by("updated_at DESC, id")
        .statement();

    assert_eq!(
        statement.sql(),
        "SELECT id FROM conversations ORDER BY updated_at DESC, id"
    );
}
