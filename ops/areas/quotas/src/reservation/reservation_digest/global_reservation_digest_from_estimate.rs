use super::*;

use std::collections::BTreeMap;

use crate::overage::estimate as overage_estimate;

pub fn summarize_overage_estimates_for_digest(items: &[overage_estimate::OverageEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_global_reservation_digests_from_estimate(
    items: &[overage_estimate::OverageEstimate],
) -> Vec<GlobalReservationDigest> {
    let mut out: Vec<GlobalReservationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_reservation_digest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_global_reservation_digest_extra(item: &GlobalReservationDigest) -> String {
    overage_estimate::normalize_overage_estimate_code(&item.code)
}
