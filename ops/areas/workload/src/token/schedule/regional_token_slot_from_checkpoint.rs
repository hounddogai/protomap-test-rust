use super::*;

use std::collections::BTreeMap;

use crate::identity::plan as identity_plan;

pub fn summarize_pending_identity_checkpoints_for_slot(
    items: &[identity_plan::PendingIdentityCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_regional_token_slots_from_checkpoint(
    items: &[identity_plan::PendingIdentityCheckpoint],
) -> Vec<RegionalTokenSlot> {
    let mut out: Vec<RegionalTokenSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_token_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_regional_token_slot(item: &RegionalTokenSlot) -> String {
    identity_plan::normalize_global_identity_plan_code(&item.code)
}
