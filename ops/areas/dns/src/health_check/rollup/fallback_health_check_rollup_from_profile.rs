use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_checkpoint as delegation_delegation_checkpoint;

pub fn summarize_scheduled_delegation_profiles_for_rollup(
    items: &[delegation_delegation_checkpoint::ScheduledDelegationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_fallback_health_check_rollups_from_profile(
    items: &[delegation_delegation_checkpoint::ScheduledDelegationProfile],
) -> Vec<FallbackHealthCheckRollup> {
    let mut out: Vec<FallbackHealthCheckRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_health_check_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_health_check_rollup(item: &FallbackHealthCheckRollup) -> f64 {
    delegation_delegation_checkpoint::adaptive_delegation_override_weight(3, true)
}
