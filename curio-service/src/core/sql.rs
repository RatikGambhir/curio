//! Bind-first SQL construction shared by the service's repositories.
//!
//! A hand-written statement keeps three parallel lists in sync: the column
//! list, the `$n` placeholders, and the argument order. Every edit has to touch
//! all three, and a mismatch still compiles — it fails, or silently binds the
//! wrong column, at runtime.
//!
//! Here, binding a value is what produces its placeholder, so those lists
//! cannot drift apart. Each column or filter is written exactly once, next to
//! the value it carries.
//!
//! Statement structure is `&'static str`, so only literals and constants reach
//! the SQL text; every caller-supplied value is bound.

use sqlx::{
    Arguments, Encode, Executor, FromRow, Postgres, Type,
    error::BoxDynError,
    postgres::{PgArguments, PgQueryResult, PgRow},
};

/// A value that can be bound to a statement parameter.
pub trait Bind<'a>: Encode<'a, Postgres> + Type<Postgres> + Send + 'a {}

impl<'a, T> Bind<'a> for T where T: Encode<'a, Postgres> + Type<Postgres> + Send + 'a {}

/// Entry point for the statement shapes the repositories use.
pub struct Sql;

impl Sql {
    /// Starts `SELECT <columns> FROM …`.
    pub fn select(columns: &'static str) -> SelectColumns {
        SelectColumns { columns }
    }

    /// Starts `INSERT INTO <table> …`.
    pub fn insert_into(table: &'static str) -> Insert {
        Insert {
            table,
            columns: Vec::new(),
            values: Vec::new(),
            conflict_key: None,
            binds: Binds::default(),
        }
    }

    /// Starts `UPDATE <table> SET …`.
    pub fn update(table: &'static str) -> Update {
        Update {
            table,
            assignments: Vec::new(),
            filters: Filters::default(),
            binds: Binds::default(),
        }
    }
}

/// Accumulates bound values and hands out the placeholder for each one.
#[derive(Default)]
struct Binds {
    arguments: PgArguments,
    count: usize,
    failure: Option<BoxDynError>,
}

impl Binds {
    /// Binds `value` and returns the placeholder that now refers to it.
    fn placeholder<'a>(&mut self, value: impl Bind<'a>) -> String {
        if let Err(failure) = self.arguments.add(value) {
            self.failure.get_or_insert(failure);
        }
        self.count += 1;
        format!("${}", self.count)
    }
}

/// `WHERE` conjuncts, each already carrying its own placeholder.
#[derive(Default)]
struct Filters(Vec<String>);

impl Filters {
    fn render(&self) -> String {
        if self.0.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.0.join(" AND "))
        }
    }
}

/// Substitutes the single `?` in a filter fragment with a real placeholder.
fn apply(fragment: &'static str, placeholder: &str) -> String {
    debug_assert_eq!(
        fragment.matches('?').count(),
        1,
        "a bound filter fragment needs exactly one `?`"
    );
    fragment.replacen('?', placeholder, 1)
}

/// `SELECT <columns>` awaiting its table.
pub struct SelectColumns {
    columns: &'static str,
}

impl SelectColumns {
    pub fn from(self, table: &'static str) -> Select {
        Select {
            columns: self.columns,
            table,
            filters: Filters::default(),
            order: None,
            binds: Binds::default(),
        }
    }
}

/// `SELECT … FROM … WHERE … ORDER BY …`.
pub struct Select {
    columns: &'static str,
    table: &'static str,
    filters: Filters,
    order: Option<&'static str>,
    binds: Binds,
}

impl Select {
    /// Adds a conjunct, binding `value` in place of its `?`.
    pub fn filter<'a>(mut self, fragment: &'static str, value: impl Bind<'a>) -> Self {
        let placeholder = self.binds.placeholder(value);
        self.filters.0.push(apply(fragment, &placeholder));
        self
    }

    pub fn order_by(mut self, order: &'static str) -> Self {
        self.order = Some(order);
        self
    }

    pub(crate) fn statement(self) -> Statement {
        let order = self
            .order
            .map(|order| format!(" ORDER BY {order}"))
            .unwrap_or_default();
        Statement {
            sql: format!(
                "SELECT {} FROM {}{}{}",
                self.columns,
                self.table,
                self.filters.render(),
                order
            ),
            binds: self.binds,
        }
    }

    pub async fn fetch_all<'e, T>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        self.statement().fetch_all(executor).await
    }
}

/// `INSERT INTO … VALUES …`, optionally an upsert.
pub struct Insert {
    table: &'static str,
    columns: Vec<&'static str>,
    values: Vec<String>,
    conflict_key: Option<&'static str>,
    binds: Binds,
}

impl Insert {
    /// Adds a column and binds its value.
    pub fn set<'a>(mut self, column: &'static str, value: impl Bind<'a>) -> Self {
        let placeholder = self.binds.placeholder(value);
        self.columns.push(column);
        self.values.push(placeholder);
        self
    }

    /// Adds a column whose value is a literal SQL expression, such as `'user'`.
    pub fn set_literal(mut self, column: &'static str, expression: &'static str) -> Self {
        self.columns.push(column);
        self.values.push(expression.to_owned());
        self
    }

    /// On a conflict of `key`, refreshes every other column from the proposed
    /// row and stamps `updated_at`. The table must have an `updated_at` column.
    ///
    /// `updated_at` is stamped here rather than copied from the proposed row,
    /// so a caller that also sets it explicitly does not produce the duplicate
    /// assignment PostgreSQL rejects.
    pub fn upsert_on(mut self, key: &'static str) -> Self {
        self.conflict_key = Some(key);
        self
    }

    /// Finishes the statement with `RETURNING <columns>`.
    pub fn returning(self, columns: &'static str) -> Statement {
        self.finish(Some(columns))
    }

    pub(crate) fn statement(self) -> Statement {
        self.finish(None)
    }

    fn finish(self, returning: Option<&'static str>) -> Statement {
        let conflict = match self.conflict_key {
            Some(key) => {
                let refreshed = self
                    .columns
                    .iter()
                    .filter(|column| **column != key && **column != UPDATED_AT)
                    .map(|column| format!("{column} = EXCLUDED.{column}"))
                    .chain([format!("{UPDATED_AT} = {NOW}")])
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(" ON CONFLICT ({key}) DO UPDATE SET {refreshed}")
            }
            None => String::new(),
        };
        let returning = returning
            .map(|columns| format!(" RETURNING {columns}"))
            .unwrap_or_default();
        Statement {
            sql: format!(
                "INSERT INTO {} ({}) VALUES ({}){}{}",
                self.table,
                self.columns.join(", "),
                self.values.join(", "),
                conflict,
                returning
            ),
            binds: self.binds,
        }
    }

    pub async fn execute<'e>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        self.statement().execute(executor).await
    }
}

/// `UPDATE … SET … WHERE …`.
pub struct Update {
    table: &'static str,
    assignments: Vec<String>,
    filters: Filters,
    binds: Binds,
}

impl Update {
    /// Assigns a column and binds its value.
    pub fn set<'a>(mut self, column: &'static str, value: impl Bind<'a>) -> Self {
        let placeholder = self.binds.placeholder(value);
        self.assignments.push(format!("{column} = {placeholder}"));
        self
    }

    /// Assigns a column the current transaction timestamp.
    pub fn set_now(mut self, column: &'static str) -> Self {
        self.assignments.push(format!("{column} = {NOW}"));
        self
    }

    /// Adds a conjunct, binding `value` in place of its `?`.
    pub fn filter<'a>(mut self, fragment: &'static str, value: impl Bind<'a>) -> Self {
        let placeholder = self.binds.placeholder(value);
        self.filters.0.push(apply(fragment, &placeholder));
        self
    }

    /// Adds a conjunct that binds nothing, such as `role = 'assistant'`.
    pub fn filter_literal(mut self, fragment: &'static str) -> Self {
        self.filters.0.push(fragment.to_owned());
        self
    }

    pub(crate) fn statement(self) -> Statement {
        Statement {
            sql: format!(
                "UPDATE {} SET {}{}",
                self.table,
                self.assignments.join(", "),
                self.filters.render()
            ),
            binds: self.binds,
        }
    }

    pub async fn execute<'e>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        self.statement().execute(executor).await
    }

    /// Executes the statement, requiring that it matched exactly one row.
    pub async fn execute_one<'e>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        self.statement().execute_one(executor).await
    }
}

/// A finished statement and the arguments its placeholders refer to.
pub struct Statement {
    sql: String,
    binds: Binds,
}

impl Statement {
    /// The rendered statement text, for assertions that do not need a database.
    #[cfg(test)]
    pub(crate) fn sql(&self) -> &str {
        &self.sql
    }

    /// Splits the statement into the parts `sqlx` needs, surfacing a bind that
    /// failed to encode instead of sending an incomplete argument list.
    fn parts(self) -> Result<(String, PgArguments), sqlx::Error> {
        match self.binds.failure {
            Some(failure) => Err(sqlx::Error::Encode(failure)),
            None => Ok((self.sql, self.binds.arguments)),
        }
    }

    pub async fn fetch_one<'e, T>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<T, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        let (sql, arguments) = self.parts()?;
        sqlx::query_as_with::<_, T, _>(&sql, arguments)
            .fetch_one(executor)
            .await
    }

    pub async fn fetch_all<'e, T>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        let (sql, arguments) = self.parts()?;
        sqlx::query_as_with::<_, T, _>(&sql, arguments)
            .fetch_all(executor)
            .await
    }

    pub async fn execute<'e>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let (sql, arguments) = self.parts()?;
        sqlx::query_with(&sql, arguments).execute(executor).await
    }

    /// Executes the statement, requiring that it matched exactly one row.
    pub async fn execute_one<'e>(
        self,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        match self.execute(executor).await?.rows_affected() {
            1 => Ok(()),
            _ => Err(sqlx::Error::RowNotFound),
        }
    }
}

const NOW: &str = "CURRENT_TIMESTAMP";
const UPDATED_AT: &str = "updated_at";
