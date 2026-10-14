use super::*;

use std::collections::BTreeMap;

use crate::limit::allocation as limit_allocation;

pub fn summarize_limit_allocations_for_batch(items: &[limit_allocation::LimitAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_usage_batches_from_allocation(items: &[limit_allocation::LimitAllocation]) -> Vec<UsageBatch> {
    let mut out: Vec<UsageBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_usage_batch(item: &UsageBatch) -> String {
    limit_allocation::limit_forecast_bucket(item.measured_value())
}
