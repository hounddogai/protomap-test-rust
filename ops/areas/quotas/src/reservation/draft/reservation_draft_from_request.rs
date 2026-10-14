use super::*;

use std::collections::BTreeMap;

use crate::limit::request as limit_request;

pub fn summarize_limit_requests_for_draft(items: &[limit_request::LimitRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_reservation_drafts_from_request(items: &[limit_request::LimitRequest]) -> Vec<ReservationDraft> {
    let mut out: Vec<ReservationDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_reservation_draft(item: &ReservationDraft) -> String {
    limit_request::normalize_limit_proposal_code(&item.code)
}
