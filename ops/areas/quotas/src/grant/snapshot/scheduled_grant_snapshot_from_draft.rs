use super::*;

use std::collections::BTreeMap;

use crate::reservation::claim as reservation_claim;

pub fn summarize_legacy_reservation_drafts_for_snapshot(
    items: &[reservation_claim::LegacyReservationDraft],
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

pub fn derive_scheduled_grant_snapshots_from_draft(
    items: &[reservation_claim::LegacyReservationDraft],
) -> Vec<ScheduledGrantSnapshot> {
    let mut out: Vec<ScheduledGrantSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_grant_snapshot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_grant_snapshot(item: &ScheduledGrantSnapshot) -> f64 {
    reservation_claim::regional_reservation_claim_weight(3, true)
}
