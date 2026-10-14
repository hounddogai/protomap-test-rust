use super::*;

use std::collections::BTreeMap;

use crate::grant::override_item as grant_override_item;

pub fn summarize_grant_overrides_for_estimate(items: &[grant_override_item::GrantOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rolling_usage_estimates_from_override(
    items: &[grant_override_item::GrantOverride],
) -> Vec<RollingUsageEstimate> {
    let mut out: Vec<RollingUsageEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_usage_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_usage_estimate(item: &RollingUsageEstimate) -> f64 {
    grant_override_item::rolling_grant_agreement_weight(3, true)
}
