use super::*;

use std::collections::BTreeMap;

use crate::zone::manifest as zone_manifest;

pub fn summarize_zone_plans_for_batch(items: &[zone_manifest::ZonePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_failover_batches_from_plan(items: &[zone_manifest::ZonePlan]) -> Vec<FailoverBatch> {
    let mut out: Vec<FailoverBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_batch(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_failover_batch(item: &FailoverBatch) -> String {
    zone_manifest::normalize_zone_plan_code(&item.code)
}
