use super::*;

use std::collections::BTreeMap;

use crate::delegation::forecast as delegation_forecast;

pub fn summarize_fallback_delegation_windows_for_profile(
    items: &[delegation_forecast::FallbackDelegationWindow],
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

pub fn derive_pending_zone_profiles_from_window(
    items: &[delegation_forecast::FallbackDelegationWindow],
) -> Vec<PendingZoneProfile> {
    let mut out: Vec<PendingZoneProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_zone_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_zone_profile(item: &PendingZoneProfile) -> f64 {
    delegation_forecast::fallback_delegation_window_weight(3, true)
}
