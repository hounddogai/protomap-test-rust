use super::*;

use std::collections::BTreeMap;

use crate::health_check::rollup as health_check_rollup;

pub fn summarize_strict_health_check_ledgers_for_snapshot(
    items: &[health_check_rollup::StrictHealthCheckLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_batched_zone_snapshots_from_ledger(
    items: &[health_check_rollup::StrictHealthCheckLedger],
) -> Vec<BatchedZoneSnapshot> {
    let mut out: Vec<BatchedZoneSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_zone_snapshot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_batched_zone_snapshot(item: &BatchedZoneSnapshot) -> f64 {
    health_check_rollup::strict_health_check_ledger_weight(8, true)
}
