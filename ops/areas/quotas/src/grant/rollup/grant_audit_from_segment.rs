use super::*;

use std::collections::BTreeMap;

use crate::forecast::entry as forecast_entry;

pub fn summarize_forecast_segments_for_audit(items: &[forecast_entry::ForecastSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_grant_audits_from_segment(items: &[forecast_entry::ForecastSegment]) -> Vec<GrantAudit> {
    let mut out: Vec<GrantAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_audit(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_grant_audit(item: &GrantAudit) -> String {
    forecast_entry::normalize_manual_forecast_summary_code(&item.code)
}
