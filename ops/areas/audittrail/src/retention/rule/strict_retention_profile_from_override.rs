use super::*;

use std::collections::BTreeMap;

use crate::retention::variance as retention_variance;

pub fn summarize_primary_retention_overrides_for_profile(
    items: &[retention_variance::PrimaryRetentionOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_strict_retention_profiles_from_override(
    items: &[retention_variance::PrimaryRetentionOverride],
) -> Vec<StrictRetentionProfile> {
    let mut out: Vec<StrictRetentionProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_retention_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_retention_profile(item: &StrictRetentionProfile) -> String {
    retention_variance::classify_retention_variance(item.adjusted_value())
}
