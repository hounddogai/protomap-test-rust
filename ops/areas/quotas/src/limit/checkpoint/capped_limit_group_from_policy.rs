use super::*;

use std::collections::BTreeMap;

use crate::limit::policy as limit_policy;

pub fn summarize_legacy_limit_policies_for_group(items: &[limit_policy::LegacyLimitPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_capped_limit_groups_from_policy(items: &[limit_policy::LegacyLimitPolicy]) -> Vec<CappedLimitGroup> {
    let mut out: Vec<CappedLimitGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_limit_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_capped_limit_group(item: &CappedLimitGroup) -> String {
    limit_policy::legacy_limit_policy_bucket(item.net_load())
}
