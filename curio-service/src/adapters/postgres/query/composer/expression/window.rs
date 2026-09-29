//! Window-function partition and ordering clauses.
use super::super::Order;
use super::{Expr, list};

impl<'a> Expr<'a> {
    /// Window functions can partition and order without interpolated clauses.
    pub fn over(
        self,
        partition: impl IntoIterator<Item = Self>,
        order: impl IntoIterator<Item = Order<'a>>,
    ) -> Self {
        let partition: Vec<_> = partition.into_iter().collect();
        let order: Vec<_> = order.into_iter().map(|order| order.0).collect();
        let has_partition = !partition.is_empty();
        let mut expr = self.push(" OVER (");
        if has_partition {
            expr = expr.push("PARTITION BY ").append(list(partition));
        }
        if !order.is_empty() {
            if has_partition {
                expr = expr.push(" ");
            }
            expr = expr.push("ORDER BY ").append(list(order));
        }
        expr.push(")")
    }
}
