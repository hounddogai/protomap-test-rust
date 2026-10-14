use super::*;

use std::collections::BTreeMap;

use crate::flow_log::log_agreement as flow_log_log_agreement;

pub fn summarize_legacy_flow_log_variances_for_forecast(
    items: &[flow_log_log_agreement::LegacyFlowLogVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_egress_forecasts_from_variance(
    items: &[flow_log_log_agreement::LegacyFlowLogVariance],
) -> Vec<EgressForecast> {
    let mut out: Vec<EgressForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_egress_forecast(item: &EgressForecast) -> f64 {
    flow_log_log_agreement::primary_flow_log_agreement_weight(1, true)
}
