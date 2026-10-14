use super::*;

use std::collections::BTreeMap;

use crate::grant::allocation as grant_allocation;

pub fn summarize_manual_grant_reservations_for_checkpoint(
    items: &[grant_allocation::ManualGrantReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_usage_checkpoints_from_reservation(
    items: &[grant_allocation::ManualGrantReservation],
) -> Vec<UsageCheckpoint> {
    let mut out: Vec<UsageCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_checkpoint(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_usage_checkpoint(item: &UsageCheckpoint) -> f64 {
    grant_allocation::grant_allocation_weight(8, true)
}
