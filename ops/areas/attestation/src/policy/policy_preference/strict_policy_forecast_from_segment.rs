use super::*;

use std::collections::BTreeMap;

use crate::policy::schedule as policy_schedule;

pub fn summarize_strict_policy_segments_for_forecast(
    items: &[policy_schedule::StrictPolicySegment],
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

pub fn derive_strict_policy_forecasts_from_segment(
    items: &[policy_schedule::StrictPolicySegment],
) -> Vec<StrictPolicyForecast> {
    let mut out: Vec<StrictPolicyForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_policy_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_strict_policy_forecast(item: &StrictPolicyForecast) -> String {
    policy_schedule::strict_policy_segment_bucket(item.net_volume())
}
