use super::*;

use std::collections::BTreeMap;

use crate::policy::metric as policy_metric;

pub fn summarize_strict_policy_items_for_digest(items: &[policy_metric::StrictPolicyItem]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_scheduled_policy_digests_from_item(
    items: &[policy_metric::StrictPolicyItem],
) -> Vec<ScheduledPolicyDigest> {
    let mut out: Vec<ScheduledPolicyDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_policy_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_scheduled_policy_digest(item: &ScheduledPolicyDigest) -> String {
    policy_metric::normalize_policy_tier_code(&item.code)
}
