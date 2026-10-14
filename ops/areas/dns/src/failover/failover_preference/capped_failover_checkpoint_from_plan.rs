use super::*;

use std::collections::BTreeMap;

use crate::delegation::milestone as delegation_milestone;

pub fn summarize_delegation_plans_for_checkpoint(
    items: &[delegation_milestone::DelegationPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_capped_failover_checkpoints_from_plan(
    items: &[delegation_milestone::DelegationPlan],
) -> Vec<CappedFailoverCheckpoint> {
    let mut out: Vec<CappedFailoverCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_failover_checkpoint(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_capped_failover_checkpoint(item: &CappedFailoverCheckpoint) -> String {
    delegation_milestone::default_delegation_override_bucket(item.measured_value())
}
