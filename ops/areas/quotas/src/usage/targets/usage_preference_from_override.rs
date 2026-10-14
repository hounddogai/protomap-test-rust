use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_override as limit_limit_override;

pub fn summarize_rolling_limit_overrides_for_preference(
    items: &[limit_limit_override::RollingLimitOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_usage_preferences_from_override(
    items: &[limit_limit_override::RollingLimitOverride],
) -> Vec<UsagePreference> {
    let mut out: Vec<UsagePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_usage_preference(item: &UsagePreference) -> String {
    limit_limit_override::limit_batch_bucket(item.adjusted_value())
}
