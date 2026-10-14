use super::*;

use std::collections::BTreeMap;

use crate::reservation::summary as reservation_summary;

pub fn summarize_reservation_summaries_for_agreement(
    items: &[reservation_summary::ReservationSummary],
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

pub fn derive_usage_agreements_from_summary(items: &[reservation_summary::ReservationSummary]) -> Vec<UsageAgreement> {
    let mut out: Vec<UsageAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_agreement(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_usage_agreement(item: &UsageAgreement) -> String {
    reservation_summary::classify_reservation_profile(item.weighted_volume())
}
