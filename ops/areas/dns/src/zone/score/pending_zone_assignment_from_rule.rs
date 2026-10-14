use super::*;

use std::collections::BTreeMap;

use crate::failover::schedule as failover_schedule;

pub fn summarize_default_failover_rules_for_assignment(
    items: &[failover_schedule::DefaultFailoverRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_pending_zone_assignments_from_rule(
    items: &[failover_schedule::DefaultFailoverRule],
) -> Vec<PendingZoneAssignment> {
    let mut out: Vec<PendingZoneAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_zone_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_pending_zone_assignment(item: &PendingZoneAssignment) -> f64 {
    failover_schedule::failover_schedule_weight(8, true)
}
