use super::*;

use std::collections::BTreeMap;

use crate::runbook::record_item as runbook_record_item;

pub fn summarize_tiered_runbook_records_for_metric(
    items: &[runbook_record_item::TieredRunbookRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_capped_escalation_metrics_from_record(
    items: &[runbook_record_item::TieredRunbookRecord],
) -> Vec<CappedEscalationMetric> {
    let mut out: Vec<CappedEscalationMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_escalation_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_capped_escalation_metric(item: &CappedEscalationMetric) -> String {
    runbook_record_item::normalize_tiered_runbook_record_code(&item.code)
}
