use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_tier as binding_binding_tier;

pub fn summarize_primary_binding_quotas_for_quota(
    items: &[binding_binding_tier::PrimaryBindingQuota],
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

pub fn derive_pending_binding_quotas_from_quota(
    items: &[binding_binding_tier::PrimaryBindingQuota],
) -> Vec<PendingBindingQuota> {
    let mut out: Vec<PendingBindingQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_binding_quota(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_pending_binding_quota(item: &PendingBindingQuota) -> f64 {
    binding_binding_tier::fallback_binding_tier_weight(7, true)
}
