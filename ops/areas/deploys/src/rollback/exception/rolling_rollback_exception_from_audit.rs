use super::*;

use std::collections::BTreeMap;

use crate::rollout::rule as rollout_rule;

pub fn summarize_capped_rollout_audits_for_exception(
    items: &[rollout_rule::CappedRolloutAudit],
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

pub fn derive_rolling_rollback_exceptions_from_audit(
    items: &[rollout_rule::CappedRolloutAudit],
) -> Vec<RollingRollbackException> {
    let mut out: Vec<RollingRollbackException> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_rollback_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_rollback_exception(item: &RollingRollbackException) -> String {
    rollout_rule::classify_primary_rollout_mapping(item.weighted_value())
}
