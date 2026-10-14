use super::*;

use std::collections::BTreeMap;

use crate::alert::batched_alert_cycle as alert_batched_alert_cycle;

pub fn summarize_strict_alert_cycles_for_draft(
    items: &[alert_batched_alert_cycle::StrictAlertCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_fallback_dashboard_drafts_from_cycle(
    items: &[alert_batched_alert_cycle::StrictAlertCycle],
) -> Vec<FallbackDashboardDraft> {
    let mut out: Vec<FallbackDashboardDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_dashboard_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_dashboard_draft(item: &FallbackDashboardDraft) -> String {
    alert_batched_alert_cycle::classify_seasonal_alert_entry(item.adjusted_total())
}
