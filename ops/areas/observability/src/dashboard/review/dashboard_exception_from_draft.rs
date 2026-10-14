use super::*;

use std::collections::BTreeMap;

use crate::slo::metric as slo_metric;

pub fn summarize_primary_slo_drafts_for_exception(items: &[slo_metric::PrimarySloDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_dashboard_exceptions_from_draft(items: &[slo_metric::PrimarySloDraft]) -> Vec<DashboardException> {
    let mut out: Vec<DashboardException> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_dashboard_exception(item: &DashboardException) -> f64 {
    slo_metric::default_slo_metric_weight(8, true)
}
