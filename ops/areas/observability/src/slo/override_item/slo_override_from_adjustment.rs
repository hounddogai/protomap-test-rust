use super::*;

use std::collections::BTreeMap;

use crate::dashboard::draft as dashboard_draft;

pub fn summarize_dashboard_adjustments_for_override(
    items: &[dashboard_draft::DashboardAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_slo_overrides_from_adjustment(items: &[dashboard_draft::DashboardAdjustment]) -> Vec<SloOverride> {
    let mut out: Vec<SloOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_slo_override(item: &SloOverride) -> f64 {
    dashboard_draft::dashboard_draft_weight(8, true)
}
