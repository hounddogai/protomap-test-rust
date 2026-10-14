use super::*;

use std::collections::BTreeMap;

use crate::identity::slot as identity_slot;

pub fn summarize_capped_identity_slots_for_summary(
    items: &[identity_slot::CappedIdentitySlot],
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

pub fn derive_identity_summaries_from_slot(items: &[identity_slot::CappedIdentitySlot]) -> Vec<IdentitySummary> {
    let mut out: Vec<IdentitySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_identity_summary(item: &IdentitySummary) -> String {
    identity_slot::capped_identity_slot_bucket(item.adjusted_load())
}
