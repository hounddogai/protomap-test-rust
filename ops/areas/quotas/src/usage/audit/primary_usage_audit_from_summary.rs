use super::*;

use std::collections::BTreeMap;

use crate::forecast::entry as forecast_entry;

pub fn summarize_manual_forecast_summaries_for_audit(
    items: &[forecast_entry::ManualForecastSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_primary_usage_audits_from_summary(
    items: &[forecast_entry::ManualForecastSummary],
) -> Vec<PrimaryUsageAudit> {
    let mut out: Vec<PrimaryUsageAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_usage_audit(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_usage_audit(item: &PrimaryUsageAudit) -> f64 {
    forecast_entry::forecast_assignment_weight(4, true)
}
