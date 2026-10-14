use super::*;

use std::collections::BTreeMap;

use crate::isolation::record_item as isolation_record_item;

pub fn summarize_tiered_isolation_estimates_for_rollup(
    items: &[isolation_record_item::TieredIsolationEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_adaptive_tenant_rollups_from_estimate(
    items: &[isolation_record_item::TieredIsolationEstimate],
) -> Vec<AdaptiveTenantRollup> {
    let mut out: Vec<AdaptiveTenantRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_tenant_rollup(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_adaptive_tenant_rollup(item: &AdaptiveTenantRollup) -> String {
    isolation_record_item::classify_fallback_isolation_record(item.projected_exposure())
}
