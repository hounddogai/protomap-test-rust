use super::*;

use std::collections::BTreeMap;

use crate::isolation::estimate as isolation_estimate;

pub fn summarize_seasonal_isolation_estimates_for_snapshot(
    items: &[isolation_estimate::SeasonalIsolationEstimate],
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

pub fn derive_tenant_snapshots_from_estimate(
    items: &[isolation_estimate::SeasonalIsolationEstimate],
) -> Vec<TenantSnapshot> {
    let mut out: Vec<TenantSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_snapshot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_tenant_snapshot(item: &TenantSnapshot) -> f64 {
    isolation_estimate::default_isolation_record_weight(6, true)
}
