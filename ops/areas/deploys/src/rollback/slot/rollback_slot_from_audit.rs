use super::*;

use std::collections::BTreeMap;

use crate::rollback::tier as rollback_tier;

pub fn summarize_rollback_audits_for_slot(items: &[rollback_tier::RollbackAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_rollback_slots_from_audit(items: &[rollback_tier::RollbackAudit]) -> Vec<RollbackSlot> {
    let mut out: Vec<RollbackSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollback_slot(item: &RollbackSlot) -> f64 {
    rollback_tier::legacy_rollback_milestone_weight(8, true)
}
