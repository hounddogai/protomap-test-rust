use super::*;

use std::collections::BTreeMap;

use crate::dashboard::tier as dashboard_tier;

pub fn summarize_dashboard_templates_for_adjustment(
    items: &[dashboard_tier::DashboardTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_default_slo_adjustments_from_template(
    items: &[dashboard_tier::DashboardTemplate],
) -> Vec<DefaultSloAdjustment> {
    let mut out: Vec<DefaultSloAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_slo_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_default_slo_adjustment(item: &DefaultSloAdjustment) -> f64 {
    dashboard_tier::pending_dashboard_batch_weight(2, true)
}
