use super::*;

use std::collections::BTreeMap;

use crate::suspension::quota as suspension_quota;

pub fn summarize_pending_suspension_schedules_for_rollup(
    items: &[suspension_quota::PendingSuspensionSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_weighted_tenant_rollups_from_schedule(
    items: &[suspension_quota::PendingSuspensionSchedule],
) -> Vec<WeightedTenantRollup> {
    let mut out: Vec<WeightedTenantRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_tenant_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_weighted_tenant_rollup(item: &WeightedTenantRollup) -> String {
    suspension_quota::classify_scheduled_suspension_allocation(item.net_volume())
}
