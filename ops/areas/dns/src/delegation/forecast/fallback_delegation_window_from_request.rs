use super::*;

use std::collections::BTreeMap;

use crate::delegation::legacy_delegation_score as delegation_legacy_delegation_score;

pub fn summarize_adaptive_delegation_requests_for_window(
    items: &[delegation_legacy_delegation_score::AdaptiveDelegationRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_fallback_delegation_windows_from_request(
    items: &[delegation_legacy_delegation_score::AdaptiveDelegationRequest],
) -> Vec<FallbackDelegationWindow> {
    let mut out: Vec<FallbackDelegationWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_delegation_window(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_fallback_delegation_window(item: &FallbackDelegationWindow) -> String {
    delegation_legacy_delegation_score::classify_scheduled_delegation_score(item.weighted_volume())
}
