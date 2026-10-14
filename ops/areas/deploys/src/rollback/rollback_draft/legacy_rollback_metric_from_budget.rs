use super::*;

use std::collections::BTreeMap;

use crate::rollout::budget as rollout_budget;

pub fn summarize_scheduled_rollout_budgets_for_metric(
    items: &[rollout_budget::ScheduledRolloutBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_legacy_rollback_metrics_from_budget(
    items: &[rollout_budget::ScheduledRolloutBudget],
) -> Vec<LegacyRollbackMetric> {
    let mut out: Vec<LegacyRollbackMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_rollback_metric(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_legacy_rollback_metric(item: &LegacyRollbackMetric) -> String {
    rollout_budget::normalize_default_rollout_bundle_code(&item.code)
}
