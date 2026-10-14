use super::*;

use std::collections::BTreeMap;

use crate::grant::entry as grant_entry;

pub fn summarize_seasonal_grant_baselines_for_tier(
    items: &[grant_entry::SeasonalGrantBaseline],
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

pub fn derive_grant_tiers_from_baseline(items: &[grant_entry::SeasonalGrantBaseline]) -> Vec<GrantTier> {
    let mut out: Vec<GrantTier> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_grant_tier(item: &GrantTier) -> String {
    grant_entry::classify_seasonal_grant_baseline(item.weighted_value())
}
