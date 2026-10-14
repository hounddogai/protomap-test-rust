use super::*;

use std::collections::BTreeMap;

use crate::approval::entry as approval_entry;

pub fn summarize_approval_reservations_for_tier(
    items: &[approval_entry::ApprovalReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_lenient_freeze_tiers_from_reservation(
    items: &[approval_entry::ApprovalReservation],
) -> Vec<LenientFreezeTier> {
    let mut out: Vec<LenientFreezeTier> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_freeze_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_lenient_freeze_tier(item: &LenientFreezeTier) -> String {
    approval_entry::normalize_pending_approval_checkpoint_code(&item.code)
}
