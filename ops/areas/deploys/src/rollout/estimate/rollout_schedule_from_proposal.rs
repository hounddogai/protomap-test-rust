use super::*;

use std::collections::BTreeMap;

use crate::freeze::reservation as freeze_reservation;
use crate::rollout::reservation as rollout_reservation;

pub fn summarize_pending_rollout_proposals_for_schedule(
    items: &[rollout_reservation::PendingRolloutProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_rollout_schedules_from_proposal(
    items: &[rollout_reservation::PendingRolloutProposal],
) -> Vec<RolloutSchedule> {
    let mut out: Vec<RolloutSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollout_schedule(item: &RolloutSchedule) -> f64 {
    rollout_reservation::seasonal_rollout_preference_weight(1, true)
}

pub fn summarize_capped_freeze_proposals_for_schedule(
    items: &[freeze_reservation::CappedFreezeProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_rollout_schedules_from_proposal_extra(
    items: &[freeze_reservation::CappedFreezeProposal],
) -> Vec<RolloutSchedule> {
    let mut out: Vec<RolloutSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_rollout_schedule(item: &RolloutSchedule) -> String {
    freeze_reservation::freeze_allocation_bucket(item.adjusted_value())
}
