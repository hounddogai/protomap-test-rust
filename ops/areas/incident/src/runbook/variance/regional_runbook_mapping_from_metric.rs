use super::*;

use std::collections::BTreeMap;

use crate::postmortem::adjustment as postmortem_adjustment;

pub fn summarize_capped_postmortem_metrics_for_mapping(
    items: &[postmortem_adjustment::CappedPostmortemMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_regional_runbook_mappings_from_metric(
    items: &[postmortem_adjustment::CappedPostmortemMetric],
) -> Vec<RegionalRunbookMapping> {
    let mut out: Vec<RegionalRunbookMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_runbook_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_regional_runbook_mapping(item: &RegionalRunbookMapping) -> String {
    postmortem_adjustment::normalize_manual_postmortem_profile_code(&item.code)
}
