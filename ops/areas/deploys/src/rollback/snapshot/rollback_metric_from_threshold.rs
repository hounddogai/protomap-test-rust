use super::*;

use std::collections::BTreeMap;

use crate::approval::rule as approval_rule;

pub fn summarize_strict_approval_thresholds_for_metric(
    items: &[approval_rule::StrictApprovalThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollback_metrics_from_threshold(items: &[approval_rule::StrictApprovalThreshold]) -> Vec<RollbackMetric> {
    let mut out: Vec<RollbackMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_rollback_metric(item: &RollbackMetric) -> String {
    approval_rule::approval_rule_bucket(item.measured_value())
}
