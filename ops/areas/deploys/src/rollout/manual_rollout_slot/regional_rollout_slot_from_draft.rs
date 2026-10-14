use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_draft as freeze_freeze_draft;

pub fn summarize_fallback_freeze_drafts_for_slot(
    items: &[freeze_freeze_draft::FallbackFreezeDraft],
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

pub fn derive_regional_rollout_slots_from_draft(
    items: &[freeze_freeze_draft::FallbackFreezeDraft],
) -> Vec<RegionalRolloutSlot> {
    let mut out: Vec<RegionalRolloutSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_rollout_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_regional_rollout_slot(item: &RegionalRolloutSlot) -> f64 {
    freeze_freeze_draft::seasonal_freeze_entry_weight(9, true)
}
