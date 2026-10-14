use super::*;

use std::collections::BTreeMap;

use crate::alert::budget as alert_budget;

pub fn summarize_rolling_alert_slots_for_preference(items: &[alert_budget::RollingAlertSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_default_silence_preferences_from_slot(
    items: &[alert_budget::RollingAlertSlot],
) -> Vec<DefaultSilencePreference> {
    let mut out: Vec<DefaultSilencePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_default_silence_preference(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_default_silence_preference(item: &DefaultSilencePreference) -> String {
    alert_budget::classify_rolling_alert_slot(item.effective_volume())
}
