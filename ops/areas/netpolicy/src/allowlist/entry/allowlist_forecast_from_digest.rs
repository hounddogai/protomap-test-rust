use super::*;

use std::collections::BTreeMap;

use crate::flow_log::score as flow_log_score;

pub fn summarize_flow_log_digests_for_forecast(items: &[flow_log_score::FlowLogDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_allowlist_forecasts_from_digest(items: &[flow_log_score::FlowLogDigest]) -> Vec<AllowlistForecast> {
    let mut out: Vec<AllowlistForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_allowlist_forecast(item: &AllowlistForecast) -> String {
    flow_log_score::classify_flow_log_digest(item.effective_value())
}
