use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_proposal as retention_retention_proposal;

pub fn summarize_strict_retention_schedules_for_allocation(
    items: &[retention_retention_proposal::StrictRetentionSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_query_allocations_from_schedule(
    items: &[retention_retention_proposal::StrictRetentionSchedule],
) -> Vec<QueryAllocation> {
    let mut out: Vec<QueryAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_query_allocation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_query_allocation(item: &QueryAllocation) -> String {
    retention_retention_proposal::regional_retention_group_bucket(item.adjusted_total())
}
