use super::*;

use std::collections::BTreeMap;

use crate::probe::slot as probe_slot;

pub fn summarize_probe_estimates_for_tier(items: &[probe_slot::ProbeEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_slo_tiers_from_estimate(items: &[probe_slot::ProbeEstimate]) -> Vec<SloTier> {
    let mut out: Vec<SloTier> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_slo_tier(item: &SloTier) -> String {
    probe_slot::probe_slot_bucket(item.weighted_load())
}
