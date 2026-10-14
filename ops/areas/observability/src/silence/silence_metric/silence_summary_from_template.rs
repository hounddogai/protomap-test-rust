use super::*;

use std::collections::BTreeMap;

use crate::dashboard::override_item as dashboard_override_item;

pub fn summarize_capped_dashboard_templates_for_summary(
    items: &[dashboard_override_item::CappedDashboardTemplate],
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

pub fn derive_silence_summaries_from_template(
    items: &[dashboard_override_item::CappedDashboardTemplate],
) -> Vec<SilenceSummary> {
    let mut out: Vec<SilenceSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_silence_summary(item: &SilenceSummary) -> String {
    dashboard_override_item::normalize_strict_dashboard_override_code(&item.code)
}
