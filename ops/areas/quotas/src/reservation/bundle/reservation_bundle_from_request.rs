use super::*;

use std::collections::BTreeMap;

use crate::grant::grant_template as grant_grant_template;

pub fn summarize_grant_requests_for_bundle(items: &[grant_grant_template::GrantRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_reservation_bundles_from_request(items: &[grant_grant_template::GrantRequest]) -> Vec<ReservationBundle> {
    let mut out: Vec<ReservationBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_reservation_bundle(item: &ReservationBundle) -> String {
    grant_grant_template::grant_request_bucket(item.effective_total())
}
