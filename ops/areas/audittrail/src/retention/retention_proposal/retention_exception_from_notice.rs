use super::*;

use std::collections::BTreeMap;

use crate::query::query_notice as query_query_notice;

pub fn summarize_tiered_query_notices_for_exception(
    items: &[query_query_notice::TieredQueryNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_retention_exceptions_from_notice(
    items: &[query_query_notice::TieredQueryNotice],
) -> Vec<RetentionException> {
    let mut out: Vec<RetentionException> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_retention_exception(item: &RetentionException) -> String {
    query_query_notice::classify_weighted_query_rollup(item.weighted_exposure())
}
