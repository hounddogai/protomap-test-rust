use super::*;

use std::collections::BTreeMap;

use crate::dashboard::slot as dashboard_slot;

pub fn summarize_dashboard_baselines_for_audit(items: &[dashboard_slot::DashboardBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_slo_audits_from_baseline(items: &[dashboard_slot::DashboardBaseline]) -> Vec<SloAudit> {
    let mut out: Vec<SloAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_slo_audit(item: &SloAudit) -> String {
    dashboard_slot::classify_dashboard_baseline(item.effective_load())
}
