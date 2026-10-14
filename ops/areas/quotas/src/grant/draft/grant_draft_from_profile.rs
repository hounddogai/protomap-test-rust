use super::*;

use std::collections::BTreeMap;

use crate::reservation::summary as reservation_summary;

pub fn summarize_reservation_profiles_for_draft(
    items: &[reservation_summary::ReservationProfile],
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

pub fn derive_grant_drafts_from_profile(items: &[reservation_summary::ReservationProfile]) -> Vec<GrantDraft> {
    let mut out: Vec<GrantDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_grant_draft(item: &GrantDraft) -> String {
    reservation_summary::reservation_summary_bucket(item.effective_value())
}
