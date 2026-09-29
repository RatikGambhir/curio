//! Private SQL fragments, identifier checks and deferred SQLx encoding.
use super::super::super::BoundSql;
use super::Expr;
use sqlx::{Postgres, QueryBuilder};

type Binder<'a> = Box<dyn FnOnce(&mut BoundSql) + Send + 'a>;
pub(super) enum Part<'a> {
    Sql(&'static str),
    Identifier(&'static str),
    Integer(i64),
    Bind(Binder<'a>),
    Invalid(&'static str),
}

impl<'a> Expr<'a> {
    fn write(self, query: &mut BoundSql) {
        for part in self.parts {
            match part {
                Part::Sql(sql) => {
                    query.push(sql);
                }
                Part::Identifier(name) if valid_identifier(name) => {
                    query.push(name);
                }
                Part::Identifier(_) => set_error(query, "invalid SQL identifier"),
                Part::Integer(value) => {
                    query.push(&value.to_string());
                }
                Part::Bind(bind) => bind(query),
                Part::Invalid(reason) => set_error(query, reason),
            }
        }
    }
    pub(in super::super) fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        let mut query = BoundSql::new("");
        self.write(&mut query);
        query.build()
    }
}
fn set_error(query: &mut BoundSql, reason: &'static str) {
    if query.error.is_none() {
        query.error = Some(sqlx::Error::Protocol(reason.to_owned()));
    }
}
fn valid_identifier(name: &str) -> bool {
    name.split('.').all(|part| {
        part == "*"
            || (!part.is_empty()
                && part.as_bytes().iter().enumerate().all(|(i, byte)| {
                    byte.is_ascii_alphabetic() || *byte == b'_' || (i > 0 && byte.is_ascii_digit())
                }))
    })
}
pub(in super::super) fn invalid<'a>(reason: &'static str) -> Expr<'a> {
    Expr {
        parts: vec![Part::Invalid(reason)],
    }
}
pub(in super::super) fn list<'a>(items: impl IntoIterator<Item = Expr<'a>>) -> Expr<'a> {
    let mut expr = Expr::sql("");
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            expr = expr.push(", ");
        }
        expr = expr.append(item);
    }
    expr
}
