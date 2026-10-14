use super::*;

use std::collections::BTreeMap;

use crate::retention::metric as retention_metric;

pub fn summarize_retention_metrics_for_budget(items: &[retention_metric::RetentionMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_audit_event_budgets_from_metric(items: &[retention_metric::RetentionMetric]) -> Vec<AuditEventBudget> {
    let mut out: Vec<AuditEventBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_audit_event_budget(item: &AuditEventBudget) -> String {
    retention_metric::classify_retention_segment(item.weighted_volume())
}
