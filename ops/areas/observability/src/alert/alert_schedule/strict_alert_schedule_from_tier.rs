use super::*;

use std::collections::BTreeMap;

use crate::silence::allocation as silence_allocation;

pub fn summarize_lenient_silence_tiers_for_schedule(
    items: &[silence_allocation::LenientSilenceTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_strict_alert_schedules_from_tier(
    items: &[silence_allocation::LenientSilenceTier],
) -> Vec<StrictAlertSchedule> {
    let mut out: Vec<StrictAlertSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_alert_schedule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_strict_alert_schedule(item: &StrictAlertSchedule) -> String {
    silence_allocation::normalize_silence_policy_code(&item.code)
}
