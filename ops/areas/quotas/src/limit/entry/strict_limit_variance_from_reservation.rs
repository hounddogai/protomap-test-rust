use super::*;

use std::collections::BTreeMap;

use crate::overage::summary as overage_summary;

pub fn summarize_capped_overage_reservations_for_variance(
    items: &[overage_summary::CappedOverageReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_strict_limit_variances_from_reservation(
    items: &[overage_summary::CappedOverageReservation],
) -> Vec<StrictLimitVariance> {
    let mut out: Vec<StrictLimitVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_limit_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_strict_limit_variance(item: &StrictLimitVariance) -> String {
    overage_summary::classify_capped_overage_reservation(item.projected_exposure())
}
