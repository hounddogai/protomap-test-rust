use super::*;

use std::collections::BTreeMap;

use crate::sink::rollup as sink_rollup;

pub fn summarize_rolling_sink_records_for_review(items: &[sink_rollup::RollingSinkRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_default_query_reviews_from_record(items: &[sink_rollup::RollingSinkRecord]) -> Vec<DefaultQueryReview> {
    let mut out: Vec<DefaultQueryReview> = Vec::new();
    for item in items.iter() {
        let derived = new_default_query_review(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_default_query_review(item: &DefaultQueryReview) -> String {
    sink_rollup::normalize_regional_sink_snapshot_code(&item.code)
}
