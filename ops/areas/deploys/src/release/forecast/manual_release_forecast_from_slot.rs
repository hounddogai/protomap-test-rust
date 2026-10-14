use super::*;

use std::collections::BTreeMap;

use crate::rollout::rollout_slot as rollout_rollout_slot;

pub fn summarize_scheduled_rollout_slots_for_forecast(
    items: &[rollout_rollout_slot::ScheduledRolloutSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_manual_release_forecasts_from_slot(
    items: &[rollout_rollout_slot::ScheduledRolloutSlot],
) -> Vec<ManualReleaseForecast> {
    let mut out: Vec<ManualReleaseForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_release_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_manual_release_forecast(item: &ManualReleaseForecast) -> String {
    rollout_rollout_slot::normalize_rollout_rule_code(&item.code)
}
