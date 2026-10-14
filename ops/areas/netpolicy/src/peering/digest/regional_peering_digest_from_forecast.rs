use super::*;

use std::collections::BTreeMap;

use crate::flow_log::forecast as flow_log_forecast;

pub fn summarize_flow_log_forecasts_for_digest(items: &[flow_log_forecast::FlowLogForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_regional_peering_digests_from_forecast(
    items: &[flow_log_forecast::FlowLogForecast],
) -> Vec<RegionalPeeringDigest> {
    let mut out: Vec<RegionalPeeringDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_peering_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_regional_peering_digest(item: &RegionalPeeringDigest) -> String {
    flow_log_forecast::classify_flow_log_checkpoint(item.weighted_total())
}
