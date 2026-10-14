use super::*;

use std::collections::BTreeMap;

use crate::identity::milestone as identity_milestone;

pub fn summarize_rolling_identity_milestones_for_tier(
    items: &[identity_milestone::RollingIdentityMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_fallback_binding_tiers_from_milestone(
    items: &[identity_milestone::RollingIdentityMilestone],
) -> Vec<FallbackBindingTier> {
    let mut out: Vec<FallbackBindingTier> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_binding_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_fallback_binding_tier(item: &FallbackBindingTier) -> String {
    identity_milestone::normalize_legacy_identity_ledger_code(&item.code)
}
