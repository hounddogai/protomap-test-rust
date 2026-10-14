use super::*;

use std::collections::BTreeMap;

use crate::slo::group as slo_group;

pub fn summarize_weighted_slo_reservations_for_milestone(
    items: &[slo_group::WeightedSloReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_manual_probe_milestones_from_reservation(
    items: &[slo_group::WeightedSloReservation],
) -> Vec<ManualProbeMilestone> {
    let mut out: Vec<ManualProbeMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_probe_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_manual_probe_milestone(item: &ManualProbeMilestone) -> String {
    slo_group::slo_estimate_bucket(item.projected_volume())
}
