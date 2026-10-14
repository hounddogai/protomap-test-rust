use super::*;

use std::collections::BTreeMap;

use crate::usage::usage_estimate as usage_usage_estimate;

pub fn summarize_usage_overrides_for_tier(items: &[usage_usage_estimate::UsageOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_grant_tiers_from_override(items: &[usage_usage_estimate::UsageOverride]) -> Vec<GrantTier> {
    let mut out: Vec<GrantTier> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_grant_tier(item: &GrantTier) -> String {
    usage_usage_estimate::normalize_usage_override_code(&item.code)
}
