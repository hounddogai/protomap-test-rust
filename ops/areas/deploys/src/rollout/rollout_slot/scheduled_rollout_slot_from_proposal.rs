use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_forecast as canary_canary_forecast;

pub fn summarize_canary_proposals_for_slot(items: &[canary_canary_forecast::CanaryProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_scheduled_rollout_slots_from_proposal(
    items: &[canary_canary_forecast::CanaryProposal],
) -> Vec<ScheduledRolloutSlot> {
    let mut out: Vec<ScheduledRolloutSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rollout_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_scheduled_rollout_slot(item: &ScheduledRolloutSlot) -> String {
    canary_canary_forecast::legacy_canary_forecast_bucket(item.net_total())
}
