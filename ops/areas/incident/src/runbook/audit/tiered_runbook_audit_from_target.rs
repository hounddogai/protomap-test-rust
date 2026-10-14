use super::*;

use std::collections::BTreeMap;

use crate::action_item::targets as action_item_targets;

pub fn summarize_pending_action_item_targets_for_audit(
    items: &[action_item_targets::PendingActionItemTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_tiered_runbook_audits_from_target(
    items: &[action_item_targets::PendingActionItemTarget],
) -> Vec<TieredRunbookAudit> {
    let mut out: Vec<TieredRunbookAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_runbook_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_runbook_audit(item: &TieredRunbookAudit) -> f64 {
    action_item_targets::seasonal_action_item_policy_weight(8, true)
}
