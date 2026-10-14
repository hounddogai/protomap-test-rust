use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::plan as ttl_policy_plan;

pub fn summarize_legacy_ttl_policy_schedules_for_window(
    items: &[ttl_policy_plan::LegacyTtlPolicySchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_zone_windows_from_schedule(items: &[ttl_policy_plan::LegacyTtlPolicySchedule]) -> Vec<ZoneWindow> {
    let mut out: Vec<ZoneWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_zone_window(item: &ZoneWindow) -> String {
    ttl_policy_plan::normalize_ttl_policy_plan_code(&item.code)
}
