use super::*;

use std::collections::BTreeMap;

use crate::grant::snapshot as grant_snapshot;

pub fn summarize_scheduled_grant_snapshots_for_plan(
    items: &[grant_snapshot::ScheduledGrantSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_reservation_plans_from_snapshot(
    items: &[grant_snapshot::ScheduledGrantSnapshot],
) -> Vec<ReservationPlan> {
    let mut out: Vec<ReservationPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_plan(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_reservation_plan(item: &ReservationPlan) -> String {
    grant_snapshot::classify_grant_mapping(item.weighted_exposure())
}
