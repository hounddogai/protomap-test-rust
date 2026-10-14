use super::*;

use std::collections::BTreeMap;

use crate::health_check::slot as health_check_slot;

pub fn summarize_health_check_groups_for_segment(
    items: &[health_check_slot::HealthCheckGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_health_check_segments_from_group(
    items: &[health_check_slot::HealthCheckGroup],
) -> Vec<HealthCheckSegment> {
    let mut out: Vec<HealthCheckSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_health_check_segment(item: &HealthCheckSegment) -> String {
    health_check_slot::health_check_slot_bucket(item.adjusted_exposure())
}
