use super::*;

use std::collections::BTreeMap;

use crate::isolation::segment as isolation_segment;

pub fn summarize_default_isolation_segments_for_schedule(
    items: &[isolation_segment::DefaultIsolationSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_primary_tenant_schedules_from_segment(
    items: &[isolation_segment::DefaultIsolationSegment],
) -> Vec<PrimaryTenantSchedule> {
    let mut out: Vec<PrimaryTenantSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_tenant_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_tenant_schedule(item: &PrimaryTenantSchedule) -> String {
    isolation_segment::isolation_adjustment_bucket(item.projected_exposure())
}
