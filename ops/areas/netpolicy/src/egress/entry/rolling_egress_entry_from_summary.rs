use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_summary as peering_peering_summary;

pub fn summarize_tiered_peering_summaries_for_entry(
    items: &[peering_peering_summary::TieredPeeringSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_rolling_egress_entries_from_summary(
    items: &[peering_peering_summary::TieredPeeringSummary],
) -> Vec<RollingEgressEntry> {
    let mut out: Vec<RollingEgressEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_egress_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_egress_entry(item: &RollingEgressEntry) -> String {
    peering_peering_summary::normalize_weighted_peering_review_code(&item.code)
}
