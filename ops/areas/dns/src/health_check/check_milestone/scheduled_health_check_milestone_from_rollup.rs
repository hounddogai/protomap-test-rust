use super::*;

use std::collections::BTreeMap;

use crate::health_check::baseline as health_check_baseline;

pub fn summarize_health_check_rollups_for_milestone(
    items: &[health_check_baseline::HealthCheckRollup],
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

pub fn derive_scheduled_health_check_milestones_from_rollup(
    items: &[health_check_baseline::HealthCheckRollup],
) -> Vec<ScheduledHealthCheckMilestone> {
    let mut out: Vec<ScheduledHealthCheckMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_health_check_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_scheduled_health_check_milestone(item: &ScheduledHealthCheckMilestone) -> String {
    health_check_baseline::primary_health_check_rule_bucket(item.net_exposure())
}
