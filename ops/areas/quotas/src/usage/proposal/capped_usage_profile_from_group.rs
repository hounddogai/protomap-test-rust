use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_mapping as limit_limit_mapping;

pub fn summarize_rolling_limit_groups_for_profile(
    items: &[limit_limit_mapping::RollingLimitGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_capped_usage_profiles_from_group(
    items: &[limit_limit_mapping::RollingLimitGroup],
) -> Vec<CappedUsageProfile> {
    let mut out: Vec<CappedUsageProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_usage_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_capped_usage_profile(item: &CappedUsageProfile) -> String {
    limit_limit_mapping::batched_limit_mapping_bucket(item.net_value())
}
