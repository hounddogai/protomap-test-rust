use super::*;

use std::collections::BTreeMap;

use crate::probe::record_item as probe_record_item;

pub fn summarize_adaptive_probe_entries_for_template(
    items: &[probe_record_item::AdaptiveProbeEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_dashboard_templates_from_entry(
    items: &[probe_record_item::AdaptiveProbeEntry],
) -> Vec<DashboardTemplate> {
    let mut out: Vec<DashboardTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_dashboard_template(item: &DashboardTemplate) -> String {
    probe_record_item::probe_claim_bucket(item.effective_value())
}
