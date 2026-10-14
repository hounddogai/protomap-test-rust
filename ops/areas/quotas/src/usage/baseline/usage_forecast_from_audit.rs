use super::*;

use std::collections::BTreeMap;

use crate::usage::agreement as usage_agreement;

pub fn summarize_legacy_usage_audits_for_forecast(
    items: &[usage_agreement::LegacyUsageAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_usage_forecasts_from_audit(items: &[usage_agreement::LegacyUsageAudit]) -> Vec<UsageForecast> {
    let mut out: Vec<UsageForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_usage_forecast(item: &UsageForecast) -> f64 {
    usage_agreement::usage_agreement_weight(9, true)
}
