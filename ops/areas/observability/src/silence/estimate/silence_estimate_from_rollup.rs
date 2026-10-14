use super::*;

use std::collections::BTreeMap;

use crate::silence::silence_rollup as silence_silence_rollup;

pub fn summarize_scheduled_silence_rollups_for_estimate(
    items: &[silence_silence_rollup::ScheduledSilenceRollup],
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

pub fn derive_silence_estimates_from_rollup(
    items: &[silence_silence_rollup::ScheduledSilenceRollup],
) -> Vec<SilenceEstimate> {
    let mut out: Vec<SilenceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_silence_estimate(item: &SilenceEstimate) -> String {
    silence_silence_rollup::normalize_scheduled_silence_budget_code(&item.code)
}
