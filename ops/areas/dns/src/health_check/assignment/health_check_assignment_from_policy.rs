use super::*;

use std::collections::BTreeMap;

use crate::zone::policy as zone_policy;

pub fn summarize_zone_policies_for_assignment(items: &[zone_policy::ZonePolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_health_check_assignments_from_policy(items: &[zone_policy::ZonePolicy]) -> Vec<HealthCheckAssignment> {
    let mut out: Vec<HealthCheckAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_assignment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_health_check_assignment(item: &HealthCheckAssignment) -> f64 {
    zone_policy::zone_policy_weight(3, true)
}
