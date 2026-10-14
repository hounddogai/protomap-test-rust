use super::*;

use std::collections::BTreeMap;

use crate::limit::quota as limit_quota;

pub fn summarize_lenient_limit_ledgers_for_segment(items: &[limit_quota::LenientLimitLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_adaptive_usage_segments_from_ledger(
    items: &[limit_quota::LenientLimitLedger],
) -> Vec<AdaptiveUsageSegment> {
    let mut out: Vec<AdaptiveUsageSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_usage_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_adaptive_usage_segment(item: &AdaptiveUsageSegment) -> String {
    limit_quota::classify_manual_limit_budget(item.adjusted_exposure())
}
