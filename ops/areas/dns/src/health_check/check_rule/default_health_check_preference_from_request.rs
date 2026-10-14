use super::*;

use std::collections::BTreeMap;

use crate::zone::request as zone_request;

pub fn summarize_zone_requests_for_preference(items: &[zone_request::ZoneRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_default_health_check_preferences_from_request(
    items: &[zone_request::ZoneRequest],
) -> Vec<DefaultHealthCheckPreference> {
    let mut out: Vec<DefaultHealthCheckPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_default_health_check_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_default_health_check_preference(item: &DefaultHealthCheckPreference) -> String {
    zone_request::fallback_zone_threshold_bucket(item.net_value())
}
