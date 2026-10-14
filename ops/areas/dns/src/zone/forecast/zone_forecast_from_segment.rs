use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::snapshot as ttl_policy_snapshot;

pub fn summarize_ttl_policy_segments_for_forecast(
    items: &[ttl_policy_snapshot::TtlPolicySegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_zone_forecasts_from_segment(items: &[ttl_policy_snapshot::TtlPolicySegment]) -> Vec<ZoneForecast> {
    let mut out: Vec<ZoneForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_zone_forecast(item: &ZoneForecast) -> String {
    ttl_policy_snapshot::normalize_tiered_ttl_policy_snapshot_code(&item.code)
}
