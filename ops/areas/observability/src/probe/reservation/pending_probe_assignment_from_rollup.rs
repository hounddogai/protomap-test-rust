use super::*;

use std::collections::BTreeMap;

use crate::silence::silence_rollup as silence_silence_rollup;

pub fn summarize_scheduled_silence_rollups_for_assignment(
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

pub fn derive_pending_probe_assignments_from_rollup(
    items: &[silence_silence_rollup::ScheduledSilenceRollup],
) -> Vec<PendingProbeAssignment> {
    let mut out: Vec<PendingProbeAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_probe_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_pending_probe_assignment(item: &PendingProbeAssignment) -> String {
    silence_silence_rollup::normalize_scheduled_silence_budget_code(&item.code)
}
