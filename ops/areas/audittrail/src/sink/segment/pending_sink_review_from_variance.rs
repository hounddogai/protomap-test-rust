use super::*;

use std::collections::BTreeMap;

use crate::sink::tier as sink_tier;

pub fn summarize_capped_sink_variances_for_review(items: &[sink_tier::CappedSinkVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_pending_sink_reviews_from_variance(items: &[sink_tier::CappedSinkVariance]) -> Vec<PendingSinkReview> {
    let mut out: Vec<PendingSinkReview> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_sink_review(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_pending_sink_review(item: &PendingSinkReview) -> f64 {
    sink_tier::sink_tier_weight(6, true)
}
