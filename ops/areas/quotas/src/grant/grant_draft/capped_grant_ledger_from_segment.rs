use super::*;

use std::collections::BTreeMap;

use crate::usage::segment as usage_segment;

pub fn summarize_regional_usage_segments_for_ledger(
    items: &[usage_segment::RegionalUsageSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_capped_grant_ledgers_from_segment(
    items: &[usage_segment::RegionalUsageSegment],
) -> Vec<CappedGrantLedger> {
    let mut out: Vec<CappedGrantLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_grant_ledger(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_grant_ledger(item: &CappedGrantLedger) -> String {
    usage_segment::normalize_rolling_usage_checkpoint_code(&item.code)
}
