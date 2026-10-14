use super::*;

use std::collections::BTreeMap;

use crate::usage::rollup as usage_rollup;

pub fn summarize_usage_rollups_for_tier(items: &[usage_rollup::UsageRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_regional_usage_tiers_from_rollup(items: &[usage_rollup::UsageRollup]) -> Vec<RegionalUsageTier> {
    let mut out: Vec<RegionalUsageTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_usage_tier(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_regional_usage_tier(item: &RegionalUsageTier) -> String {
    usage_rollup::normalize_seasonal_usage_variance_code(&item.code)
}
