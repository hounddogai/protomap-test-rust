use super::*;

use std::collections::BTreeMap;

use crate::grant::plan as grant_plan;

pub fn summarize_grant_notices_for_draft(items: &[grant_plan::GrantNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_strict_reservation_drafts_from_notice(items: &[grant_plan::GrantNotice]) -> Vec<StrictReservationDraft> {
    let mut out: Vec<StrictReservationDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_reservation_draft(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_strict_reservation_draft(item: &StrictReservationDraft) -> String {
    grant_plan::tiered_grant_ledger_bucket(item.weighted_value())
}
