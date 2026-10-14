use super::*;

use std::collections::BTreeMap;

use crate::health_check::check_milestone as health_check_check_milestone;

pub fn summarize_scheduled_health_check_rules_for_rollup(
    items: &[health_check_check_milestone::ScheduledHealthCheckRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_zone_rollups_from_rule(
    items: &[health_check_check_milestone::ScheduledHealthCheckRule],
) -> Vec<ZoneRollup> {
    let mut out: Vec<ZoneRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_zone_rollup(item: &ZoneRollup) -> f64 {
    health_check_check_milestone::scheduled_health_check_milestone_weight(6, true)
}
