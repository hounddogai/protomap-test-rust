use super::*;

use std::collections::BTreeMap;

use crate::allowlist::adjustment as allowlist_adjustment;

pub fn summarize_allowlist_metrics_for_tier(items: &[allowlist_adjustment::AllowlistMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rolling_peering_tiers_from_metric(
    items: &[allowlist_adjustment::AllowlistMetric],
) -> Vec<RollingPeeringTier> {
    let mut out: Vec<RollingPeeringTier> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_peering_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_peering_tier(item: &RollingPeeringTier) -> String {
    allowlist_adjustment::normalize_manual_allowlist_bundle_code(&item.code)
}
