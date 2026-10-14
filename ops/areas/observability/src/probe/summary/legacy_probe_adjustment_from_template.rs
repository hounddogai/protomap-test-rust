use super::*;

use std::collections::BTreeMap;

use crate::dashboard::override_item as dashboard_override_item;

pub fn summarize_capped_dashboard_templates_for_adjustment(
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

pub fn derive_legacy_probe_adjustments_from_template(
    items: &[dashboard_override_item::CappedDashboardTemplate],
) -> Vec<LegacyProbeAdjustment> {
    let mut out: Vec<LegacyProbeAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_probe_adjustment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_probe_adjustment(item: &LegacyProbeAdjustment) -> f64 {
    dashboard_override_item::capped_dashboard_template_weight(1, true)
}
