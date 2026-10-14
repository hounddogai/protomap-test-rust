use super::*;

use std::collections::BTreeMap;

use crate::postmortem::slot as postmortem_slot;

pub fn summarize_postmortem_slots_for_cycle(items: &[postmortem_slot::PostmortemSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_capped_postmortem_cycles_from_slot(
    items: &[postmortem_slot::PostmortemSlot],
) -> Vec<CappedPostmortemCycle> {
    let mut out: Vec<CappedPostmortemCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_postmortem_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_capped_postmortem_cycle(item: &CappedPostmortemCycle) -> String {
    postmortem_slot::postmortem_claim_bucket(item.adjusted_value())
}
