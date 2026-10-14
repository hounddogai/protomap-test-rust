use super::*;

use std::collections::BTreeMap;

use crate::reservation::policy as reservation_policy;

pub fn summarize_global_reservation_snapshots_for_rollup(
    items: &[reservation_policy::GlobalReservationSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_limit_rollups_from_snapshot(items: &[reservation_policy::GlobalReservationSnapshot]) -> Vec<LimitRollup> {
    let mut out: Vec<LimitRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_limit_rollup(item: &LimitRollup) -> String {
    reservation_policy::normalize_reservation_plan_code(&item.code)
}
