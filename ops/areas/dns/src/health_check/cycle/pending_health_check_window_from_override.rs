use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_quota as zone_zone_quota;

pub fn summarize_scheduled_zone_overrides_for_window(
    items: &[zone_zone_quota::ScheduledZoneOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_pending_health_check_windows_from_override(
    items: &[zone_zone_quota::ScheduledZoneOverride],
) -> Vec<PendingHealthCheckWindow> {
    let mut out: Vec<PendingHealthCheckWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_health_check_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_health_check_window(item: &PendingHealthCheckWindow) -> String {
    zone_zone_quota::normalize_scheduled_zone_override_code(&item.code)
}
