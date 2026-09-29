/// Declare a code-owned column enum without repeating its SQL mapping.
#[macro_export]
macro_rules! sql_columns {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident => $sql:literal),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $vis enum $name { $($variant),* }
        impl $crate::adapters::postgres::query::SqlColumn for $name {
            fn sql(self) -> &'static str {
                match self { $(Self::$variant => $sql),* }
            }
        }
    };
}
pub use crate::sql_columns;
