use super::*;

use std::collections::BTreeMap;

use crate::federation::allocation as federation_allocation;

pub fn summarize_legacy_federation_schedules_for_tier(
    items: &[federation_allocation::LegacyFederationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_adaptive_credential_tiers_from_schedule(
    items: &[federation_allocation::LegacyFederationSchedule],
) -> Vec<AdaptiveCredentialTier> {
    let mut out: Vec<AdaptiveCredentialTier> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_credential_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_adaptive_credential_tier(item: &AdaptiveCredentialTier) -> String {
    federation_allocation::normalize_legacy_federation_adjustment_code(&item.code)
}
