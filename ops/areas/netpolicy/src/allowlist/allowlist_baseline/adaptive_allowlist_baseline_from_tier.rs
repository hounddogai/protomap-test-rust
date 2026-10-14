use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_plan as peering_peering_plan;

pub fn summarize_rolling_peering_tiers_for_baseline(
    items: &[peering_peering_plan::RollingPeeringTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_adaptive_allowlist_baselines_from_tier(
    items: &[peering_peering_plan::RollingPeeringTier],
) -> Vec<AdaptiveAllowlistBaseline> {
    let mut out: Vec<AdaptiveAllowlistBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_allowlist_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_allowlist_baseline(item: &AdaptiveAllowlistBaseline) -> f64 {
    peering_peering_plan::scheduled_peering_metric_weight(5, true)
}
