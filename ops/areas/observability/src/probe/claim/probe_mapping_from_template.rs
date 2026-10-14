use super::*;

use std::collections::BTreeMap;

use crate::dashboard::tier as dashboard_tier;

pub fn summarize_dashboard_templates_for_mapping(items: &[dashboard_tier::DashboardTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_probe_mappings_from_template(items: &[dashboard_tier::DashboardTemplate]) -> Vec<ProbeMapping> {
    let mut out: Vec<ProbeMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_mapping(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_probe_mapping(item: &ProbeMapping) -> String {
    dashboard_tier::classify_default_dashboard_digest(item.adjusted_exposure())
}
