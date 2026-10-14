use super::*;

use std::collections::BTreeMap;

use crate::suspension::legacy_suspension_metric as suspension_legacy_suspension_metric;

pub fn summarize_strict_suspension_rollups_for_snapshot(
    items: &[suspension_legacy_suspension_metric::StrictSuspensionRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tenant_snapshots_from_rollup(
    items: &[suspension_legacy_suspension_metric::StrictSuspensionRollup],
) -> Vec<TenantSnapshot> {
    let mut out: Vec<TenantSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_tenant_snapshot(item: &TenantSnapshot) -> String {
    suspension_legacy_suspension_metric::default_suspension_metric_bucket(item.projected_volume())
}
