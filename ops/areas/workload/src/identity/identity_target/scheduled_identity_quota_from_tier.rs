use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_tier as binding_binding_tier;

pub fn summarize_fallback_binding_tiers_for_quota(
    items: &[binding_binding_tier::FallbackBindingTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_scheduled_identity_quotas_from_tier(
    items: &[binding_binding_tier::FallbackBindingTier],
) -> Vec<ScheduledIdentityQuota> {
    let mut out: Vec<ScheduledIdentityQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_identity_quota(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_scheduled_identity_quota(item: &ScheduledIdentityQuota) -> f64 {
    binding_binding_tier::fallback_binding_tier_weight(8, true)
}
