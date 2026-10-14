use super::*;

use std::collections::BTreeMap;

use crate::suspension::pending_suspension_metric as suspension_pending_suspension_metric;

pub fn summarize_fallback_suspension_budgets_for_template(
    items: &[suspension_pending_suspension_metric::FallbackSuspensionBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_capped_isolation_templates_from_budget(
    items: &[suspension_pending_suspension_metric::FallbackSuspensionBudget],
) -> Vec<CappedIsolationTemplate> {
    let mut out: Vec<CappedIsolationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_isolation_template(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_capped_isolation_template(item: &CappedIsolationTemplate) -> String {
    suspension_pending_suspension_metric::manual_suspension_quota_bucket(item.measured_value())
}
