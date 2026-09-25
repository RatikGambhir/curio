use sqlx::Error;
use tracing::error;
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_FILTER: &str = "curio_service=info";

pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    // Tests or embedding applications may already have installed a subscriber.
    // In that case, retaining the existing subscriber is safer than panicking.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .try_init();
}

pub(crate) fn log_database_error(operation: &'static str, error: &Error) {
    let database_error = error.as_database_error();
    let database_code = database_error
        .and_then(|error| error.code())
        .map(|code| code.into_owned());
    let constraint = database_error
        .and_then(|error| error.constraint())
        .map(str::to_owned);

    error!(
        operation,
        error_kind = sqlx_error_kind(error),
        database_code = ?database_code,
        constraint = ?constraint,
        "database operation failed"
    );
}

fn sqlx_error_kind(error: &Error) -> &'static str {
    match error {
        Error::Configuration(_) => "configuration",
        Error::InvalidArgument(_) => "invalid_argument",
        Error::Database(_) => "database",
        Error::Io(_) => "io",
        Error::Tls(_) => "tls",
        Error::Protocol(_) => "protocol",
        Error::RowNotFound => "row_not_found",
        Error::TypeNotFound { .. } => "type_not_found",
        Error::ColumnIndexOutOfBounds { .. } => "column_index_out_of_bounds",
        Error::ColumnNotFound(_) => "column_not_found",
        Error::ColumnDecode { .. } => "column_decode",
        Error::Encode(_) => "encode",
        Error::Decode(_) => "decode",
        Error::AnyDriverError(_) => "driver",
        Error::PoolTimedOut => "pool_timed_out",
        Error::PoolClosed => "pool_closed",
        Error::WorkerCrashed => "worker_crashed",
        Error::Migrate(_) => "migration",
        Error::InvalidSavePointStatement => "invalid_savepoint",
        Error::BeginFailed => "transaction_begin",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::sqlx_error_kind;

    #[test]
    fn database_error_kinds_are_stable_and_sanitized() {
        assert_eq!(
            sqlx_error_kind(&sqlx::Error::PoolTimedOut),
            "pool_timed_out"
        );
        assert_eq!(sqlx_error_kind(&sqlx::Error::RowNotFound), "row_not_found");
    }
}
